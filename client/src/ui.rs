// client/src/ui.rs
// RebornMP UI Manager - ImGui Chat (Full Working Version)

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::cell::RefCell;
use lazy_static::lazy_static;
use imgui::Context;

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
        
        messages.push_back(ChatMessage { text: text.clone(), timestamp, is_system });
        
        while messages.len() > self.max_messages {
            messages.pop_front();
        }
        
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
            self.toggle_input();
        }
        None
    }
    
    pub fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
    
    pub fn set_console_mode(&self, _enabled: bool) {
        // Для совместимости
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}

// ========== IMGUI UI МЕНЕДЖЕР ==========
pub struct UIManager {
    imgui_ctx: RefCell<Option<Context>>,
}

impl UIManager {
    pub fn new() -> Self {
        UIManager {
            imgui_ctx: RefCell::new(None),
        }
    }
    
    pub fn init_imgui(&self) {
        let mut ctx = self.imgui_ctx.borrow_mut();
        if ctx.is_none() {
            *ctx = Some(Context::create());
            
            // Настройка стиля
            if let Some(ref mut imgui_ctx) = *ctx {
                let style = imgui_ctx.style_mut();
                style.window_rounding = 5.0;
                style.window_padding = [10.0, 10.0];
                style.colors[imgui::StyleColor::WindowBg] = [0.0, 0.0, 0.0, 0.8];
                style.colors[imgui::StyleColor::Text] = [1.0, 1.0, 1.0, 1.0];
            }
            
            println!("[UI] ImGui initialized");
        }
    }
    
    pub fn render(&self) {
        let mut ctx = self.imgui_ctx.borrow_mut();
        if let Some(ref mut imgui_ctx) = *ctx {
            let ui = imgui_ctx.frame();
            
            // Окно чата
            let window = imgui::Window::new("RebornMP Chat")
                .size([400.0, 300.0], imgui::Condition::FirstUseEver)
                .position([20.0, 100.0], imgui::Condition::FirstUseEver);
            window.build(&ui, || {
                let messages = CHAT.get_messages();
                let start = if messages.len() > 15 { messages.len() - 15 } else { 0 };
                
                for msg in messages.iter().skip(start) {
                    let color = if msg.is_system {
                        [1.0, 0.8, 0.2, 1.0]
                    } else {
                        [0.2, 1.0, 0.2, 1.0]
                    };
                    ui.text_colored(color, &msg.text);
                }
            });
            
            // Поле ввода
            if CHAT.is_input_open() {
                let mut input = CHAT.get_input_text();
                
                let input_window = imgui::Window::new("Chat Input")
                    .size([400.0, 60.0], imgui::Condition::FirstUseEver)
                    .position([20.0, 420.0], imgui::Condition::FirstUseEver)
                    .title_bar(false);
                
                input_window.build(&ui, || {
                    if ui.input_text("##input", &mut input)
                        .enter_returns_true(true)
                        .build() {
                        if let Some(msg) = CHAT.send_message() {
                            if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                                net.lock().unwrap().send_chat(&msg);
                            }
                        }
                    }
                    
                    if ui.button("Send", [80.0, 25.0]) {
                        if let Some(msg) = CHAT.send_message() {
                            if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                                net.lock().unwrap().send_chat(&msg);
                            }
                        }
                    }
                    
                    ui.same_line();
                    
                    if ui.button("Close", [80.0, 25.0]) {
                        CHAT.toggle_input();
                    }
                });
            }
        }
    }
}

lazy_static! {
    pub static ref UI: UIManager = UIManager::new();
}

pub fn add_chat_message(text: String, is_system: bool) {
    CHAT.add_message(text, is_system);
}
