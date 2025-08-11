use once_cell::sync::Lazy;

pub static API_ID: Lazy<i32> = Lazy::new(|| {
    std::env::var("TELEGRAM_API_ID")
        .expect("TELEGRAM_API_ID environment variable not set")
        .trim()
        .parse()
        .expect("TELEGRAM_API_ID must be a valid integer")
});

pub static API_HASH: Lazy<String> = Lazy::new(|| {
    std::env::var("TELEGRAM_API_HASH")
        .expect("TELEGRAM_API_HASH environment variable not set")
        .trim().to_string()
});

pub static TARGET_CHAT: Lazy<i64> = Lazy::new(|| {
    std::env::var("TELEGRAM_TARGET_CHAT")
        .expect("TELEGRAM_TARGET_CHAT environment variable not set")
        .trim()
        .parse()
        .expect("TELEGRAM_TARGET_CHAT must be a valid integer")
});
