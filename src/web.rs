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
struct AppState {
    history: Arc<Mutex<History>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    chat_id: String,
    message: String,
}

const HISTORY_PATH: &str = "/home/arshia/local-char/history.json";

pub async fn run() {
    let state = AppState {
        history: Arc::new(Mutex::new(load_history())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/script.js", get(script))
        .route("/chat", post(chat))
        .route("/new_chat", post(new_chat))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7878")
        .await
        .unwrap();

    println!("Server running: http://127.0.0.1:7878");

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let html = fs::read_to_string("src/web/index.html")
        .unwrap_or_else(|_| "missing index.html".to_string());

    Html(html)
}

async fn script() -> impl IntoResponse {
    let js = fs::read_to_string("src/web/script.js")
        .unwrap_or_else(|_| "console.log('missing script');".to_string());

    ([("Content-Type", "application/javascript")], js)
}

async fn new_chat(State(state): State<AppState>) -> impl IntoResponse {
    let mut history = state.history.lock().unwrap();

    let id = format!("chat_{}", history.len() + 1);

    history.insert(id.clone(), vec![]);

    save_history(&history);

    println!("NEW CHAT CREATED: {}", id);

    Json(json!({ "chat_id": id }))
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    println!("CHAT HIT");

    {
        let mut history = state.history.lock().unwrap();

        let chat = history.entry(req.chat_id.clone()).or_default();

        chat.push(Message {
            role: "user".to_string(),
            content: req.message.clone(),
        });
    }

    let snapshot = {
        let history = state.history.lock().unwrap();

        history
            .get(&req.chat_id)
            .cloned()
            .unwrap_or_default()
    };

    let client = reqwest::Client::new();

    let res = client
        .post("http://127.0.0.1:8080/v1/chat/completions")
        .json(&json!({
            "model": "local-model",
            "messages": snapshot
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

        let chat = history.entry(req.chat_id.clone()).or_default();

        chat.push(Message {
            role: "assistant".to_string(),
            content: ai.clone(),
        });

        save_history(&history);
    }

    Json(json!({ "response": ai }))
}

fn load_history() -> History {
    fs::read_to_string(HISTORY_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(HashMap::new)
}

fn save_history(history: &History) {
    let data = serde_json::to_string_pretty(history).unwrap();

    match std::fs::write(HISTORY_PATH, &data) {
        Ok(_) => {
            println!("SAVE OK -> {}", HISTORY_PATH);
        }
        Err(e) => {
            println!("SAVE FAILED: {:?}", e);
        }
    }

    // extra verification read-back
    match std::fs::read_to_string(HISTORY_PATH) {
        Ok(v) => println!("READBACK OK, size = {}", v.len()),
        Err(e) => println!("READBACK FAILED: {:?}", e),
    }
}