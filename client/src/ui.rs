// client/src/ui.rs
// RebornMP UI Manager - ImGui Version (Working)

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use imgui::{Context, Window};

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
    
    pub fn render_imgui(&self, ui: &imgui::Ui) {
        // Окно чата
        if *self.is_open.lock().unwrap() {
            let mut input_text = self.get_input_text();
            
            Window::new(im_str("Chat Input"))
                .size([400.0, 100.0], imgui::Condition::FirstUseEver)
                .build(ui, || {
                    ui.input_text(im_str("##chat_input"), &mut input_text)
                        .build();
                    
                    if ui.button(im_str("Send"), [100.0, 30.0]) {
                        if !input_text.is_empty() {
                            let msg = input_text.clone();
                            *self.input_buffer.lock().unwrap() = msg.clone();
                            if let Some(msg) = self.send_message() {
                                if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                                    net.lock().unwrap().send_chat(&msg);
                                }
                            }
                        }
                    }
                    
                    ui.same_line();
                    
                    if ui.button(im_str("Close"), [100.0, 30.0]) {
                        self.toggle_input();
                    }
                });
            
            *self.input_buffer.lock().unwrap() = input_text;
        }
        
        // HUD сообщений
        Window::new(im_str("Chat History"))
            .size([400.0, 300.0], imgui::Condition::FirstUseEver)
            .position([20.0, 20.0], imgui::Condition::FirstUseEver)
            .build(ui, || {
                let messages = self.get_messages();
                let start = if messages.len() > 10 { messages.len() - 10 } else { 0 };
                
                for msg in &messages[start..] {
                    let color = if msg.is_system { [1.0, 0.8, 0.2, 1.0] } else { [0.2, 1.0, 0.2, 1.0] };
                    ui.text_colored(color, &msg.text);
                }
            });
    }
}

pub struct UIManager {
    imgui_context: Mutex<Option<Context>>,
    last_render: Mutex<u64>,
}

impl UIManager {
    pub fn new() -> Self {
        println!("[UI] UIManager initialized (ImGui Mode)");
        
        let mut ctx = Context::create();
        ctx.fonts().add_default_font();
        
        UIManager {
            imgui_context: Mutex::new(Some(ctx)),
            last_render: Mutex::new(0),
        }
    }
    
    pub fn render(&self) {
        if let Some(ctx) = self.imgui_context.lock().unwrap().as_mut() {
            let ui = ctx.frame();
            CHAT.render_imgui(&ui);
        }
    }
    
    pub fn update(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut last = self.last_render.lock().unwrap();
        if now - *last > 0 {
            *last = now;
            self.render();
        }
    }
    
    pub fn add_chat_message(&self, text: &str, is_system: bool) {
        CHAT.add_message(text.to_string(), is_system);
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
    pub static ref UI: UIManager = UIManager::new();
}

pub fn add_chat_message(text: String, is_system: bool) {
    CHAT.add_message(text, is_system);
}
