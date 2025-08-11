# Telegram Chat Setup Guide

This guide will help you set up the Telegram chat application with your own API credentials.

## Prerequisites

1. A Telegram account
2. Rust installed on your system
3. `cargo-leptos` installed: `cargo install cargo-leptos --locked`

## Setup Steps

### 1. Get Telegram API Credentials

1. Go to https://my.telegram.org/
2. Log in with your phone number
3. Go to "API development tools"
4. Create a new application and get your `api_id` and `api_hash`

### 2. Set Environment Variables

Create a `.env` file in the project root with:

```bash
TELEGRAM_API_ID=your_api_id_here
TELEGRAM_API_HASH=your_api_hash_here
TELEGRAM_TARGET_CHAT=your_chat_id_here
```

### 3. Get Your Chat ID

To find the chat ID you want to monitor:

1. First, authenticate with Telegram by running the CLI tool:
   ```bash
   # Create a simple CLI runner first
   cargo run --bin cli-auth
   ```

2. Or use the list mode to see available chats:
   ```bash
   # This will show all your chats with their IDs
   cargo run --bin cli-auth -- --list
   ```

### 4. First-time Authentication

Before running the web app, you need to authenticate with Telegram once:

1. Run the CLI authentication tool (you'll need to create this)
2. Enter your phone number when prompted
3. Enter the verification code sent to your Telegram app
4. If you have 2FA enabled, enter your password
5. This will create a session file that the web app can use

### 5. Run the Web Application

```bash
cargo leptos watch
```

The application will be available at `http://127.0.0.1:3000`

## Features

- **Real-time Chat Display**: Shows messages from your target Telegram chat
- **Scroll Up/Down**: Smooth scrolling with automatic scroll-to-bottom for new messages  
- **Load More**: Automatically loads older messages when scrolling to the top
- **Responsive Design**: Works on desktop and mobile devices
- **Session Management**: Persistent authentication using Telegram session files

## Project Structure

- `src/app.rs` - Main Leptos application component
- `src/chat.rs` - Chat interface and message display components
- `src/telegram.rs` - Telegram client integration and message handling
- `src/config.rs` - Configuration management for API credentials
- `style/main.scss` - CSS styling for the chat interface

## Troubleshooting

### "Not Authorized" Error
- Make sure you've run the CLI authentication tool first
- Check that your API credentials are correct
- Verify the session file exists and is readable

### Chat Not Loading
- Verify your `TELEGRAM_TARGET_CHAT` ID is correct
- Make sure you have access to the chat/channel
- Check that the Telegram client has proper permissions

### Build Errors
- Make sure all dependencies are installed: `cargo update`
- Verify you have the latest version of `cargo-leptos`
- Check that your Rust version supports the dependencies

## Security Notes

- Never commit your `.env` file to version control
- Keep your API credentials secure
- The session file contains authentication data - keep it safe
- Consider using environment variables in production instead of `.env` files
