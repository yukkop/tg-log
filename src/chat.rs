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
    let chat_state = RwSignal::new(ChatState::default());
    let scroll_container_ref = NodeRef::<html::Div>::new();
    let is_auto_scroll = RwSignal::new(true);
    
    // Load initial messages
    let load_messages = Action::new(move |chat_id: &i64| {
        let chat_id = *chat_id;
        async move {
            #[cfg(feature = "ssr")]
            {
                use crate::telegram::{create_telegram_client, get_chat_history};
                match create_telegram_client().await {
                    Ok(client) => {
                        match get_chat_history(&client, chat_id, 50).await {
                            Ok(messages) => {
                                chat_state.update(|state| {
                                    state.messages = messages;
                                    state.loading = false;
                                    state.error = None;
                                    state.chat_id = Some(chat_id);
                                });
                            }
                            Err(e) => {
                                chat_state.update(|state| {
                                    state.loading = false;
                                    state.error = Some(format!("Failed to load messages: {}", e));
                                });
                            }
                        }
                    }
                    Err(e) => {
                        chat_state.update(|state| {
                            state.loading = false;
                            state.error = Some(format!("Failed to connect to Telegram: {}", e));
                        });
                    }
                }
            }
            #[cfg(not(feature = "ssr"))]
            {
                // Client-side placeholder - in a real app you'd make an API call
                chat_state.update(|state| {
                    state.messages = vec![
                        ChatMessage {
                            id: 1,
                            text: "Welcome to the chat! (Demo mode - not connected to Telegram)".to_string(),
                            timestamp: chrono::Utc::now().timestamp(),
                            sender: "System".to_string(),
                            chat_id,
                        }
                    ];
                    state.loading = false;
                    state.chat_id = Some(chat_id);
                });
            }
        }
    });

    // Load more messages when scrolling up
    let load_more_messages = Action::new(move |_: &()| {
        async move {
            let state = chat_state.get();
            if let Some(_chat_id) = state.chat_id {
                if !state.loading && state.has_more_history {
                    chat_state.update(|s| s.loading = true);
                    
                    #[cfg(feature = "ssr")]
                    {
                        // In a real implementation, you'd load messages before the earliest message
                        // For now, we'll just simulate no more history
                        chat_state.update(|state| {
                            state.loading = false;
                            state.has_more_history = false;
                        });
                    }
                    #[cfg(not(feature = "ssr"))]
                    {
                        // Simulate loading more messages
                        chat_state.update(|state| {
                            state.loading = false;
                            state.has_more_history = false;
                        });
                    }
                }
            }
        }
    });

    // Auto-scroll to bottom when new messages arrive
    Effect::new(move |_| {
        let messages_count = chat_state.get().messages.len();
        if is_auto_scroll.get() && messages_count > 0 {
            if let Some(container) = scroll_container_ref.get() {
                container.set_scroll_top(container.scroll_height());
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
            
            // Check if scrolled to top and load more messages
            if scroll_top <= 10 && chat_state.get().has_more_history {
                load_more_messages.dispatch(());
            }
        }
    };

    // Initialize with a default chat ID (you can modify this)
    Effect::new(move |_| {
        #[cfg(feature = "ssr")]
        {
            use crate::config::TARGET_CHAT;
            load_messages.dispatch(*TARGET_CHAT);
        }
        #[cfg(not(feature = "ssr"))]
        {
            // Placeholder chat ID for client-side
            load_messages.dispatch(-1001234567890);
        }
    });

    view! {
        <div class="chat-container">
            <div class="chat-header">
                <h2>"Telegram Chat"</h2>
                {move || {
                    let state = chat_state.get();
                    if let Some(chat_id) = state.chat_id {
                        view! { <span class="chat-id">"Chat ID: " {chat_id}</span> }.into_any()
                    } else {
                        view! { <span class="chat-id">"No chat loaded"</span> }.into_any()
                    }
                }}
            </div>
            
            <div 
                class="messages-container"
                node_ref=scroll_container_ref
                on:scroll=on_scroll
            >
                {move || {
                    let state = chat_state.get();
                    
                    if state.loading && state.messages.is_empty() {
                        view! {
                            <div class="loading-indicator">
                                "Loading messages..."
                            </div>
                        }.into_any()
                    } else if let Some(error) = &state.error {
                        view! {
                            <div class="error-message">
                                {error.clone()}
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <>
                                {if state.loading {
                                    view! {
                                        <div class="loading-more">
                                            "Loading more messages..."
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                                
                                {state.messages.into_iter().map(|message| {
                                    view! {
                                        <MessageComponent message=message />
                                    }
                                }).collect::<Vec<_>>()}
                            </>
                        }.into_any()
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
