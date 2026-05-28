// client/src/ui.rs
// RebornMP UI Manager - Full Version

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;

#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub timestamp: u64,
    pub is_system: bool,
}

pub struct ChatManager {
    messages: Mutex<VecDeque<ChatMessage>>,
    input_buffer: Mutex<String>,
    is_open: Mutex<bool>,
    console_mode: Mutex<bool>,
    max_messages: usize,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            console_mode: Mutex::new(false),
            max_messages: 50,
        }
    }
    
    pub fn add_message(&self, text: String, is_system: bool) {
        let mut messages = self.messages.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        messages.push_back(ChatMessage { text: text.clone(), timestamp, is_system });
        
        while messages.len() > self.max_messages {
            messages.pop_front();
        }
        
        let prefix = if is_system { "[SYSTEM]" } else { "[CHAT]" };
        println!("{} {}", prefix, text);
        
        if *self.console_mode.lock().unwrap() {
            self.render_console();
        }
    }
    
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }
    
    pub fn toggle_input(&self) {
        let mut is_open = self.is_open.lock().unwrap();
        *is_open = !*is_open;
        if !*is_open {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.clear();
        }
    }
    
    pub fn is_input_open(&self) -> bool {
        *self.is_open.lock().unwrap()
    }
    
    pub fn get_input_text(&self) -> String {
        self.input_buffer.lock().unwrap().clone()
    }
    
    pub fn add_char(&self, c: char) {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.push(c);
        }
    }
    
    pub fn backspace(&self) {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.pop();
        }
    }
    
    pub fn send_message(&self) -> Option<String> {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            let message = buffer.clone();
            if !message.is_empty() {
                buffer.clear();
                self.toggle_input();
                return Some(message);
            }
            self.toggle_input();
        }
        None
    }
    
    pub fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
    
    pub fn set_console_mode(&self, enabled: bool) {
        *self.console_mode.lock().unwrap() = enabled;
        if enabled {
            println!("[UI] Console mode enabled");
        }
    }
    
    fn render_console(&self) {
        print!("\r\x1B[2J\x1B[1;1H");
        println!("=== RebornMP Chat (Console Mode) ===");
        println!("----------------------------------------");
        
        let messages = self.get_messages();
        let start = if messages.len() > 10 { messages.len() - 10 } else { 0 };
        for msg in &messages[start..] {
            let prefix = if msg.is_system { "🔧" } else { "💬" };
            println!("{} {}", prefix, msg.text);
        }
        
        if self.is_input_open() {
            println!("\n> {}", self.get_input_text());
        }
    }
    
    pub fn render(&self) {
        if *self.console_mode.lock().unwrap() {
            self.render_console();
        }
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}

pub fn add_chat_message(text: String, is_system: bool) {
    CHAT.add_message(text, is_system);
}
