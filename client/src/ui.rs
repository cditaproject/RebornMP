// client/src/ui.rs
// RebornMP DirectX Overlay Chat

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ptr;
use winapi::um::d3d11::*;
use winapi::um::dxgi::*;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::*;
use winapi::um::d3dcommon::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;
use lazy_static::lazy_static;

#[derive(Clone)]
pub struct ChatMessage {
    pub text: String,
    pub timestamp: u64,
    pub is_system: bool,
}

pub struct DirectXOverlay {
    swapchain: *mut IDXGISwapChain,
    device: *mut ID3D11Device,
    context: *mut ID3D11DeviceContext,
    render_target_view: *mut ID3D11RenderTargetView,
    hwnd: HWND,
    initialized: bool,
}

impl DirectXOverlay {
    pub fn new() -> Self {
        DirectXOverlay {
            swapchain: ptr::null_mut(),
            device: ptr::null_mut(),
            context: ptr::null_mut(),
            render_target_view: ptr::null_mut(),
            hwnd: ptr::null_mut(),
            initialized: false,
        }
    }
    
    pub fn find_window(&mut self) -> bool {
        unsafe {
            self.hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if self.hwnd.is_null() {
                println!("[UI] GTA V window not found");
                return false;
            }
            println!("[UI] Found GTA V window");
            true
        }
    }
    
    pub fn init(&mut self) -> bool {
        if self.initialized { return true; }
        
        unsafe {
            let mut swapchain_desc: DXGI_SWAP_CHAIN_DESC = std::mem::zeroed();
            swapchain_desc.BufferDesc.Width = 1920;
            swapchain_desc.BufferDesc.Height = 1080;
            swapchain_desc.BufferDesc.RefreshRate.Numerator = 60;
            swapchain_desc.BufferDesc.RefreshRate.Denominator = 1;
            swapchain_desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
            swapchain_desc.SampleDesc.Count = 1;
            swapchain_desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
            swapchain_desc.BufferCount = 2;
            swapchain_desc.OutputWindow = self.hwnd;
            swapchain_desc.Windowed = 1;
            
            let result = D3D11CreateDeviceAndSwapChain(
                ptr::null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                0,
                D3D11_SDK_VERSION,
                &swapchain_desc,
                &mut self.swapchain,
                &mut self.device,
                ptr::null_mut(),
                &mut self.context,
            );
            
            if result >= 0 {
                self.initialized = true;
                println!("[UI] DirectX overlay initialized");
                true
            } else {
                println!("[UI] Failed to init DirectX: {}", result);
                false
            }
        }
    }
    
    pub fn render_text(&self, text: &str, x: i32, y: i32, color: u32) {
        // Простая отрисовка текста через Win32 GDI поверх DirectX
        unsafe {
            if !self.hwnd.is_null() {
                let hdc = GetDC(self.hwnd);
                SetTextColor(hdc, color);
                SetBkMode(hdc, TRANSPARENT);
                TextOutA(hdc, x, y, text.as_ptr() as *const i8, text.len() as i32);
                ReleaseDC(self.hwnd, hdc);
            }
        }
    }
}

pub struct ChatManager {
    messages: Mutex<VecDeque<ChatMessage>>,
    input_buffer: Mutex<String>,
    is_open: Mutex<bool>,
    overlay: Mutex<DirectXOverlay>,
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            messages: Mutex::new(VecDeque::with_capacity(100)),
            input_buffer: Mutex::new(String::new()),
            is_open: Mutex::new(false),
            overlay: Mutex::new(DirectXOverlay::new()),
        }
    }
    
    pub fn init(&self) {
        let mut overlay = self.overlay.lock().unwrap();
        if overlay.find_window() {
            overlay.init();
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
        let overlay = self.overlay.lock().unwrap();
        if !overlay.initialized { return; }
        
        let messages = self.messages.lock().unwrap();
        let start = if messages.len() > 12 { messages.len() - 12 } else { 0 };
        
        let mut y = 50;
        for msg in messages.iter().skip(start) {
            let color = if msg.is_system { 0x00FFFFAA } else { 0x0000FF00 };
            let text = format!("{} {}", if msg.is_system { "[SYS]" } else "[CHAT]", msg.text);
            overlay.render_text(&text, 10, y, color);
            y += 25;
        }
        
        if *self.is_open.lock().unwrap() {
            let input = self.get_input_text();
            overlay.render_text(&format!("> {}", input), 10, y + 10, 0xFFFFFFFF);
        }
    }
    
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.messages.lock().unwrap().iter().cloned().collect()
    }
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
}
