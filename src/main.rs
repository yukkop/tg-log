use grammers_client::{Client, Config};
use grammers_client::SignInError;
use grammers_client::types::Update;
use grammers_session::Session;
use once_cell::sync::Lazy;
use std::env;

static PASSWORD: Lazy<Option<String>> = Lazy::new(|| {
if let Ok(password) = env::var("PASSWORD") {
    Some(password)
} else {
    None
}
});

static API_ID: Lazy<i32> = Lazy::new(|| {
env::var("API_ID").expect("API_ID env var not set").parse::<i32>().expect("API_ID must be a number")
});

static API_HASH: Lazy<String> = Lazy::new(|| {
env::var("API_HASH").expect("API_HASH env var not set")
});

static TARGET_CHAT: Lazy<i64> = Lazy::new(|| {
env::var("TARGET_CHAT").expect("TARGET_CHAT env var not set").parse::<i64>().expect("TARGET_CHAT must be a number")
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let list_mode = std::env::args().any(|arg| arg == "--list");

    let session = Session::load_file_or_create("session")?;
    let config = Config {
        session,
        api_id: *API_ID,
        api_hash: API_HASH.clone(),
        params: Default::default(),
    };

    let mut client = Client::connect(config).await?;

    if !client.is_authorized().await? {
        let token = client.request_login_code(&prompt("Phone: ")).await?;
        let code = prompt("Code: ");
        match client.sign_in(&token, &code).await {
            Err(SignInError::PasswordRequired(password_token)) => {
                let pwd = prompt("Password: ");
                client.check_password(password_token, &pwd).await?;
            }
            Err(other) => return Err(format!("Login failed: {other:?}").into()),
            Ok(_) => {}
        }
    }

    if list_mode {
        let mut dialogs = client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            let chat = dialog.chat();
            println!("{} ({:?})", chat.name(), chat.id());
        }
        return Ok(());
    }

    loop {
        let update = client.next_update().await?;
        if let Update::NewMessage(msg) = update {
            if msg.chat().id() == *TARGET_CHAT {
                println!("{}: {}", msg.chat().id(), msg.text());
            }
        }
    }
}

fn prompt(msg: &str) -> String {
    use std::io::{self, Write};
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}
