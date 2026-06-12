use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::io;

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

fn main() -> io::Result<()> {
    while true{
        let client = Client::new();
        let mut buffer = String::new();

        println!("You:");
        io::stdin().read_line(&mut buffer)?;

        let body = json!({
            "model": "local-model",
            "messages": [{
                "role": "user",
                "content": buffer.trim()
            }],
            "max_tokens": 128
        });

        let response = client
            .post("http://127.0.0.1:8080/v1/chat/completions")
            .json(&body)
            .send();

        match response {
            Ok(r) => {
                let chat: ChatResponse = r.json().unwrap();
                println!("{}", chat.choices[0].message.content);
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }

        println!("character");

    }
    Ok(())
}