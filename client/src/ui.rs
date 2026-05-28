// client/src/ui.rs
// RebornMP UI Manager - ImGui Working Version

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use std::cell::RefCell;

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
        
        println!("[{}] {}", if is_system { "SYS" } else { "CHAT" }, text);
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
    
    pub fn get_input_text(&self) -> String {
        self.input_buffer.lock().unwrap().clone()
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
    
    pub fn send_message(&self) -> Option<String> {
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
    
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}

// ImGui рендеринг - не храним в static!
thread_local! {
    static IMGUI_CONTEXT: RefCell<Option<imgui::Context>> = RefCell::new(None);
}

pub fn init_imgui() {
    IMGUI_CONTEXT.with(|ctx| {
        let mut ctx_borrow = ctx.borrow_mut();
        if ctx_borrow.is_none() {
            *ctx_borrow = Some(imgui::Context::create());
            println!("[UI] ImGui initialized");
        }
    });
}

pub fn render_imgui() {
    IMGUI_CONTEXT.with(|ctx| {
        if let Some(ref mut imgui_ctx) = *ctx.borrow_mut() {
            let ui = imgui_ctx.frame();
            
            // Окно чата
            imgui::Window::new("RebornMP Chat")
                .size([400.0, 300.0], imgui::Condition::FirstUseEver)
                .position([20.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
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
                
                imgui::Window::new("Chat Input")
                    .size([400.0, 60.0], imgui::Condition::FirstUseEver)
                    .position([20.0, 420.0], imgui::Condition::FirstUseEver)
                    .title_bar(false)
                    .build(&ui, || {
                        if ui.input_text("##input", &mut input).enter_returns_true(true).build() {
                            if let Some(msg) = CHAT.send_message() {
                                // Отправка сообщения
                                crate::hooks::send_chat_message(&msg);
                            }
                        }
                        
                        if ui.button("Send", [80.0, 25.0]) {
                            if let Some(msg) = CHAT.send_message() {
                                crate::hooks::send_chat_message(&msg);
                            }
                        }
                        
                        ui.same_line();
                        
                        if ui.button("Close", [80.0, 25.0]) {
                            CHAT.toggle_input();
                        }
                    });
            }
        }
    });
}
