#[cfg(feature = "ssr")]
use grammers_client::{Client, Config};
#[cfg(feature = "ssr")]
use grammers_session::Session;
#[cfg(feature = "ssr")]
use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use strum::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i32,
    pub text: String,
    pub formatted_text: Vec<TextEntity>,
    pub timestamp: i64,
    pub sender: String,
    pub chat_id: i64,
    pub message_type: MessageType,
    pub media_info: Option<MediaInfo>,
    pub reply_to: Option<i32>,
    pub forwarded_from: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextEntity {
    pub offset: usize,
    pub length: usize,
    pub entity_type: EntityType,
    pub url: Option<String>, // For links
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityType {
    Bold,
    Italic,
    Code,
    Pre,
    Link,
    TextLink,
    Mention,
    Hashtag,
    BotCommand,
    Email,
    Phone,
    Underline,
    Strikethrough,
    Spoiler,
}

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
pub enum MessageType {
    Text,
    Photo,
    Video,
    Document,
    Audio,
    Voice,
    Sticker,
    Location,
    Contact,
    Poll,
    System,
}

impl MessageType {
    pub fn get_emoji(self) -> &'static str {
        match self {
            MessageType::Text => "💬",
            MessageType::Photo => "📷",
            MessageType::Video => "🎥", 
            MessageType::Document => "📄",
            MessageType::Audio => "🎵",
            MessageType::Voice => "🎤",
            MessageType::Sticker => "😀",
            MessageType::Location => "📍",
            MessageType::Contact => "👤",
            MessageType::Poll => "📊",
            MessageType::System => "⚙️",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,
    pub caption: Option<String>,
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
    // First, let's try to get all dialogs to find our chat
    let mut dialogs = client.iter_dialogs();
    let mut target_chat = None;
    let mut found_ids = Vec::new();
    
    while let Some(dialog) = dialogs.next().await? {
        let dialog_chat = dialog.chat();
        let dialog_id = dialog_chat.id();
        found_ids.push(dialog_id);
        
        // Debug logging
        eprintln!("Found chat: ID={}, Name={}", dialog_id, dialog_chat.name());
        
        // Check if this is our target chat
        if dialog_id == chat_id {
            target_chat = Some(dialog_chat.clone());
            break;
        }
    }
    
    let chat = match target_chat {
        Some(chat) => chat,
        None => {
            eprintln!("Chat with ID {} not found. Available chat IDs: {:?}", chat_id, found_ids);
            return Err(format!("Chat with ID {} not found in dialogs", chat_id).into());
        }
    };
    
    eprintln!("Found chat, attempting to get messages from: {}", chat.name());
    
    let mut messages = Vec::new();
    let mut iter = client.iter_messages(chat);
    
    for i in 0..limit {
        if let Some(message) = iter.next().await? {
            let (message_type, media_info, text) = classify_message(&message);
            let formatted_text = extract_text_entities(&message);
            
            eprintln!("Message {}: ID={}, Type={:?}, Text={}", i + 1, message.id(), message_type, text);
            
            messages.push(ChatMessage {
                id: message.id(),
                text,
                formatted_text,
                timestamp: message.date().timestamp(),
                sender: message.sender().map(|s| s.name().to_string()).unwrap_or_else(|| "Unknown".to_string()),
                chat_id,
                message_type,
                media_info,
                reply_to: message.reply_to_message_id(),
                forwarded_from: None, // TODO: Extract forward info properly
            });
        } else {
            eprintln!("No more messages after {} messages", i);
            break;
        }
    }
    
    eprintln!("Retrieved {} messages total", messages.len());
    
    // Reverse to show oldest first
    messages.reverse();
    Ok(messages)
}

#[cfg(feature = "ssr")]
fn classify_message(message: &grammers_client::types::Message) -> (MessageType, Option<MediaInfo>, String) {
    use grammers_client::types::Media;
    
    // Check if message has media
    if let Some(media) = message.media() {
        match media {
            Media::Photo(_photo) => {
                let caption = message.text().to_string();
                let media_info = MediaInfo {
                    file_name: None,
                    file_size: None, // TODO: Extract photo size properly
                    mime_type: Some("image/jpeg".to_string()),
                    caption: if caption.is_empty() { None } else { Some(caption.clone()) },
                };
                let text = if caption.is_empty() { "[Photo]".to_string() } else { caption };
                (MessageType::Photo, Some(media_info), text)
            },
            Media::Document(doc) => {
                let caption = message.text().to_string();
                let mime_type = doc.mime_type().map(|m| m.to_string());
                
                // Determine if it's a video, audio, or document
                let message_type = match mime_type.as_deref() {
                    Some(t) if t.starts_with("video/") => MessageType::Video,
                    Some(t) if t.starts_with("audio/") => MessageType::Audio,
                    _ => MessageType::Document,
                };
                
                let media_info = MediaInfo {
                    file_name: Some(doc.name().to_string()),
                    file_size: Some(doc.size() as u64),
                    mime_type,
                    caption: if caption.is_empty() { None } else { Some(caption.clone()) },
                };
                
                let text = if caption.is_empty() {
                    match message_type {
                        MessageType::Video => "[Video]".to_string(),
                        MessageType::Audio => "[Audio]".to_string(),
                        _ => format!("[Document: {}]", doc.name()),
                    }
                } else { 
                    caption 
                };
                
                (message_type, Some(media_info), text)
            },
            Media::Sticker(_) => {
                (MessageType::Sticker, None, "[Sticker]".to_string())
            },
            Media::Contact(_) => {
                (MessageType::Contact, None, "[Contact]".to_string())
            },
            // Media::Location(_) => {
            //     (MessageType::Location, None, "[Location]".to_string())
            // },
            Media::Poll(_) => {
                (MessageType::Poll, None, "[Poll]".to_string())
            },
            _ => {
                let text = message.text().to_string();
                let display_text = if text.is_empty() { "[Media]".to_string() } else { text };
                (MessageType::Text, None, display_text)
            }
        }
    } else {
        // Text message
        let text = message.text().to_string();
        if text.is_empty() {
            (MessageType::System, None, "[System Message]".to_string())
        } else {
            (MessageType::Text, None, text)
        }
    }
}

#[cfg(feature = "ssr")]
fn extract_text_entities(message: &grammers_client::types::Message) -> Vec<TextEntity> {
    let mut entities = Vec::new();
    
    // Try different possible methods to access entities from grammers-client
    eprintln!("Extracting entities from message...");
    
    // Method 1: Try fmt_entities()
    if let Some(fmt_entities) = try_extract_fmt_entities(message) {
        entities.extend(fmt_entities);
        eprintln!("Found {} entities via fmt_entities", entities.len());
        return entities;
    }
    
    // Method 2: Try accessing raw entities
    if let Some(raw_entities) = try_extract_raw_entities(message) {
        entities.extend(raw_entities);
        eprintln!("Found {} entities via raw access", entities.len());
        return entities;
    }
    
    // Method 3: We CANNOT parse markdown because Telegram doesn't send markdown!
    eprintln!("No entities found. Plain text only: {}", message.text());
    eprintln!("Formatted text is IMPOSSIBLE without access to Telegram entities!");
    
    // Return empty - no formatting possible without entities
    entities
}

#[cfg(feature = "ssr")]
fn try_extract_fmt_entities(message: &grammers_client::types::Message) -> Option<Vec<TextEntity>> {
    eprintln!("Trying to access message entities...");
    eprintln!("Message text: {:?}", message.text());
    
    // Check what methods are available on the message
    // The entities MUST be accessible somehow, as they're part of the Telegram message
    
    // Try accessing fmt_entities() directly
    if let Some(entities) = message.fmt_entities() {
        eprintln!("\n=== SUCCESS! fmt_entities() returned {} entities! ===", entities.len());
        
        // Debug print everything we can about entities
        eprintln!("Entities type: {:?}", std::any::type_name_of_val(&entities));
        eprintln!("Full entities debug: {:?}", entities);
        
        // Try to access individual entities
        for (i, entity) in entities.iter().enumerate() {
            eprintln!("\nEntity {}: {:?}", i, entity);
            // Once we see this output, we'll know EXACTLY how to extract offset/length/type
        }
        
        eprintln!("\n=== Converting entities to TextEntity format ===");
        
        // Now convert grammers entities to our TextEntity format
        return Some(convert_grammers_entities_to_text_entities(entities));
    }
    
    eprintln!("fmt_entities() returned None - no formatting in this message");
    None
}

#[cfg(feature = "ssr")]
fn convert_grammers_entities_to_text_entities(entities: &[grammers_tl_types::enums::MessageEntity]) -> Vec<TextEntity> {
    let mut result = Vec::new();
    
    for entity in entities {
        let (offset, length, entity_type, url) = match entity {
            grammers_tl_types::enums::MessageEntity::Bold(bold) => {
                (bold.offset, bold.length, EntityType::Bold, None)
            },
            grammers_tl_types::enums::MessageEntity::Italic(italic) => {
                (italic.offset, italic.length, EntityType::Italic, None)
            },
            grammers_tl_types::enums::MessageEntity::Code(code) => {
                (code.offset, code.length, EntityType::Code, None)
            },
            grammers_tl_types::enums::MessageEntity::Pre(pre) => {
                (pre.offset, pre.length, EntityType::Pre, None)
            },
            grammers_tl_types::enums::MessageEntity::TextUrl(text_url) => {
                (text_url.offset, text_url.length, EntityType::TextLink, Some(text_url.url.clone()))
            },
            grammers_tl_types::enums::MessageEntity::Url(url) => {
                (url.offset, url.length, EntityType::Link, None)
            },
            grammers_tl_types::enums::MessageEntity::Mention(mention) => {
                (mention.offset, mention.length, EntityType::Mention, None)
            },
            grammers_tl_types::enums::MessageEntity::Hashtag(hashtag) => {
                (hashtag.offset, hashtag.length, EntityType::Hashtag, None)
            },
            grammers_tl_types::enums::MessageEntity::BotCommand(bot_command) => {
                (bot_command.offset, bot_command.length, EntityType::BotCommand, None)
            },
            grammers_tl_types::enums::MessageEntity::Email(email) => {
                (email.offset, email.length, EntityType::Email, None)
            },
            grammers_tl_types::enums::MessageEntity::Phone(phone) => {
                (phone.offset, phone.length, EntityType::Phone, None)
            },
            _ => {
                eprintln!("Unknown entity type: {:?}", entity);
                continue;
            }
        };
        
        result.push(TextEntity {
            offset: offset as usize,
            length: length as usize,
            entity_type: entity_type.clone(),
            url,
        });
        
        eprintln!("Converted entity: offset={}, length={}, type={:?}", offset, length, entity_type);
    }
    
    eprintln!("Successfully converted {} entities!", result.len());
    result
}

#[cfg(feature = "ssr")]
fn try_extract_raw_entities(_message: &grammers_client::types::Message) -> Option<Vec<TextEntity>> {
    // Based on research, grammers-client doesn't easily expose entities
    // We'll need to rely on text parsing instead
    None
}

#[cfg(feature = "ssr")]
fn extract_markdown_entities(text: &str) -> Vec<TextEntity> {
    let mut entities = Vec::new();
    
    eprintln!("Parsing text for markdown entities: {}", text);
    
    // Multi-line code blocks ```code```
    if let Ok(re) = regex::Regex::new(r"```([^`]*)```") {
        for mat in re.find_iter(text) {
            eprintln!("Found code block at {}-{}: {}", mat.start(), mat.end(), mat.as_str());
            entities.push(TextEntity {
                offset: mat.start(),
                length: mat.len(),
                entity_type: EntityType::Pre,
                url: None,
            });
        }
    }
    
    // **bold** (multi-line)
    if let Ok(re) = regex::Regex::new(r"(?s)\*\*([^*]*?)\*\*") {
        for mat in re.find_iter(text) {
            eprintln!("Found bold at {}-{}: {}", mat.start(), mat.end(), mat.as_str());
            entities.push(TextEntity {
                offset: mat.start(),
                length: mat.len(),
                entity_type: EntityType::Bold,
                url: None,
            });
        }
    }
    
    // *italic* (but not **bold**)
    if let Ok(re) = regex::Regex::new(r"(?s)(?<!\*)\*([^*]+?)\*(?!\*)") {
        for mat in re.find_iter(text) {
            eprintln!("Found italic at {}-{}: {}", mat.start(), mat.end(), mat.as_str());
            entities.push(TextEntity {
                offset: mat.start(),
                length: mat.len(),
                entity_type: EntityType::Italic,
                url: None,
            });
        }
    }
    
    // `inline code`
    if let Ok(re) = regex::Regex::new(r"`([^`]+)`") {
        for mat in re.find_iter(text) {
            eprintln!("Found inline code at {}-{}: {}", mat.start(), mat.end(), mat.as_str());
            entities.push(TextEntity {
                offset: mat.start(),
                length: mat.len(),
                entity_type: EntityType::Code,
                url: None,
            });
        }
    }
    
    // URLs
    if let Ok(re) = regex::Regex::new(r"https?://[^\s]+") {
        for mat in re.find_iter(text) {
            eprintln!("Found URL at {}-{}: {}", mat.start(), mat.end(), mat.as_str());
            entities.push(TextEntity {
                offset: mat.start(),
                length: mat.len(),
                entity_type: EntityType::Link,
                url: Some(mat.as_str().to_string()),
            });
        }
    }
    
    // Sort entities by offset to handle overlapping correctly
    entities.sort_by_key(|e| e.offset);
    
    eprintln!("Found {} total entities", entities.len());
    entities
}


