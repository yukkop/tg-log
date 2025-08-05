use once_cell::sync::Lazy;
use std::env;

pub static API_ID: Lazy<i32> = Lazy::new(|| {
env::var("API_ID").expect("API_ID env var not set").trim().parse::<i32>().expect("API_ID must be a number")
});

pub static API_HASH: Lazy<String> = Lazy::new(|| {
env::var("API_HASH").expect("API_HASH env var not set").trim().to_string()
});

pub static TARGET_CHAT: Lazy<i64> = Lazy::new(|| {
env::var("TARGET_CHAT").expect("TARGET_CHAT env var not set").trim().parse::<i64>().expect("TARGET_CHAT must be a number")
});


