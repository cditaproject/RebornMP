// client/src/ui.rs
// RebornMP UI Manager - DirectX Overlay

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ptr;
use lazy_static::lazy_static;
use winapi::um::d3d11::*;
use winapi::um::dxgi::*;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::*;
use winapi::um::d3dcommon::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

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
}

impl DirectXOverlay {
    pub fn new() -> Self {
        DirectXOverlay {
            swapchain: ptr::null_mut(),
            device: ptr::null_mut(),
            context: ptr::null_mut(),
            render_target_view: ptr::null_mut(),
            hwnd: ptr::null_mut(),
        }
    }
    
    pub fn find_game_window(&mut self) -> bool {
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
    
    pub fn create_overlay(&mut self) -> bool {
        unsafe {
            if self.hwnd.is_null() {
                return false;
            }
            
            let dxgi_factory: *mut IDXGIFactory = ptr::null_mut();
            let mut adapter: *mut IDXGIAdapter = ptr::null_mut();
            
            // Создаём swap chain
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
            
            if result < 0 {
                println!("[UI] Failed to create D3D11 device");
                return false;
            }
            
            println!("[UI] DirectX overlay created");
            true
        }
    }
    
    pub fn render_text(&mut self, text: &str, x: i32, y: i32, color: u32) {
        unsafe {
            if self.context.is_null() {
                return;
            }
            
            // В реальном проекте здесь рендеринг текста через DirectWrite
            // Пока просто заглушка
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
    
    pub fn init_ui(&self) {
        let mut overlay = self.overlay.lock().unwrap();
        if overlay.find_game_window() {
            overlay.create_overlay();
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
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.clear();
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
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.push(c);
            self.render();
        }
    }
    
    pub fn backspace(&self) {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.pop();
            self.render();
        }
    }
    
    pub fn send_message(&self) -> Option<String> {
        if *self.is_open.lock().unwrap() {
            let mut buffer = self.input_buffer.lock().unwrap();
            let message = buffer.clone();
            if !message.is_empty() {
                buffer.clear();
                *self.is_open.lock().unwrap() = false;
                return Some(message);
            }
            *self.is_open.lock().unwrap() = false;
        }
        None
    }
    
    pub fn render(&self) {
        let mut overlay = self.overlay.lock().unwrap();
        let messages = self.get_messages();
        let start = if messages.len() > 10 { messages.len() - 10 } else { 0 };
        
        let mut y = 50;
        for msg in &messages[start..] {
            let color = if msg.is_system { 0xFFFFAA00 } else { 0xFF00FF00 };
            overlay.render_text(&msg.text, 10, y, color);
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
    
    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
        self.render();
    }
}

pub struct UIManager {
    last_render: Mutex<u64>,
}

impl UIManager {
    pub fn new() -> Self {
        println!("[UI] UIManager initialized (DirectX Mode)");
        let ui = UIManager {
            last_render: Mutex::new(0),
        };
        CHAT.init_ui();
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
}

lazy_static! {
    pub static ref CHAT: ChatManager = ChatManager::new();
    pub static ref UI: UIManager = UIManager::new();
}

pub fn add_chat_message(text: String, is_system: bool) {
    CHAT.add_message(text, is_system);
}
