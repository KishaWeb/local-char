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
    system_prompt: String,
    backstory: String,
    greeting: String,
    relationships: Vec<String>,
    signature_lines: Vec<String>,
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
        "{}\n\nBackstory:\n{}\n\nRelationships:\n{}\n\nSignature lines (use only in emotional moments):\n- {}",
        c.system_prompt,
        c.backstory,
        c.relationships.join(", "),
        c.signature_lines.join("\n- ")
    )
}

fn get_character(file: &CharacterFile, id: &str) -> Option<Character> {
    file.characters.iter().find(|c| c.id == id).cloned()
}

fn init_messages(character: &Character) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: format!(
                "{}\n\nBackstory:\n{}\n\nRelationships:\n{}\n\nSignature lines:\n- {}",
                character.system_prompt,
                character.backstory,
                character.relationships.join(", "),
                character.signature_lines.join("\n- ")
            ),
        }
    ]
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

    let mut character = get_character(&file, input)
        .expect("Character not found");

    println!("\nAI: {}\n", character.greeting);

    let mut messages = init_messages(&character);

    loop {
        print!("You: ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();

        let user_input = user_input.trim().to_string();

        let parts: Vec<&str> = user_input.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        let command = parts[0];

        match command {
            "/list" => {
                println!("Available characters:");
                for c in &file.characters {
                    println!("- {}", c.id);
                }
                continue;
            }

            "/exit" => break,

            "/help" => {
                println!("/list - show characters");
                println!("/exit - quit");
                continue;
            }

            "/char" => {
                if parts.len() < 2 {
                    println!("Usage: /char <id>");
                    continue;
                }

                let new_id = parts[1];

                let new_char = match get_character(&file, new_id) {
                    Some(c) => c,
                    None => {
                        println!("Character not found.");
                        continue;
                    }
                };

                character = new_char;

                println!("\nSwitched to: {}\n", character.id);
                println!("AI: {}\n", character.greeting);

                messages = init_messages(&character);

                continue;
            }

            _ => {}
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