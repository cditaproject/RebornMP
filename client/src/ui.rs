// client/src/ui.rs
// Реальная отрисовка чата поверх GTA V

use std::sync::Mutex;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

pub struct UIManager {
    chat_messages: Mutex<Vec<ChatMessage>>,
    input_active: Mutex<bool>,
    input_text: Mutex<String>,
    show_hud: Mutex<bool>,
}

struct ChatMessage {
    text: String,
    timestamp: f64,
    is_system: bool,
}

impl UIManager {
    pub fn new() -> Self {
        UIManager {
            chat_messages: Mutex::new(Vec::new()),
            input_active: Mutex::new(false),
            input_text: Mutex::new(String::new()),
            show_hud: Mutex::new(true),
        }
    }
    
    pub fn add_chat_message(&self, message: &str, is_system: bool) {
        let mut chat = self.chat_messages.lock().unwrap();
        chat.push(ChatMessage {
            text: message.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            is_system,
        });
        
        // Оставляем только последние 50 сообщений
        if chat.len() > 50 {
            chat.remove(0);
        }
        
        println!("[CHAT] {}", message); // Для отладки в консоль
    }
    
    pub fn render(&self) {
        unsafe {
            // Здесь будет отрисовка через ImGui или DirectX
            // Пока используем MessageBox для теста
            let mut chat = self.chat_messages.lock().unwrap();
            if let Some(last) = chat.last() {
                if last.timestamp > std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64() - 5.0 {
                    // Показываем последнее сообщение
                    // В реальности здесь будет отрисовка через DirectX
                }
            }
        }
    }
}
