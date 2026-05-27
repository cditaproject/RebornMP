use std::collections::VecDeque;
use std::sync::Mutex;
use winapi::um::winuser::{MessageBoxA, MB_OK};
use std::ffi::CString;

const MAX_MESSAGES: usize = 20;

pub struct Chat {
    messages: VecDeque<String>,
    input_buffer: String,
    is_open: bool,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            input_buffer: String::new(),
            is_open: false,
        }
    }
    
    pub fn add_message(&mut self, sender: &str, text: &str) {
        let msg = format!("[{}] {}: {}", get_time(), sender, text);
        self.messages.push_back(msg);
        if self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
        println!("{}", self.messages.back().unwrap());
    }
    
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            println!("[CHAT] Type your message (press Enter to send, Escape to cancel)");
            self.input_buffer.clear();
        }
    }
    
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    pub fn handle_input(&mut self, input: char) -> bool {
        if !self.is_open {
            return false;
        }
        
        match input {
            '\r' => { // Enter
                if !self.input_buffer.is_empty() {
                    let msg = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.is_open = false;
                    return true; // Отправить сообщение
                }
                self.is_open = false;
            }
            '\x08' => { // Backspace
                self.input_buffer.pop();
            }
            '\x1B' => { // Escape
                self.input_buffer.clear();
                self.is_open = false;
            }
            c if c.is_ascii_graphic() || c == ' ' => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        
        // Обновляем отображение
        print!("\r[CHAT] {:<70}", self.input_buffer);
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        
        false
    }
    
    pub fn get_current_input(&self) -> String {
        self.input_buffer.clone()
    }
}

fn get_time() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}