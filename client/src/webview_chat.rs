// client/src/webview_chat.rs
// WebView2 чат - правильная реализация по документации

use webview2::{EnvironmentBuilder, Controller, WebView};
use winapi::um::winuser::*;
use std::ptr;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    static CONTROLLER: RefCell<Option<Controller>> = RefCell::new(None);
    static WEBVIEW: RefCell<Option<WebView>> = RefCell::new(None);
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
}

pub fn init_webview() -> bool {
    if INITIALIZED.with(|i| i.load(Ordering::Relaxed)) {
        return true;
    }
    
    unsafe {
        let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
        if hwnd.is_null() {
            println!("[WebView] GTA V window not found");
            return false;
        }
        
        println!("[WebView] Creating environment...");
        
        // Используем EnvironmentBuilder как в документации
        let env = match EnvironmentBuilder::new()
            .build() {
            Ok(env) => {
                println!("[WebView] Environment created");
                env
            }
            Err(e) => {
                println!("[WebView] Failed to create environment: {:?}", e);
                return false;
            }
        };
        
        // Создаём контроллер
        let controller = match env.create_controller(hwnd as isize, None) {
            Ok(c) => {
                println!("[WebView] Controller created");
                c
            }
            Err(e) => {
                println!("[WebView] Failed to create controller: {:?}", e);
                return false;
            }
        };
        
        // Получаем WebView из контроллера
        let webview = match controller.get_core_web_view2() {
            Ok(wv) => {
                println!("[WebView] WebView obtained");
                wv
            }
            Err(e) => {
                println!("[WebView] Failed to get WebView: {:?}", e);
                return false;
            }
        };
        
        // HTML интерфейс
        let html = r#"
        <!DOCTYPE html>
        <html style="background: transparent;">
        <head>
            <meta charset="UTF-8">
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }
                body { background: transparent; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; }
                #chat-container {
                    position: fixed;
                    bottom: 20px;
                    left: 20px;
                    width: 450px;
                    max-height: 350px;
                    overflow-y: auto;
                    background: linear-gradient(to top, rgba(0,0,0,0.85), rgba(0,0,0,0.7));
                    border-radius: 10px;
                    padding: 12px;
                    backdrop-filter: blur(5px);
                    border-left: 3px solid #00ff00;
                }
                .message {
                    margin: 8px 0;
                    padding: 6px 10px;
                    border-radius: 6px;
                    word-wrap: break-word;
                    animation: fadeIn 0.2s ease-in;
                }
                @keyframes fadeIn {
                    from { opacity: 0; transform: translateY(5px); }
                    to { opacity: 1; transform: translateY(0); }
                }
                .system-message { color: #ffaa44; text-shadow: 0 0 2px rgba(0,0,0,0.5); }
                .player-message { color: #44ff44; text-shadow: 0 0 2px rgba(0,0,0,0.5); }
                .player-name { color: #ffaa44; font-weight: bold; }
                #input-container {
                    position: fixed;
                    bottom: 20px;
                    left: 20px;
                    width: 450px;
                    background: rgba(0,0,0,0.95);
                    border-radius: 8px;
                    border: 1px solid #44ff44;
                    padding: 10px;
                    display: none;
                    backdrop-filter: blur(5px);
                }
                #chat-input {
                    width: 100%;
                    background: transparent;
                    border: none;
                    color: white;
                    font-size: 14px;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    outline: none;
                }
                #chat-input::placeholder { color: #888; }
            </style>
        </head>
        <body>
            <div id="chat-container"></div>
            <div id="input-container">
                <input type="text" id="chat-input" placeholder="Type your message..." autocomplete="off">
            </div>
            <script>
                let chatContainer = document.getElementById('chat-container');
                let inputContainer = document.getElementById('input-container');
                let chatInput = document.getElementById('chat-input');
                
                window.addMessage = function(text, isSystem) {
                    let msgDiv = document.createElement('div');
                    msgDiv.className = 'message ' + (isSystem ? 'system-message' : 'player-message');
                    msgDiv.textContent = text;
                    chatContainer.appendChild(msgDiv);
                    msgDiv.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                    while (chatContainer.children.length > 100) {
                        chatContainer.removeChild(chatContainer.firstChild);
                    }
                };
                
                window.showInput = function() {
                    inputContainer.style.display = 'block';
                    chatInput.focus();
                };
                
                window.hideInput = function() {
                    inputContainer.style.display = 'none';
                    chatInput.value = '';
                };
                
                chatInput.addEventListener('keypress', function(e) {
                    if (e.key === 'Enter' && chatInput.value.trim()) {
                        window.external.notify(chatInput.value);
                        window.hideInput();
                    }
                });
            </script>
        </body>
        </html>
        "#;
        
        webview.navigate_to_string(html);
        
        CONTROLLER.with(|c| {
            *c.borrow_mut() = Some(controller);
        });
        
        WEBVIEW.with(|w| {
            *w.borrow_mut() = Some(webview);
        });
        
        INITIALIZED.with(|i| i.store(true, Ordering::Relaxed));
        println!("[WebView] Ready!");
        true
    }
}

pub fn add_message(text: &str, is_system: bool) {
    WEBVIEW.with(|w| {
        if let Some(wv) = w.borrow().as_ref() {
            let js = format!(r#"window.addMessage("{}", {});"#, 
                text.replace('"', "\\\"").replace('\n', " "),
                if is_system { "true" } else { "false" }
            );
            let _ = wv.execute_script(&js);
        }
    });
}

pub fn show_chat() {
    WEBVIEW.with(|w| {
        if let Some(wv) = w.borrow().as_ref() {
            let _ = wv.execute_script("window.showInput();");
        }
    });
}

pub fn hide_chat() {
    WEBVIEW.with(|w| {
        if let Some(wv) = w.borrow().as_ref() {
            let _ = wv.execute_script("window.hideInput();");
        }
    });
}

pub fn is_ready() -> bool {
    INITIALIZED.with(|i| i.load(Ordering::Relaxed))
}
