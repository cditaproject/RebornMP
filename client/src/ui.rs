// client/src/ui.rs
// RebornMP UI Manager - With WebView2

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use webview2::Controller;
use windows::Win32::Foundation::HWND;

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
    webview: Mutex<Option<Controller>>,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            console_mode: Mutex::new(true),
            max_messages: 50,
            webview: Mutex::new(None),
        }
    }
    
    pub fn init_webview(&self, parent_hwnd: isize) -> Result<(), String> {
        let hwnd = HWND(parent_hwnd as *mut _);
        
        // Создаём среду WebView2
        let env = webview2::Environment::create(None, |_env| Ok(()))
            .map_err(|e| format!("Failed to create env: {:?}", e))?;
        
        // Создаём контроллер
        let controller = env.create_controller(hwnd, None)
            .map_err(|e| format!("Failed to create controller: {:?}", e))?;
        
        // Простой HTML для чата
        let html = r#"
        <html><body style='background:transparent; color:white; font-family:Arial; margin:0; padding:0;'>
        <div id='chat' style='position:fixed; bottom:20px; left:20px; width:400px; max-height:300px; overflow-y:auto;'></div>
        <script>
            window.addMessage = function(text, isSystem) {
                let div = document.getElementById('chat');
                let msg = document.createElement('div');
                msg.style.background = 'rgba(0,0,0,0.7)';
                msg.style.padding = '5px';
                msg.style.margin = '2px';
                msg.style.borderRadius = '5px';
                msg.style.color = isSystem ? '#ffaa00' : '#00ff00';
                msg.innerText = text;
                div.appendChild(msg);
                div.scrollTop = div.scrollHeight;
            };
        </script>
        </body></html>
        "#;
        
        controller.navigate_to_html(html)
            .map_err(|e| format!("Failed to load HTML: {:?}", e))?;
        
        *self.webview.lock().unwrap() = Some(controller);
        println!("[UI] WebView2 initialized!");
        Ok(())
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
        
        // Отправляем в WebView если есть
        if let Some(webview) = self.webview.lock().unwrap().as_ref() {
            let js = format!("window.addMessage('{}', {})", 
                text.replace("'", "\\'"),
                if is_system { "true" } else { "false" }
            );
            let _ = webview.execute_script(&js);
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
    }
    
    pub fn render(&self) {
        if *self.console_mode.lock().unwrap() {
            print!("\r\x1B[2J\x1B[1;1H");
            println!("=== RebornMP Chat ===");
            println!("----------------------------------------");
            
            let messages = self.get_messages();
            let start = if messages.len() > 15 { messages.len() - 15 } else { 0 };
            for msg in &messages[start..] {
                let prefix = if msg.is_system { "🔧" } else { "💬" };
                println!("{} {}", prefix, msg.text);
            }
            
            if self.is_input_open() {
                println!("\n> {}", self.get_input_text());
            }
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
