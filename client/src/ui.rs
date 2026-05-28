// client/src/ui.rs
// RebornMP Chat - Like YimMenuV2

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use imgui::{Context, Window, ImString, Condition};

#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub timestamp: u64,
    pub is_system: bool,
    pub sender: String,
}

pub struct ChatManager {
    messages: Mutex<VecDeque<ChatMessage>>,
    input_buffer: Mutex<String>,
    is_open: Mutex<bool>,
    imgui_ctx: Mutex<Option<Context>>,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            imgui_ctx: Mutex::new(None),
        }
    }
    
    pub fn init_imgui(&self) {
        let mut ctx = Context::create();
        
        // Настройка стиля как в YimMenu
        let style = ctx.style_mut();
        style.window_rounding = 5.0;
        style.window_padding = [10.0, 10.0];
        style.colors[imgui::StyleColor::WindowBg] = [0.0, 0.0, 0.0, 0.8];
        style.colors[imgui::StyleColor::Text] = [1.0, 1.0, 1.0, 1.0];
        
        *self.imgui_ctx.lock().unwrap() = Some(ctx);
        println!("[UI] ImGui initialized (YimMenu style)");
    }
    
    pub fn add_message(&self, text: String, is_system: bool, sender: String) {
        let mut messages = self.messages.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        messages.push_back(ChatMessage { text, timestamp, is_system, sender });
        
        while messages.len() > 50 {
            messages.pop_front();
        }
        
        println!("[CHAT] {}", messages.back().unwrap().text);
    }
    
    pub fn toggle_input(&self) {
        let mut is_open = self.is_open.lock().unwrap();
        *is_open = !*is_open;
        if !*is_open {
            self.input_buffer.lock().unwrap().clear();
        }
    }
    
    pub fn is_input_open(&self) -> bool {
        *self.is_open.lock().unwrap()
    }
    
    pub fn add_char(&self, c: char) {
        if *self.is_open.lock().unwrap() {
            self.input_buffer.lock().unwrap().push(c);
        }
    }
    
    pub fn backspace(&self) {
        if *self.is_open.lock().unwrap() {
            self.input_buffer.lock().unwrap().pop();
        }
    }
    
    pub fn send_current_message(&self) -> Option<String> {
        if *self.is_open.lock().unwrap() {
            let msg = self.input_buffer.lock().unwrap().clone();
            if !msg.is_empty() {
                self.input_buffer.lock().unwrap().clear();
                *self.is_open.lock().unwrap() = false;
                return Some(msg);
            }
            *self.is_open.lock().unwrap() = false;
        }
        None
    }
    
    pub fn render(&self, ui: &imgui::Ui) {
        // Окно чата (как в YimMenu)
        let messages = self.messages.lock().unwrap();
        let start = if messages.len() > 15 { messages.len() - 15 } else { 0 };
        
        Window::new(ImString::new("RebornMP Chat"))
            .size([400.0, 300.0], Condition::FirstUseEver)
            .position([20.0, 100.0], Condition::FirstUseEver)
            .bg_alpha(0.8)
            .build(ui, || {
                // История сообщений
                for msg in messages.iter().skip(start) {
                    let color = if msg.is_system {
                        [1.0, 0.8, 0.2, 1.0]  // Жёлтый для системы
                    } else {
                        [0.2, 1.0, 0.2, 1.0]  // Зелёный для игроков
                    };
                    ui.text_colored(color, &msg.text);
                }
            });
        
        // Поле ввода (когда открыто)
        if *self.is_open.lock().unwrap() {
            let mut input = self.input_buffer.lock().unwrap().clone();
            
            Window::new(ImString::new("Chat Input"))
                .size([400.0, 60.0], Condition::FirstUseEver)
                .position([20.0, 420.0], Condition::FirstUseEver)
                .title_bar(false)
                .build(ui, || {
                    ui.input_text(ImString::new("##chat_input"), &mut input)
                        .enter_returns_true(true)
                        .build();
                    
                    if ui.button(ImString::new("Send"), [80.0, 25.0]) {
                        if let Some(msg) = self.send_current_message() {
                            crate::hooks::send_chat_message(&msg);
                        }
                    }
                    
                    ui.same_line();
                    
                    if ui.button(ImString::new("Close"), [80.0, 25.0]) {
                        *self.is_open.lock().unwrap() = false;
                    }
                    
                    *self.input_buffer.lock().unwrap() = input;
                });
        }
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
