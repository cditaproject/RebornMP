use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tungstenite::{connect, Message};
use url::Url;
use serde_json::json;
use std::sync::LazyLock;
use std::sync::Mutex;

static SERVER_ADDR: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("ws://127.0.0.1:8080/ws".to_string()));
static CONNECTED: AtomicBool = AtomicBool::new(false);

pub fn set_server_addr(addr: String) {
    let ws_addr = if addr.starts_with("ws://") {
        addr.clone()
    } else if addr.starts_with("http://") {
        addr.replace("http://", "ws://") + "/ws"
    } else {
        format!("ws://{}/ws", addr)
    };
    *SERVER_ADDR.lock().unwrap() = ws_addr.clone();
    println!("[Network] Server address set to: {}", ws_addr);
}

pub fn start_client() {
    println!("[Network] Starting client...");
    
    thread::spawn(|| {
        connect_to_server();
    });
}

fn connect_to_server() {
    let addr = SERVER_ADDR.lock().unwrap().clone();
    println!("[Network] Connecting to {}", addr);
    
    let url = match Url::parse(&addr) {
        Ok(url) => url,
        Err(e) => {
            println!("[Network] Invalid URL: {}", e);
            return;
        }
    };
    
    match connect(url) {
        Ok((mut ws_stream, _)) => {
            println!("[Network] Successfully connected to server!");
            CONNECTED.store(true, Ordering::Relaxed);
            
            // Отправляем приветственное сообщение
            let welcome_msg = json!({
                "type": "connect",
                "client": "freemode-mp",
                "version": "1.0.0"
            }).to_string();
            
            if let Err(e) = ws_stream.send(Message::Text(welcome_msg)) {
                println!("[Network] Failed to send welcome: {}", e);
                return;
            }
            
            // Приём сообщений
            loop {
                match ws_stream.read() {
                    Ok(Message::Text(text)) => {
                        println!("[Network] Received: {}", text);
                        handle_message(&text);
                    }
                    Ok(Message::Close(_)) => {
                        println!("[Network] Connection closed by server");
                        break;
                    }
                    Err(e) => {
                        println!("[Network] Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            println!("[Network] Failed to connect: {}", e);
        }
    }
    CONNECTED.store(false, Ordering::Relaxed);
}

fn handle_message(text: &str) {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
            match msg_type {
                "welcome" => {
                    println!("[Network] Welcome from server!");
                }
                _ => {
                    println!("[Network] Server message: {}", text);
                }
            }
        }
    }
}

pub fn send_chat(text: String) -> bool {
    if !CONNECTED.load(Ordering::Relaxed) {
        println!("[Network] Not connected to server");
        return false;
    }
    
    println!("[Network] Sending chat: {}", text);
    true
}

pub fn update() {}

pub fn shutdown() {
    println!("[Network] Shutting down...");
    CONNECTED.store(false, Ordering::Relaxed);
}