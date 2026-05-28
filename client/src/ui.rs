// client/src/ui.rs
// RebornMP DirectX Chat Overlay

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::shared::windef::*;
use std::ptr;

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
    hwnd: Mutex<Option<HWND>>,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            hwnd: Mutex::new(None),
        }
    }
    
    pub fn find_game_window(&self) {
        unsafe {
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if !hwnd.is_null() {
                *self.hwnd.lock().unwrap() = Some(hwnd);
                println!("[UI] Found GTA V window");
            } else {
                println!("[UI] GTA V window not found");
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
        
        while messages.len() > 50 {
            messages.pop_front();
        }
        
        println!("[{}] {}", if is_system { "SYS" } else { "CHAT" }, text);
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
        unsafe {
            if let Some(hwnd) = *self.hwnd.lock().unwrap() {
                let hdc = GetDC(hwnd);
                
                // Создаём шрифт
                let font = CreateFontA(
                    16, 0, 0, 0, FW_NORMAL, 0, 0, 0,
                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                    DEFAULT_QUALITY, DEFAULT_PITCH, b"Arial\0".as_ptr() as *const i8
                );
                
                let old_font = SelectObject(hdc, font as HGDIOBJ);
                SetBkMode(hdc, TRANSPARENT);
                
                let messages = self.messages.lock().unwrap();
                let start = if messages.len() > 12 { messages.len() - 12 } else { 0 };
                
                let mut y = 50;
                for msg in messages.iter().skip(start) {
                    let color = if msg.is_system { RGB(255, 200, 100) } else { RGB(100, 255, 100) };
                    SetTextColor(hdc, color);
                    
                    let prefix = if msg.is_system { "[SYS]" } else { "[CHAT]" };
                    let text = format!("{} {}", prefix, msg.text);
                    TextOutA(hdc, 10, y, text.as_ptr() as *const i8, text.len() as i32);
                    y += 25;
                }
                
                if *self.is_open.lock().unwrap() {
                    let input = self.get_input_text();
                    SetTextColor(hdc, RGB(255, 255, 255));
                    let prompt = format!("> {}", input);
                    TextOutA(hdc, 10, y + 10, prompt.as_ptr() as *const i8, prompt.len() as i32);
                }
                
                SelectObject(hdc, old_font);
                DeleteObject(font as HGDIOBJ);
                ReleaseDC(hwnd, hdc);
            }
        }
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
