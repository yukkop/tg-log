#[cfg(feature = "ssr")]
use grammers_client::{Client, Config};
#[cfg(feature = "ssr")]
use grammers_session::Session;
#[cfg(feature = "ssr")]
use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i32,
    pub text: String,
    pub timestamp: i64,
    pub sender: String,
    pub chat_id: i64,
}

#[derive(Clone, Debug)]
pub struct ChatHistory {
    pub messages: VecDeque<ChatMessage>,
    pub max_size: usize,
}

impl ChatHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        if self.messages.len() >= self.max_size {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.iter().cloned().collect()
    }
}

#[cfg(feature = "ssr")]
pub async fn create_telegram_client() -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let session = Session::load_file_or_create("session")?;
    
    let config = Config {
        session,
        api_id: *API_ID,
        api_hash: API_HASH.clone(),
        params: Default::default(),
    };

    let client = Client::connect(config).await?;
    
    if !client.is_authorized().await? {
        return Err("Telegram client not authorized. Please run the CLI tool first to authenticate.".into());
    }
    
    Ok(client)
}

#[cfg(feature = "ssr")]
pub async fn get_chat_history(client: &Client, chat_id: i64, limit: i32) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
    use grammers_client::types::Chat;
    
    let chat = match client.resolve_username(&format!("{}", chat_id)).await {
        Ok(Some(Chat::User(user))) => Chat::User(user),
        Ok(Some(Chat::Group(group))) => Chat::Group(group),
        Ok(Some(Chat::Channel(channel))) => Chat::Channel(channel),
        _ => {
            // Try to get chat by ID directly
            return Ok(vec![]); // For now, return empty if we can't resolve
        }
    };
    
    let mut messages = Vec::new();
    let mut iter = client.iter_messages(&chat);
    
    for _ in 0..limit {
        if let Some(message) = iter.next().await? {
            messages.push(ChatMessage {
                id: message.id(),
                text: message.text().to_string(),
                timestamp: message.date().timestamp(),
                sender: message.sender().map(|s| s.name().to_string()).unwrap_or_else(|| "Unknown".to_string()),
                chat_id,
            });
        } else {
            break;
        }
    }
    
    // Reverse to show oldest first
    messages.reverse();
    Ok(messages)
}
