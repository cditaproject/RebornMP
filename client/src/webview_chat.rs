// client/src/webview_chat.rs
// Для версии webview2 0.1.4

use webview2::WebView;
use winapi::um::winuser::*;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell;

thread_local! {
    static WEBVIEW: RefCell<Option<WebView>> = RefCell::new(None);
    static WEBVIEW_READY: AtomicBool = AtomicBool::new(false);
}

pub fn init_webview() -> bool {
    unsafe {
        let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
        if hwnd.is_null() {
            println!("[WebView] GTA V window not found");
            return false;
        }
        
        // Создание WebView2 через WebView::new
        let webview = match WebView::new(Some(hwnd as usize)) {
            Ok(wv) => {
                println!("[WebView] Created successfully");
                wv
            }
            Err(e) => {
                println!("[WebView] Failed to create: {:?}", e);
                return false;
            }
        };
        
        // HTML код
        let html = r#"
        <html style="background: transparent;">
        <head><style>
            body { margin: 0; padding: 0; background: transparent; font-family: 'Segoe UI'; }
            #chat { position: fixed; bottom: 20px; left: 20px; width: 400px; max-height: 300px; overflow-y: auto; background: rgba(0,0,0,0.7); border-radius: 8px; padding: 10px; color: white; }
            .msg { margin: 5px 0; padding: 5px; border-radius: 4px; }
            .system { color: #ffaa00; }
            .player { color: #00ff00; }
            #input { position: fixed; bottom: 20px; left: 20px; width: 400px; background: rgba(0,0,0,0.9); border: 1px solid #444; border-radius: 4px; padding: 8px; color: white; display: none; }
        </style></head>
        <body>
            <div id="chat"></div>
            <input type="text" id="input" placeholder="Message...">
            <script>
                let chat = document.getElementById('chat');
                let input = document.getElementById('input');
                window.addMessage = function(t, s) {
                    let m = document.createElement('div');
                    m.className = 'msg ' + (s ? 'system' : 'player');
                    m.textContent = t;
                    chat.appendChild(m);
                    chat.scrollTop = chat.scrollHeight;
                    if (chat.children.length > 50) chat.removeChild(chat.children[0]);
                };
                window.showInput = function() { input.style.display = 'block'; input.focus(); };
                window.hideInput = function() { input.style.display = 'none'; input.value = ''; };
                input.addEventListener('keypress', function(e) {
                    if (e.key === 'Enter' && input.value.trim()) {
                        window.external.sendMessage(input.value);
                        window.hideInput();
                    }
                });
            </script>
        </body>
        </html>
        "#;
        
        webview.navigate_to_string(html);
        
        WEBVIEW.with(|w| {
            *w.borrow_mut() = Some(webview);
        });
        
        WEBVIEW_READY.with(|r| r.store(true, Ordering::Relaxed));
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
    WEBVIEW_READY.with(|r| r.load(Ordering::Relaxed))
}
