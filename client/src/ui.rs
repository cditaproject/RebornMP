// client/src/ui.rs
// RebornMP UI Manager

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use winapi::um::wingdi::RGB;
use crate::overlay::SimpleOverlay;

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
}

impl ChatManager {
    pub fn new() -> Self {
        let overlay = SimpleOverlay::new();
        overlay.init();
        
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
        }
    }
    
    pub fn add_message(&self, text: String, is_system: bool) {
        let mut messages = self.messages.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        messages.push_back(ChatMessage { text: text.clone(), timestamp, is_system });
        
        while messages.len() > 50 {
            messages.pop_front();
        }
        
        println!("[{}] {}", if is_system { "SYSTEM" } else { "CHAT" }, text);
        self.render();
    }
    
    pub fn toggle_input(&self) {
        let mut is_open = self.is_open.lock().unwrap();
        *is_open = !*is_open;
        if !*is_open {
            self.input_buffer.lock().unwrap().clear();
        }
        self.render();
    }
    
    pub fn is_input_open(&self) -> bool {
        *self.is_open.lock().unwrap()
    }
    
    pub fn get_input_text(&self) -> String {
        self.input_buffer.lock().unwrap().clone()
    }
    
    pub fn add_char(&self, c: char) {
        if *self.is_open.lock().unwrap() {
            self.input_buffer.lock().unwrap().push(c);
            self.render();
        }
    }
    
    pub fn backspace(&self) {
        if *self.is_open.lock().unwrap() {
            self.input_buffer.lock().unwrap().pop();
            self.render();
        }
    }
    
    pub fn send_message(&self) -> Option<String> {
        if *self.is_open.lock().unwrap() {
            let msg = self.input_buffer.lock().unwrap().clone();
            if !msg.is_empty() {
                self.input_buffer.lock().unwrap().clear();
                *self.is_open.lock().unwrap() = false;
                self.render();
                return Some(msg);
            }
            *self.is_open.lock().unwrap() = false;
            self.render();
        }
        None
    }
    
    pub fn render(&self) {
        let overlay = SimpleOverlay::new();
        
        // Очищаем область чата
        overlay.clear_area(10, 50, 400, 400);
        
        // Рисуем фон чата
        overlay.rect(10, 50, 400, 350, RGB(0, 0, 0));
        
        let messages = self.messages.lock().unwrap();
        let start = if messages.len() > 15 { messages.len() - 15 } else { 0 };
        
        let mut y = 70;
        for msg in messages.iter().skip(start) {
            let color = if msg.is_system {
                RGB(255, 200, 100)
            } else {
                RGB(100, 255, 100)
            };
            
            let prefix = if msg.is_system { "[SYS]" } else { "[CHAT]" };
            let text = format!("{} {}", prefix, msg.text);
            overlay.text(&text, 20, y, color);
            y += 22;
        }
        
        if *self.is_open.lock().unwrap() {
            let input = self.get_input_text();
            overlay.rect(10, y + 10, 400, 35, RGB(40, 40, 40));
            overlay.text(&format!("> {}", input), 20, y + 20, RGB(255, 255, 255));
        }
    }
    
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
