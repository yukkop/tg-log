use axum::{routing::get, Router};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::env;
use tokio::sync::broadcast;
use tglog_core::start_telegram_logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_id = env::var("API_ID")?.parse()?;
    let api_hash = env::var("API_HASH")?;
    let chat_id = env::var("TARGET_CHAT")?.parse()?;

    let (tx, _) = broadcast::channel(128);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let _ = start_telegram_logger(api_id, api_hash, chat_id, tx_clone).await;
    });

    let app = Router::new()
        .route("/api/logs", get(move || {
            let mut rx = tx.subscribe();
            async move {
                let mut lines = vec![];
                while let Ok(msg) = rx.recv().await {
                    lines.push(msg);
                    if lines.len() > 20 {
                        break;
                    }
                }
                axum::Json(lines)
            }
        }))
        .leptos_routes(generate_route_list(app), crate::App)
        .with_state(());

    println!("serving at http://localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
