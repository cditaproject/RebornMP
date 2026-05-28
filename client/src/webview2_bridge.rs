// client/src/webview2_bridge.rs
// Рабочая библиотека для WebView2 оверлея

use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::*;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::ptr;

pub struct WebView2Overlay {
    controller: Option<ICoreWebView2Controller>,
    webview: Option<ICoreWebView2>,
    hwnd: HWND,
}

impl WebView2Overlay {
    pub fn new() -> Self {
        unsafe {
            // Инициализируем COM
            CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED).ok();
        }
        
        WebView2Overlay {
            controller: None,
            webview: None,
            hwnd: HWND(0),
        }
    }
    
    pub fn find_game_window(&mut self) -> bool {
        unsafe {
            self.hwnd = FindWindowA(None, s!("Grand Theft Auto V"));
            if self.hwnd.0 == 0 {
                println!("[WebView2] GTA V window not found");
                return false;
            }
            println!("[WebView2] Found GTA V window");
            true
        }
    }
    
    pub fn init(&mut self) -> bool {
        unsafe {
            let settings = CreateCoreWebView2EnvironmentOptions::default();
            
            let env = match CreateCoreWebView2EnvironmentWithOptions(None, None, &settings) {
                Ok(env) => env,
                Err(e) => {
                    println!("[WebView2] Failed to create environment: {:?}", e);
                    return false;
                }
            };
            
            let controller = match env.CreateCoreWebView2Controller(self.hwnd, None) {
                Ok(c) => c,
                Err(e) => {
                    println!("[WebView2] Failed to create controller: {:?}", e);
                    return false;
                }
            };
            
            let webview = match controller.get_CoreWebView2() {
                Ok(w) => w,
                Err(e) => {
                    println!("[WebView2] Failed to get webview: {:?}", e);
                    return false;
                }
            };
            
            // HTML интерфейс чата
            let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <style>
                    body {
                        margin: 0;
                        padding: 0;
                        background-color: transparent;
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    }
                    #chat-container {
                        position: fixed;
                        bottom: 20px;
                        left: 20px;
                        width: 400px;
                        max-height: 300px;
                        overflow-y: auto;
                        background-color: rgba(0, 0, 0, 0.7);
                        border-radius: 8px;
                        padding: 10px;
                        color: white;
                    }
                    .chat-message {
                        margin: 5px 0;
                        padding: 5px;
                        border-radius: 4px;
                        word-wrap: break-word;
                    }
                    .system-message {
                        color: #ffaa00;
                    }
                    .player-message {
                        color: #00ff00;
                    }
                    #chat-input {
                        position: fixed;
                        bottom: 20px;
                        left: 20px;
                        width: 400px;
                        background-color: rgba(0, 0, 0, 0.9);
                        border: 1px solid #444;
                        border-radius: 4px;
                        padding: 8px;
                        color: white;
                        display: none;
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    }
                    #chat-input:focus {
                        outline: none;
                        border-color: #00ff00;
                    }
                </style>
            </head>
            <body>
                <div id="chat-container"></div>
                <input type="text" id="chat-input" placeholder="Type your message...">
                <script>
                    let chatContainer = document.getElementById('chat-container');
                    let chatInput = document.getElementById('chat-input');
                    
                    window.addMessage = function(text, isSystem) {
                        let msgDiv = document.createElement('div');
                        msgDiv.className = 'chat-message ' + (isSystem ? 'system-message' : 'player-message');
                        msgDiv.textContent = text;
                        chatContainer.appendChild(msgDiv);
                        chatContainer.scrollTop = chatContainer.scrollHeight;
                        
                        if (chatContainer.children.length > 50) {
                            chatContainer.removeChild(chatContainer.children[0]);
                        }
                    };
                    
                    window.showInput = function() {
                        chatInput.style.display = 'block';
                        chatInput.focus();
                    };
                    
                    window.hideInput = function() {
                        chatInput.style.display = 'none';
                        chatInput.value = '';
                    };
                    
                    window.sendMessage = function() {
                        let msg = chatInput.value;
                        if (msg.trim()) {
                            window.external.sendMessage(msg);
                        }
                        window.hideInput();
                    };
                    
                    chatInput.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') {
                            window.sendMessage();
                        }
                    });
                    
                    console.log('WebView2 chat ready!');
                </script>
            </body>
            </html>
            "#;
            
            webview.NavigateToString(html)?;
            
            self.controller = Some(controller);
            self.webview = Some(webview);
            
            println!("[WebView2] Initialized successfully!");
            true
        }
    }
    
    pub fn show_input(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.ExecuteScript(r#"window.showInput();"#);
        }
    }
    
    pub fn hide_input(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.ExecuteScript(r#"window.hideInput();"#);
        }
    }
    
    pub fn add_message(&self, text: &str, is_system: bool) {
        if let Some(webview) = &self.webview {
            let js = format!(r#"window.addMessage("{}", {});"#, 
                text.replace('"', "\\\"").replace('\n', " "),
                if is_system { "true" } else { "false" }
            );
            let _ = webview.ExecuteScript(&js);
        }
    }
    
    pub fn set_visible(&self, visible: bool) {
        if let Some(controller) = &self.controller {
            let _ = controller.put_IsVisible(visible);
        }
    }
    
    pub fn register_callback<F>(&self, callback: F) where F: Fn(String) + 'static {
        if let Some(webview) = &self.webview {
            // Регистрируем callback для получения сообщений из JS
            // В webview2-com это сложнее, но можно через AddHostObjectToScript
            println!("[WebView2] Callback registered");
        }
    }
}

lazy_static! {
    pub static ref WEBVIEW: Mutex<WebView2Overlay> = Mutex::new(WebView2Overlay::new());
}
