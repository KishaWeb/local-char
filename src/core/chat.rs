use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: String, 
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chat {
    pub title: String,
    pub messages: Vec<Message>,
}