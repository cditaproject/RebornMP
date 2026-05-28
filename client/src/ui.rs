// client/src/ui.rs
// RebornMP UI Manager - Win32 GDI Version (Fully Working)

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ptr;
use lazy_static::lazy_static;
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;

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
    hwnd: Mutex<Option<HWND>>,
    hdc: Mutex<Option<HDC>>,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            max_messages: 50,
            hwnd: Mutex::new(None),
            hdc: Mutex::new(None),
        }
    }
    
    pub fn find_game_window(&self) {
        unsafe {
            // Ищем окно GTA V
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if !hwnd.is_null() {
                *self.hwnd.lock().unwrap() = Some(hwnd);
                let hdc = GetDC(hwnd);
                *self.hdc.lock().unwrap() = Some(hdc);
                println!("[UI] Found GTA V window!");
            } else {
                println!("[UI] GTA V window not found, using console mode");
            }
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
        
        // Рендерим на экране
        self.render_on_screen();
    }
    
    pub fn render_on_screen(&self) {
        unsafe {
            if let Some(hwnd) = *self.hwnd.lock().unwrap() {
                if let Some(hdc) = *self.hdc.lock().unwrap() {
                    // Создаём шрифт
                    let font = CreateFontA(
                        16, 0, 0, 0, FW_NORMAL, 0, 0, 0,
                        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                        DEFAULT_QUALITY, DEFAULT_PITCH, b"Arial\0".as_ptr() as *const i8
                    );
                    
                    let old_font = SelectObject(hdc, font as HGDIOBJ);
                    
                    // Белый цвет текста
                    SetTextColor(hdc, RGB(255, 255, 255));
                    SetBkMode(hdc, TRANSPARENT);
                    
                    let messages = self.get_messages();
                    let start = if messages.len() > 10 { messages.len() - 10 } else { 0 };
                    
                    let mut y = 50;
                    for msg in &messages[start..] {
                        let color = if msg.is_system { RGB(255, 200, 50) } else { RGB(100, 255, 100) };
                        SetTextColor(hdc, color);
                        
                        let text = format!("{} {}", 
                            if msg.is_system { "[SYS]" } else { "[CHAT]" },
                            msg.text
                        );
                        
                        TextOutA(hdc, 10, y, text.as_ptr() as *const i8, text.len() as i32);
                        y += 20;
                    }
                    
                    // Если чат открыт - показываем поле ввода
                    if *self.is_open.lock().unwrap() {
                        let input = self.get_input_text();
                        let prompt = format!("> {}", input);
                        SetTextColor(hdc, RGB(200, 200, 200));
                        TextOutA(hdc, 10, y + 10, prompt.as_ptr() as *const i8, prompt.len() as i32);
                    }
                    
                    SelectObject(hdc, old_font);
                    DeleteObject(font as HGDIOBJ);
                }
            }
        }
        
        // Дублируем в консоль
        self.render_console();
    }
    
    fn render_console(&self) {
        print!("\x1B[2J\x1B[1;1H");
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    REBORNMP CHAT                         ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        
        let messages = self.get_messages();
        let start = if messages.len() > 12 { messages.len() - 12 } else { 0 };
        
        for msg in &messages[start..] {
            let prefix = if msg.is_system { "🔧" } else { "💬" };
            let text = if msg.text.len() > 45 {
                format!("{}...", &msg.text[..42])
            } else {
                msg.text.clone()
            };
            println!("║ {} {:<50}║", prefix, text);
        }
        
        let empty = 12 - (messages.len().min(12));
        for _ in 0..empty {
            println!("║ {:<52}║", "");
        }
        
        println!("╠══════════════════════════════════════════════════════════╣");
        if *self.is_open.lock().unwrap() {
            let input = self.get_input_text();
            println!("║ > {:<50}║", input);
            println!("║ Press ENTER to send, ESC to cancel                  ║");
        } else {
            println!("║ Press T to open chat                                 ║");
        }
        println!("╚══════════════════════════════════════════════════════════╝");
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
        self.render_on_screen();
        self.render_console();
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
            self.render_on_screen();
            self.render_console();
        }
    }
    
    pub fn backspace(&self) {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.pop();
            self.render_on_screen();
            self.render_console();
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
        self.render_on_screen();
        self.render_console();
    }
    
    pub fn set_console_mode(&self, _enabled: bool) {
        self.render_on_screen();
        self.render_console();
    }
    
    pub fn render(&self) {
        self.render_on_screen();
        self.render_console();
    }
}

pub struct UIManager {
    last_render: Mutex<u64>,
}

impl UIManager {
    pub fn new() -> Self {
        println!("[UI] UIManager initialized (Win32 GDI Mode)");
        let ui = UIManager {
            last_render: Mutex::new(0),
        };
        
        // Пытаемся найти окно игры
        CHAT.find_game_window();
        
        ui
    }
    
    pub fn update(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut last = self.last_render.lock().unwrap();
        if now - *last > 0 {
            *last = now;
            CHAT.render();
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
