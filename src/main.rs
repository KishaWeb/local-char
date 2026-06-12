use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{self, Write};

#[derive(Deserialize)]
struct CharacterFile {
    characters: Vec<Character>,
}

#[derive(Deserialize, Clone)]
struct Character {
    id: String,
    name: String,
    system_prompt: String,
    backstory: String,
    greeting: String,
    relationships: Vec<String>,
}

#[derive(Clone, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: RespMessage,
}

#[derive(Deserialize)]
struct RespMessage {
    content: String,
}

fn load_characters() -> CharacterFile {
    let data = fs::read_to_string("src/characters/character.json")
        .expect("failed to read character file");

    serde_json::from_str(&data)
        .expect("failed to parse json")
}

fn build_system_prompt(c: &Character) -> String {
    format!(
        "{}\n\nBackstory:\n{}\n\nRelationships:\n{}",
        c.system_prompt,
        c.backstory,
        c.relationships.join(", ")
    )
}

fn get_character(file: &CharacterFile, id: &str) -> Option<Character> {
    file.characters.iter().find(|c| c.id == id).cloned()
}

fn main() {
    let client = Client::new();

    let file = load_characters();

    println!("Available characters:");
    for c in &file.characters {
        println!("- {}", c.id);
    }

    print!("\nChoose character id: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    let character = get_character(&file, input)
        .expect("Character not found");

    println!("\nAI: {}\n", character.greeting);

    let system_prompt = build_system_prompt(&character);

    let mut messages: Vec<Message> = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
        }
    ];

    loop {
        print!("You: ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim().to_string();

        if user_input == "exit" {
            break;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: user_input,
        });

        let body = json!({
            "model": "local-model",
            "messages": messages
        });

        let response: ChatResponse = client
            .post("http://127.0.0.1:8080/v1/chat/completions")
            .json(&body)
            .send()
            .unwrap()
            .json()
            .unwrap();

        let reply = response.choices[0].message.content.clone();

        println!("AI: {}\n", reply);

        messages.push(Message {
            role: "assistant".to_string(),
            content: reply,
        });
    }
}