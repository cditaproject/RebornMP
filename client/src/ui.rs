// client/src/ui.rs
// RebornMP UI Manager - Full Version (Chat + HUD + DirectX)

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;

// ========== СТРУКТУРА СООБЩЕНИЯ ЧАТА ==========
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
            println!("[UI] Console mode enabled - chat will appear in console");
        }
    }
    
    fn render_console(&self) {
        print!("\r\x1B[2J\x1B[1;1H");
        println!("=== RebornMP Chat (Console Mode) ===");
        println!("Type messages below. Press Enter to send.");
        println!("----------------------------------------");
        
        let messages = self.get_messages();
        let start = if messages.len() > 10 { messages.len() - 10 } else { 0 };
        for msg in &messages[start..] {
            let time = msg.timestamp % 86400;
            let prefix = if msg.is_system { "🔧" } else { "💬" };
            println!("[{}] {} {}", time, prefix, msg.text);
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

// ========== ОСНОВНОЙ UI МЕНЕДЖЕР ==========
pub struct UIManager {
    show_hud: Mutex<bool>,
    show_debug: Mutex<bool>,
    last_render: Mutex<u64>,
}

impl UIManager {
    pub fn new() -> Self {
        println!("[UI] UIManager initialized");
        UIManager {
            show_hud: Mutex::new(true),
            show_debug: Mutex::new(false),
            last_render: Mutex::new(0),
        }
    }
    
    pub fn update(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut last = self.last_render.lock().unwrap();
        if now - *last > 1 {
            *last = now;
            CHAT.render();
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

// ========== ГЛОБАЛЬНЫЙ ЭКЗЕМПЛЯР ==========
lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}

pub fn add_chat_message(text: String, is_system: bool) {
    CHAT.add_message(text, is_system);
}
// Добавить в конец файла

use webview2::Controller;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct WebViewUI {
    controller: Option<Controller>,
    is_visible: bool,
}

impl WebViewUI {
    pub fn new(parent_hwnd: HWND) -> Result<Self, String> {
        let env = webview2::Environment::create(None, |env| Ok(())).unwrap();
        let controller = env.create_controller(parent_hwnd, None).unwrap();
        
        controller.set_visible(false).unwrap();
        
        let html = include_str!("../ui/index.html");
        controller.navigate_to_html(html).unwrap();
        
        Ok(WebViewUI {
            controller: Some(controller),
            is_visible: false,
        })
    }
    
    pub fn show_chat(&mut self) {
        if let Some(c) = &self.controller {
            let _ = c.set_visible(true);
            self.is_visible = true;
            // Выполнить JS для показа ввода
            let _ = c.execute_script("window.reborn.showInput()");
        }
    }
    
    pub fn hide_chat(&mut self) {
        if let Some(c) = &self.controller {
            let _ = c.set_visible(false);
            self.is_visible = false;
        }
    }
    
    pub fn add_message(&self, text: &str, is_system: bool) {
        if let Some(c) = &self.controller {
            let js = format!("window.reborn.addMessage('{}', {})", 
                text.replace("'", "\\'"), 
                if is_system { "true" } else { "false" }
            );
            let _ = c.execute_script(&js);
        }
    }
}
