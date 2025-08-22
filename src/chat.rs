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
                        timestamp: chrono::Utc::now().timestamp(),
                        sender: "System".to_string(),
                        chat_id: -1001234567890,
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
                {move || {
                    let messages = messages_resource.get();
                    
                    match messages {
                        None => {
                            view! {
                                <div class="loading-indicator">
                                    "Loading messages..."
                                </div>
                            }.into_any()
                        }
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

    view! {
        <div class="message">
            <div class="message-header">
                <span class="sender">{message.sender}</span>
                <span class="timestamp">{formatted_time}</span>
            </div>
            <div class="message-text">
                {message.text}
            </div>
        </div>
    }
}
