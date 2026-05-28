// client/src/webview_chat.rs
// WebView2 чат для версии 0.1.4 - полностью исправлен

use webview2::{Environment, Controller, WebView};
use winapi::um::winuser::*;
use winapi::shared::windef::HWND;
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
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        let result = Environment::builder()
            .build(move |env_result| {
                let _ = tx.send(env_result);
                Ok(())
            });
        
        if result.is_err() {
            println!("[WebView] Failed to start builder");
            return false;
        }
        
        let env = match rx.recv().unwrap() {
            Ok(env) => env,
            Err(e) => {
                println!("[WebView] Environment error: {:?}", e);
                return false;
            }
        };
        
        println!("[WebView] Environment created, creating controller...");
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        let _ = env.create_controller(hwnd, move |controller_result| {
            let _ = tx.send(controller_result);
            Ok(())
        });
        
        let controller = match rx.recv().unwrap() {
            Ok(c) => c,
            Err(e) => {
                println!("[WebView] Controller error: {:?}", e);
                return false;
            }
        };
        
        println!("[WebView] Controller created, getting webview...");
        
        let webview = match controller.get_webview() {
            Ok(wv) => wv,
            Err(e) => {
                println!("[WebView] Failed to get webview: {:?}", e);
                return false;
            }
        };
        
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
                
                window.sendMessage = function() {
                    let msg = chatInput.value;
                    if (msg.trim()) {
                        window.external.notify(msg);
                        window.hideInput();
                    }
                };
                
                chatInput.addEventListener('keypress', function(e) {
                    if (e.key === 'Enter') {
                        window.sendMessage();
                    }
                });
                
                console.log('WebView2 Chat Ready!');
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
            let _ = wv.execute_script(&js, |_| Ok(()));
        }
    });
}

pub fn show_chat() {
    WEBVIEW.with(|w| {
        if let Some(wv) = w.borrow().as_ref() {
            let _ = wv.execute_script("window.showInput();", |_| Ok(()));
        }
    });
}

pub fn hide_chat() {
    WEBVIEW.with(|w| {
        if let Some(wv) = w.borrow().as_ref() {
            let _ = wv.execute_script("window.hideInput();", |_| Ok(()));
        }
    });
}
