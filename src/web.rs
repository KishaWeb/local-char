use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

type History = HashMap<String, Vec<Message>>;

#[derive(Clone)]
pub struct AppState {
    pub history: Arc<Mutex<History>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub chat_id: String,
    pub message: String,
    pub character_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CharacterFile {
    pub characters: Vec<Character>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub system_prompt: String,
    pub backstory: String,
    pub greeting: String,
    pub relationships: Vec<String>,
    pub signature_lines: Vec<String>,
    pub switch_hint: String,
}

const HISTORY_PATH: &str = "/home/arshia/local-char/history.json";
const CHARACTER_PATH: &str = "/home/arshia/local-char/src/characters/character.json";

pub async fn run(lan: bool) {
    let addr = if lan {
        println!("LAN mode enabled");
        "0.0.0.0:7878"
    } else {
        println!("Local mode enabled");
        "127.0.0.1:7878"
    };

    let state = AppState {
        history: Arc::new(Mutex::new(load_history())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/script.js", get(script))
        .route("/chat", post(chat))
        .route("/new_chat", post(new_chat))
        .route("/characters", get(characters))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    println!("Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let html = fs::read_to_string("src/web/index.html")
        .unwrap_or_else(|_| "missing index.html".to_string());

    Html(html)
}

async fn script() -> impl IntoResponse {
    let js = fs::read_to_string("src/web/script.js")
        .unwrap_or_else(|_| "console.log('missing script')".to_string());

    ([("Content-Type", "application/javascript")], js)
}

async fn characters() -> impl IntoResponse {
    let chars = load_characters();

    let list: Vec<_> = chars
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "greeting": c.greeting,
                "switch_hint": c.switch_hint
            })
        })
        .collect();

    Json(list)
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let characters = load_characters();

    let character = characters
        .into_iter()
        .find(|c| c.id == req.character_id)
        .unwrap_or(Character {
            id: "default".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            backstory: "".to_string(),
            greeting: "Hello".to_string(),
            relationships: vec![],
            signature_lines: vec![],
            switch_hint: "neutral".to_string(),
        });

    {
        let mut history = state.history.lock().unwrap();

        history
            .entry(req.chat_id.clone())
            .or_default()
            .push(Message {
                role: "user".to_string(),
                content: req.message.clone(),
            });
    }

    let mut messages = {
        let history = state.history.lock().unwrap();
        history.get(&req.chat_id).cloned().unwrap_or_default()
    };

    messages.insert(
        0,
        Message {
            role: "system".to_string(),
            content: character.system_prompt.clone(),
        },
    );

    if messages.len() > 30 {
        let start = messages.len() - 30;
        messages = messages[start..].to_vec();
    }

    let client = reqwest::Client::new();

    let res = client
        .post("http://127.0.0.1:8080/v1/chat/completions")
        .json(&json!({
            "model": "local-model",
            "messages": messages
        }))
        .send()
        .await;

    let ai = match res {
        Ok(r) => {
            let v: serde_json::Value = r.json().await.unwrap_or_default();

            v["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("no response")
                .to_string()
        }
        Err(_) => "llama.cpp error".to_string(),
    };

    {
        let mut history = state.history.lock().unwrap();

        history
            .entry(req.chat_id.clone())
            .or_default()
            .push(Message {
                role: "assistant".to_string(),
                content: ai.clone(),
            });

        save_history(&history);
    }

    Json(json!({
        "response": ai,
        "character": character.id
    }))
}

async fn new_chat(State(state): State<AppState>) -> impl IntoResponse {
    let mut history = state.history.lock().unwrap();

    let id = format!("chat_{}", history.len() + 1);

    history.insert(id.clone(), vec![]);

    save_history(&history);

    Json(json!({ "chat_id": id }))
}

fn load_history() -> History {
    fs::read_to_string(HISTORY_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_history(history: &History) {
    let data = serde_json::to_string_pretty(history).unwrap();
    let _ = fs::write(HISTORY_PATH, data);
}

fn load_characters() -> Vec<Character> {
    fs::read_to_string(CHARACTER_PATH)
        .ok()
        .and_then(|s| serde_json::from_str::<CharacterFile>(&s).ok())
        .map(|c| c.characters)
        .unwrap_or_default()
}