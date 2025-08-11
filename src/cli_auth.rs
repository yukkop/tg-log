use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use std::io::{self, Write};
use tg_log_new::config::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let list_mode = std::env::args().any(|arg| arg == "--list");

    println!("Loading session...");
    let session = Session::load_file_or_create("session")?;
    println!("Session loaded successfully");
    
    if std::path::Path::new("session").exists() {
        let metadata = std::fs::metadata("session")?;
        println!("Session file exists: {} bytes", metadata.len());
    } else {
        println!("No existing session file found, will create new one");
    }

    let config = Config {
        session,
        api_id: *API_ID,
        api_hash: API_HASH.clone(),
        params: Default::default(),
    };

    println!("Connecting to Telegram...");
    let client = Client::connect(config).await?;
    println!("Connected successfully");

    println!("Checking authorization status...");
    let is_authorized = client.is_authorized().await?;
    println!("Is authorized: {}", is_authorized);

    if !is_authorized {
        println!("Session invalid or expired, requesting new login...");
        let token = client.request_login_code(&prompt("Phone: ")).await?;
        let code = prompt("Code: ");
        match client.sign_in(&token, &code).await {
            Err(SignInError::PasswordRequired(password_token)) => {
                let pwd = secure_prompt("Password: ");
                client.check_password(password_token, &pwd).await?;
                println!("Successfully signed in with password!");
            }
            Err(other) => return Err(format!("Login failed: {other:?}").into()),
            Ok(_) => {
                println!("Successfully signed in!");
            }
        }
        
        println!("Saving session...");
        match client.session().save_to_file("session") {
            Ok(_) => println!("Session saved successfully!"),
            Err(e) => {
                eprintln!("Warning: Failed to save session: {}", e);
                eprintln!("You may need to authenticate again next time.");
            }
        }
        
        println!("Verifying session was saved...");
        let is_now_authorized = client.is_authorized().await?;
        println!("Authorization status after save: {}", is_now_authorized);
        
    } else {
        println!("Using existing session - no login required!");
    }

    if list_mode {
        println!("\nAvailable chats:");
        println!("================");
        let mut dialogs = client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            let chat = dialog.chat();
            println!("{} (ID: {:?})", chat.name(), chat.id());
        }
        return Ok(());
    }

    println!("\nAuthentication complete! You can now run the web application with:");
    println!("cargo leptos watch");

    Ok(())
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}

fn secure_prompt(msg: &str) -> String {
    use std::process::Command;
    
    print!("{}", msg);
    io::stdout().flush().unwrap();
    
    // Disable echo using stty command (works on Unix)
    #[cfg(unix)]
    {
        let _ = Command::new("stty")
            .args(["-echo"])
            .status();
        
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        
        let _ = Command::new("stty")
            .args(["echo"])
            .status();
        
        println!();
        password.trim().to_string()
    }
    
    #[cfg(not(unix))]
    {
        // Fallback for non-Unix systems
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        password.trim().to_string()
    }
}
