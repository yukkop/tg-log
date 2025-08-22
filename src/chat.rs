use leptos::prelude::*;
use leptos::html;
use serde::{Deserialize, Serialize};
use crate::telegram::ChatMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub loading: bool,
    pub error: Option<String>,
    pub chat_id: Option<i64>,
    pub has_more_history: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            loading: false,
            error: None,
            chat_id: None,
            has_more_history: true,
        }
    }
}

#[component]
pub fn ChatInterface() -> impl IntoView {
    let scroll_container_ref = NodeRef::<html::Div>::new();
    let is_auto_scroll = RwSignal::new(true);
    
    // Use Resource::new for SSR data loading
    let messages_resource: Resource<Result<Vec<ChatMessage>, String>> = Resource::new(
        || (),
        move |_| async move {
            #[cfg(feature = "ssr")]
            {
                use crate::telegram::{create_telegram_client, get_chat_history};
                use crate::config::TARGET_CHAT;
                
                match create_telegram_client().await {
                    Ok(client) => {
                        match get_chat_history(&client, *TARGET_CHAT, 50).await {
                            Ok(messages) => Ok(messages),
                            Err(e) => Err(format!("Failed to load messages: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Failed to connect to Telegram: {}", e)),
                }
            }
            #[cfg(not(feature = "ssr"))]
            {
                // Client-side fallback
                Ok(vec![
                    ChatMessage {
                        id: 1,
                        text: "Welcome to the chat! (Demo mode - not connected to Telegram)".to_string(),
                        formatted_text: vec![],
                        timestamp: chrono::Utc::now().timestamp(),
                        sender: "System".to_string(),
                        chat_id: -1001234567890,
                        message_type: crate::telegram::MessageType::System,
                        media_info: None,
                        reply_to: None,
                        forwarded_from: None,
                    }
                ])
            }
        }
    );

    // Auto-scroll to bottom when new messages arrive
    Effect::new(move |_| {
        let messages = messages_resource.get();
        if let Some(Ok(messages)) = messages {
            if is_auto_scroll.get() && !messages.is_empty() {
                if let Some(container) = scroll_container_ref.get() {
                    container.set_scroll_top(container.scroll_height());
                }
            }
        }
    });

    // Handle scroll events to detect if user is at bottom
    let on_scroll = move |_| {
        if let Some(container) = scroll_container_ref.get() {
            let scroll_top = container.scroll_top();
            let scroll_height = container.scroll_height();
            let client_height = container.client_height();
            
            // Check if scrolled to bottom (with small tolerance)
            let at_bottom = scroll_top + client_height >= scroll_height - 10;
            is_auto_scroll.set(at_bottom);
        }
    };

    view! {
        <div class="chat-container">
            <div class="chat-header">
                <h2>"Telegram Chat"</h2>
                {move || {
                    #[cfg(feature = "ssr")]
                    {
                        use crate::config::TARGET_CHAT;
                        view! { <span class="chat-id">"Chat ID: " {*TARGET_CHAT}</span> }.into_any()
                    }
                    #[cfg(not(feature = "ssr"))]
                    {
                        view! { <span class="chat-id">"Chat ID: Demo Mode"</span> }.into_any()
                    }
                }}
            </div>
            
            <div 
                class="messages-container"
                node_ref=scroll_container_ref
                on:scroll=on_scroll
            >
                <Suspense
                    fallback=move || view! {
                        <div class="loading-indicator">
                            "Loading messages..."
                        </div>
                    }
                >
                    {move || {
                        let messages = messages_resource.get();
                        
                        match messages {
                            None => view! { <div></div> }.into_any(),
                            Some(Ok(messages)) => {
                                if messages.is_empty() {
                                    view! {
                                        <div class="no-messages">
                                            "No messages found in this chat."
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <>
                                            {messages.into_iter().map(|message| {
                                                view! {
                                                    <MessageComponent message=message />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </>
                                    }.into_any()
                                }
                            }
                            Some(Err(error)) => {
                                view! {
                                    <div class="error-message">
                                        {error}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
            
            {move || {
                if !is_auto_scroll.get() {
                    view! {
                        <button 
                            class="scroll-to-bottom"
                            on:click=move |_| {
                                if let Some(container) = scroll_container_ref.get() {
                                    container.set_scroll_top(container.scroll_height());
                                }
                                is_auto_scroll.set(true);
                            }
                        >
                            "↓ Scroll to bottom"
                        </button>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn MessageComponent(message: ChatMessage) -> impl IntoView {
    let formatted_time = {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(message.timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        dt.format("%H:%M:%S").to_string()
    };

    let message_class = format!("message message-type-{}", 
        message.message_type.to_string()
    );

    view! {
        <div class={message_class}>
            // Reply indicator
            {message.reply_to.map(|reply_id| {
                view! {
                    <div class="reply-indicator">
                        "↳ Reply to message #{}" {reply_id}
                    </div>
                }.into_any()
            })}
            
            // Forward indicator
            {message.forwarded_from.as_ref().map(|from| {
                view! {
                    <div class="forward-indicator">
                        "↪ Forwarded from: " {from.clone()}
                    </div>
                }.into_any()
            })}
            
            <div class="message-header">
                <span class="sender">{message.sender}</span>
                <div class="message-meta">
                    <span class="message-type-badge">{message.message_type.get_emoji()}</span>
                    <span class="timestamp">{formatted_time}</span>
                </div>
            </div>
            
            // Media info
            {message.media_info.as_ref().map(|media| {
                render_media_info(media)
            })}
            
            <div class="message-text">
                {render_formatted_text(&message.text, &message.formatted_text)}
            </div>
        </div>
    }
}

fn render_media_info(media: &crate::telegram::MediaInfo) -> impl IntoView {
    view! {
        <div class="media-info">
            {media.file_name.as_ref().map(|name| {
                view! {
                    <div class="file-name">"📎 " {name.clone()}</div>
                }.into_any()
            })}
            {media.file_size.map(|size| {
                view! {
                    <div class="file-size">Size: {format_file_size(size)}</div>
                }.into_any()
            })}
            {media.mime_type.as_ref().map(|mime| {
                view! {
                    <div class="mime-type">"Type: " {mime.clone()}</div>
                }.into_any()
            })}
        </div>
    }
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn render_formatted_text(text: &str, entities: &[crate::telegram::TextEntity]) -> impl IntoView {
    use crate::telegram::EntityType;
    
    if entities.is_empty() {
        return view! { <span>{text}</span> }.into_any();
    }
    
    let mut sorted_entities = entities.to_vec();
    sorted_entities.sort_by_key(|e| e.offset);
    
    let mut result_parts = Vec::new();
    let mut current_pos = 0;
    
    for entity in sorted_entities {
        // Add text before the entity
        if entity.offset > current_pos {
            let before_text = &text[current_pos..entity.offset];
            if !before_text.is_empty() {
                result_parts.push(view! { <span>{before_text}</span> }.into_any());
            }
        }
        
        // Add the formatted entity
        let entity_end = entity.offset + entity.length;
        let entity_text = &text[entity.offset..entity_end];
        
        let formatted_element = match entity.entity_type {
            EntityType::Bold => {
                // Telegram entities point to plain text - no markdown stripping needed
                view! { <strong>{entity_text}</strong> }.into_any()
            },
            EntityType::Italic => {
                // Telegram entities point to plain text - no markdown stripping needed
                view! { <em>{entity_text}</em> }.into_any()
            },
            EntityType::Code => {
                // Telegram entities point to plain text - no markdown stripping needed
                view! { <code class="inline-code">{entity_text}</code> }.into_any()
            },
            EntityType::Pre => {
                // Telegram entities point to plain text - no markdown stripping needed
                view! { <pre class="code-block">{entity_text}</pre> }.into_any()
            },
            EntityType::Link | EntityType::TextLink => {
                let href = entity.url.as_deref().unwrap_or(entity_text);
                view! { 
                    <a href={href} target="_blank" rel="noopener noreferrer" class="message-link">
                        {entity_text}
                    </a> 
                }.into_any()
            },
            EntityType::Mention => view! { 
                <span class="mention">{entity_text}</span> 
            }.into_any(),
            EntityType::Hashtag => view! { 
                <span class="hashtag">{entity_text}</span> 
            }.into_any(),
            EntityType::BotCommand => view! { 
                <span class="bot-command">{entity_text}</span> 
            }.into_any(),
            EntityType::Email => view! { 
                <a href={format!("mailto:{}", entity_text)} class="email-link">
                    {entity_text}
                </a> 
            }.into_any(),
            EntityType::Phone => view! { 
                <a href={format!("tel:{}", entity_text)} class="phone-link">
                    {entity_text}
                </a> 
            }.into_any(),
            EntityType::Underline => view! { 
                <span class="underline">{entity_text}</span> 
            }.into_any(),
            EntityType::Strikethrough => view! { 
                <span class="strikethrough">{entity_text}</span> 
            }.into_any(),
            EntityType::Spoiler => view! { 
                <span class="spoiler">{entity_text}</span> 
            }.into_any(),
        };
        
        result_parts.push(formatted_element);
        current_pos = entity_end;
    }
    
    // Add any remaining text
    if current_pos < text.len() {
        let remaining_text = &text[current_pos..];
        if !remaining_text.is_empty() {
            result_parts.push(view! { <span>{remaining_text}</span> }.into_any());
        }
    }
    
    view! {
        <span class="formatted-text">
            {result_parts}
        </span>
    }.into_any()
}
