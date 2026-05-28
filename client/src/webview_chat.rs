// client/src/webview_chat.rs
// WebView2 чат с thread_local (работает без Send/Sync)

use webview2::WebView;
use winapi::um::winuser::*;
use std::cell::RefCell;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    static WEBVIEW_INSTANCE: RefCell<Option<WebView>> = RefCell::new(None);
    static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    static CHAT_VISIBLE: AtomicBool = AtomicBool::new(false);
}

pub fn init_webview() -> bool {
    IS_INITIALIZED.with(|init| {
        if init.load(Ordering::Relaxed) {
            return true;
        }
        
        unsafe {
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if hwnd.is_null() {
                println!("[WebView] GTA V not found");
                return false;
            }
            
            // Создаём WebView2
            let webview = match WebView::new(Some(hwnd as usize)) {
                Ok(wv) => wv,
                Err(e) => {
                    println!("[WebView] Create failed: {:?}", e);
                    return false;
                }
            };
            
            // HTML интерфейс
            let html = r#"
            <html style="background: transparent;">
            <head>
                <style>
                    body { margin: 0; padding: 0; background: transparent; font-family: 'Segoe UI', Arial, sans-serif; }
                    #chat { position: fixed; bottom: 20px; left: 20px; width: 400px; max-height: 300px; overflow-y: auto; background: rgba(0,0,0,0.7); border-radius: 8px; padding: 10px; color: white; }
                    .msg { margin: 5px 0; padding: 5px; border-radius: 4px; }
                    .system { color: #ffaa00; }
                    .player { color: #00ff00; }
                    #input { position: fixed; bottom: 20px; left: 20px; width: 400px; background: rgba(0,0,0,0.9); border: 1px solid #444; border-radius: 4px; padding: 8px; color: white; display: none; }
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
                        if (chatDiv.children.length > 50) chatDiv.removeChild(chatDiv.children[0]);
                    };
                    window.showInput = function() { inputDiv.style.display = 'block'; inputDiv.focus(); };
                    window.hideInput = function() { inputDiv.style.display = 'none'; inputDiv.value = ''; };
                    window.sendMessage = function() {
                        let msg = inputDiv.value;
                        if (msg.trim()) { window.external.notify(msg); }
                        window.hideInput();
                    };
                    inputDiv.addEventListener('keypress', function(e) { if (e.key === 'Enter') window.sendMessage(); });
                </script>
            </body>
            </html>
            "#;
            
            webview.navigate_to_string(html);
            webview.set_visible(false);
            
            WEBVIEW_INSTANCE.with(|wv| {
                *wv.borrow_mut() = Some(webview);
            });
            
            init.store(true, Ordering::Relaxed);
            println!("[WebView] Initialized!");
            true
        }
    })
}

pub fn add_message(text: &str, is_system: bool) {
    if !init_webview() { return; }
    
    WEBVIEW_INSTANCE.with(|wv| {
        if let Some(webview) = wv.borrow().as_ref() {
            let js = format!(r#"window.addMessage("{}", {});"#, 
                text.replace('"', "\\\"").replace('\n', " "),
                if is_system { "true" } else { "false" }
            );
            let _ = webview.execute_script(&js, None);
        }
    });
}

pub fn show_chat() {
    if !init_webview() { return; }
    CHAT_VISIBLE.store(true, Ordering::Relaxed);
    
    WEBVIEW_INSTANCE.with(|wv| {
        if let Some(webview) = wv.borrow().as_ref() {
            let _ = webview.set_visible(true);
            let _ = webview.execute_script("window.showInput();", None);
        }
    });
}

pub fn hide_chat() {
    CHAT_VISIBLE.store(false, Ordering::Relaxed);
    
    WEBVIEW_INSTANCE.with(|wv| {
        if let Some(webview) = wv.borrow().as_ref() {
            let _ = webview.execute_script("window.hideInput();", None);
            let _ = webview.set_visible(false);
        }
    });
}

pub fn is_chat_visible() -> bool {
    CHAT_VISIBLE.load(Ordering::Relaxed)
}
