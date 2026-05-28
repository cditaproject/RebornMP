// client/src/ui.rs
// RebornMP UI Manager - Full Version (Chat + HUD)

use std::sync::Mutex;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

// ========== СТРУКТУРЫ СООБЩЕНИЙ ==========
#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub timestamp: u64,
    pub is_system: bool,
}

// ========== МЕНЕДЖЕР ЧАТА ==========
pub struct ChatManager {
    messages: Mutex<VecDeque<ChatMessage>>,
    input_buffer: Mutex<String>,
    is_open: Mutex<bool>,
    max_messages: usize,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            max_messages: 50,
        }
    }
    
    pub fn add_message(&self, text: String, is_system: bool) {
        let mut messages = self.messages.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        messages.push_back(ChatMessage { text, timestamp, is_system });
        
        while messages.len() > self.max_messages {
            messages.pop_front();
        }
        
        // Для отладки выводим в консоль
        let prefix = if is_system { "[SYSTEM]" } else { "[CHAT]" };
        println!("{} {}", prefix, text);
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
        }
        None
    }
    
    pub fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
}

// ========== ОСНОВНОЙ UI МЕНЕДЖЕР ==========
pub struct UIManager {
    show_hud: Mutex<bool>,
    show_debug: Mutex<bool>,
}

impl UIManager {
    pub fn new() -> Self {
        println!("[UI] UIManager initialized");
        UIManager {
            show_hud: Mutex::new(true),
            show_debug: Mutex::new(false),
        }
    }
    
    pub fn update(&self) {
        // Рендерим HUD (пока только в консоль для отладки)
        if *self.show_hud.lock().unwrap() {
            // Здесь будет отрисовка через DirectX/ImGui
            // Пока просто выводим состояние чата
            let chat = CHAT;
            if chat.is_input_open() {
                print!("\r[CHAT] > {:<50}", chat.get_input_text());
            }
        }
    }
    
    pub fn show_hud(&self, visible: bool) {
        let mut hud = self.show_hud.lock().unwrap();
        *hud = visible;
    }
    
    pub fn toggle_debug(&self) {
        let mut debug = self.show_debug.lock().unwrap();
        *debug = !*debug;
        let status = if *debug { "on" } else { "off" };
        CHAT.add_message(format!("🔧 Debug mode: {}", status), true);
    }
    
    pub fn add_chat_message(&self, text: &str, is_system: bool) {
        CHAT.add_message(text.to_string(), is_system);
    }
}

// ========== ГЛОБАЛЬНЫЙ ЭКЗЕМПЛЯР ЧАТА ==========
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
