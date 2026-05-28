// client/src/ui.rs
// Используем WebView2 для чата

use std::collections::VecDeque;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::webview2_bridge::WEBVIEW;

#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub is_system: bool,
}

pub struct ChatManager {
    messages: Mutex<VecDeque<ChatMessage>>,
    is_open: Mutex<bool>,
}

impl ChatManager {
    pub fn new() -> Self {
        // Инициализируем WebView2
        let mut webview = WEBVIEW.lock().unwrap();
        if webview.find_game_window() {
            webview.init();
            webview.set_visible(true);
        }
        
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            is_open: Mutex::new(false),
        }
    }
    
    pub fn add_message(&self, text: String, is_system: bool) {
        let mut messages = self.messages.lock().unwrap();
        messages.push_back(ChatMessage { text: text.clone(), is_system });
        
        while messages.len() > 50 {
            messages.pop_front();
        }
        
        // Отправляем в WebView2
        let webview = WEBVIEW.lock().unwrap();
        webview.add_message(&text, is_system);
        
        println!("[{}] {}", if is_system { "SYS" } else { "CHAT" }, text);
    }
    
    pub fn toggle_input(&self) {
        let mut is_open = self.is_open.lock().unwrap();
        *is_open = !*is_open;
        
        let webview = WEBVIEW.lock().unwrap();
        if *is_open {
            webview.show_input();
        } else {
            webview.hide_input();
        }
    }
    
    pub fn is_input_open(&self) -> bool {
        *self.is_open.lock().unwrap()
    }
    
    pub fn send_message(&self, msg: &str) {
        if !msg.is_empty() {
            self.add_message(format!("You: {}", msg), false);
            *self.is_open.lock().unwrap() = false;
            let webview = WEBVIEW.lock().unwrap();
            webview.hide_input();
        }
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
