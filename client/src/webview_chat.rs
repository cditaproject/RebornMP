// client/src/webview_chat.rs
// Упрощённый WebView2 чат

use webview2::WebView;
use winapi::um::winuser::*;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::ptr;

pub struct WebViewChat {
    webview: Option<WebView>,
    is_open: bool,
}

impl WebViewChat {
    pub fn new() -> Self {
        WebViewChat {
            webview: None,
            is_open: false,
        }
    }
    
    pub fn init(&mut self) -> bool {
        unsafe {
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if hwnd.is_null() {
                println!("[WebView] GTA V not found");
                return false;
            }
            
            // Создаём WebView2 поверх окна игры
            let webview = match WebView::new(Some(hwnd as usize)) {
                Ok(wv) => wv,
                Err(e) => {
                    println!("[WebView] Failed to create: {:?}", e);
                    return false;
                }
            };
            
            // HTML интерфейс чата
            let html = r#"
            <html style="background: transparent;">
            <head>
                <style>
                    body {
                        margin: 0;
                        padding: 0;
                        background: transparent;
                        font-family: 'Segoe UI', Arial, sans-serif;
                    }
                    #chat {
                        position: fixed;
                        bottom: 20px;
                        left: 20px;
                        width: 400px;
                        max-height: 300px;
                        overflow-y: auto;
                        background: rgba(0,0,0,0.7);
                        border-radius: 8px;
                        padding: 10px;
                        color: white;
                    }
                    .msg {
                        margin: 5px 0;
                        padding: 5px;
                        border-radius: 4px;
                    }
                    .system { color: #ffaa00; }
                    .player { color: #00ff00; }
                    #input {
                        position: fixed;
                        bottom: 20px;
                        left: 20px;
                        width: 400px;
                        background: rgba(0,0,0,0.9);
                        border: 1px solid #444;
                        border-radius: 4px;
                        padding: 8px;
                        color: white;
                        display: none;
                    }
                </style>
            </head>
            <body>
                <div id="chat"></div>
                <input type="text" id="input" placeholder="Type message...">
                <script>
                    let chatDiv = document.getElementById('chat');
                    let inputDiv = document.getElementById('input');
                    
                    window.addMessage = function(text, isSystem) {
                        let msg = document.createElement('div');
                        msg.className = 'msg ' + (isSystem ? 'system' : 'player');
                        msg.textContent = text;
                        chatDiv.appendChild(msg);
                        chatDiv.scrollTop = chatDiv.scrollHeight;
                        if (chatDiv.children.length > 50) {
                            chatDiv.removeChild(chatDiv.children[0]);
                        }
                    };
                    
                    window.showInput = function() {
                        inputDiv.style.display = 'block';
                        inputDiv.focus();
                    };
                    
                    window.hideInput = function() {
                        inputDiv.style.display = 'none';
                        inputDiv.value = '';
                    };
                    
                    window.sendMessage = function() {
                        let msg = inputDiv.value;
                        if (msg.trim()) {
                            window.external.notify(msg);
                        }
                        window.hideInput();
                    };
                    
                    inputDiv.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') window.sendMessage();
                    });
                </script>
            </body>
            </html>
            "#;
            
            webview.navigate_to_string(html);
            webview.set_visible(false);
            
            self.webview = Some(webview);
            println!("[WebView] Initialized successfully!");
            true
        }
    }
    
    pub fn add_message(&self, text: &str, is_system: bool) {
        if let Some(webview) = &self.webview {
            let js = format!(r#"window.addMessage("{}", {});"#, 
                text.replace('"', "\\\"").replace('\n', " "),
                if is_system { "true" } else { "false" }
            );
            let _ = webview.execute_script(&js);
        }
    }
    
    pub fn show_chat(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.set_visible(true);
            let _ = webview.execute_script("window.showInput();");
        }
    }
    
    pub fn hide_chat(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.execute_script("window.hideInput();");
            let _ = webview.set_visible(false);
        }
    }
}

lazy_static! {
    pub static ref WEBVIEW_CHAT: Mutex<WebViewChat> = Mutex::new(WebViewChat::new());
}
