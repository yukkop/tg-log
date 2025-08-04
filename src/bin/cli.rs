use grammers_client::{Client, Config};
use grammers_client::SignInError;
use grammers_client::types::Update;
use grammers_session::Session;
use tglog::*;

fn prompt(msg: &str) -> String {
    use std::io::{self, Write};
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}
