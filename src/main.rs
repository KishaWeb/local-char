use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

#[derive(Deserialize, Clone)]
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
}

#[derive(Clone, Serialize, Deserialize)]
struct Message {
    timestamp: u64,
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ChatSession {
    name: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct SessionFile {
    chats: Vec<ChatSession>,
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

fn pick_chat(chats: &[String]) -> io::Result<usize> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let items: Vec<ListItem> = chats
                .iter()
                .map(|c| ListItem::new(c.clone()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Chats")
                        .borders(Borders::ALL),
                )
                .highlight_symbol("> ");

            let mut state = ListState::default();

            state.select(Some(selected));

            f.render_stateful_widget(list, area, &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < chats.len() {
                        selected += 1;
                    }
                }

                KeyCode::Enter => {
                    disable_raw_mode()?;

                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen
                    )?;

                    return Ok(selected);
                }

                KeyCode::Esc => {
                    disable_raw_mode()?;

                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen
                    )?;

                    return Ok(0);
                }

                _ => {}
            }
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn load_sessions() -> SessionFile {
    match fs::read_to_string("history.json") {
        Ok(data) => serde_json::from_str(&data).unwrap_or(SessionFile {
            chats: vec![ChatSession {
                name: "Chat 1".to_string(),
                messages: vec![],
            }],
        }),
        Err(_) => {
            let init = SessionFile {
                chats: vec![ChatSession {
                    name: "Chat 1".to_string(),
                    messages: vec![],
                }],
            };

            let _ = fs::write("history.json", serde_json::to_string_pretty(&init).unwrap());
            init
        }
    }
}

fn save_sessions(sessions: &SessionFile) {
    let data = serde_json::to_string_pretty(sessions).unwrap();
    fs::write("history.json", data).unwrap();
}

fn load_characters() -> CharacterFile {
    let data = fs::read_to_string("src/characters/character.json")
        .expect("failed to read character file");

    serde_json::from_str(&data).expect("failed to parse json")
}

fn get_character(file: &CharacterFile, id: &str) -> Option<Character> {
    file.characters.iter().find(|c| c.id == id).cloned()
}

fn build_system_prompt(c: &Character) -> String {
    format!(
        "{}\n\nBackstory:\n{}\n\nRelationships:\n{}",
        c.system_prompt,
        c.backstory,
        c.relationships.join(", ")
    )
}

fn main() {
    let client = Client::new();
    let file = load_characters();
    let mut sessions = load_sessions();
    let new_chat_number = sessions.chats.len() + 1;
    sessions.chats.push(ChatSession {
        name: format!("Chat {}", new_chat_number),
        messages: vec![],
    });
    let mut current_chat = sessions.chats.len() - 1;
    save_sessions(&sessions);

    let mut character: Option<Character> = None;

    println!("Available characters:");
    for c in &file.characters {
        println!("- {}", c.id);
    }

    print!("Choose character id (or 'none'): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input != "none" {
        character = get_character(&file, input);
        if let Some(c) = &character {
            println!("\nAI: {}\n", c.greeting);
        }
    }

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

        match parts[0] {
            "/exit" => break,

            "/help" => {
                println!("/list");
                println!("/char <id>");
                println!("/exit");
                println!("/chats (placeholder)");
                continue;
            }

            "/list" => {
                for c in &file.characters {
                    println!("- {}", c.id);
                }
                continue;
            }

            "/char" => {
                if parts.len() < 2 {
                    println!("usage: /char <id>");
                    continue;
                }

                if parts[1] == "none" {
                    character = None;
                    println!("character cleared");
                    continue;
                }

                if let Some(c) = get_character(&file, parts[1]) {
                    character = Some(c);
                    println!("switched character");
                } else {
                    println!("not found");
                }

                continue;
            }

            "/newchat" => {
                let new_name = format!("Chat {}", sessions.chats.len() + 1);

                sessions.chats.push(ChatSession {
                    name: new_name.clone(),
                    messages: vec![],
                });

                current_chat = sessions.chats.len() - 1;

                save_sessions(&sessions);

                println!("created and switched to {}", new_name);

                continue;
            }

            "/chats" => {
                for (i, chat) in sessions.chats.iter().enumerate() {
                    if i == current_chat {
                        println!("* {} ({})", i + 1, chat.name);
                    } else {
                        println!("  {} ({})", i + 1, chat.name);
                    }
                }

                continue;
            }

            "/switch" => {
                let mut names: Vec<String> = vec!["[New Chat]".to_string()];

                names.extend(
                    sessions.chats.iter().map(|c| c.name.clone())
                );

                let selected = pick_chat(&names).unwrap();

                if selected == 0 {
                    let new_name = format!("Chat {}", sessions.chats.len() + 1);

                    sessions.chats.push(ChatSession {
                        name: new_name.clone(),
                        messages: vec![],
                    });

                    current_chat = sessions.chats.len() - 1;

                    save_sessions(&sessions);

                    println!("created and switched to {}", new_name);
                } else {
                    current_chat = selected - 1;

                    println!(
                        "switched to {}",
                        sessions.chats[current_chat].name
                    );
                }

                continue;
            }

            _ => {}
        }

        let chat = &mut sessions.chats[current_chat];

        chat.messages.push(Message {
            timestamp: now(),
            role: "user".to_string(),
            content: user_input.clone(),
        });

        let mut api_messages = vec![];

        if let Some(c) = &character {
            api_messages.push(json!({
                "role": "system",
                "content": build_system_prompt(c)
            }));
        }

        for m in &chat.messages {
            api_messages.push(json!({
                "role": m.role,
                "content": m.content
            }));
        }

        let body = json!({
            "model": "local-model",
            "messages": api_messages
        });

        let response = client
            .post("http://127.0.0.1:8080/v1/chat/completions")
            .json(&body)
            .send();

        if response.is_err() {
            println!("request failed");
            continue;
        }

        let response: ChatResponse = response.unwrap().json().unwrap();

        let reply = response.choices[0].message.content.clone();

        println!("AI: {}\n", reply);

        chat.messages.push(Message {
            timestamp: now(),
            role: "assistant".to_string(),
            content: reply,
        });

        save_sessions(&sessions);
    }

    save_sessions(&sessions);
}