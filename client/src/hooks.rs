// client/src/hooks.rs
// RebornMP Hooks Manager - With UI Support

use std::sync::atomic::{AtomicBool, Ordering};
use winapi::um::winuser::{GetAsyncKeyState, VK_RETURN, VK_BACK, VK_ESCAPE};

static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);
const VK_T: i32 = 0x54;

fn handle_keyboard() {
    unsafe {
        // Открытие чата по T
        if GetAsyncKeyState(VK_T) & 1 != 0 {
            crate::ui::CHAT.toggle_input();
        }
        
        // Если чат открыт
        if crate::ui::CHAT.is_input_open() {
            // Enter - отправка
            if GetAsyncKeyState(VK_RETURN) & 1 != 0 {
                if let Some(msg) = crate::ui::CHAT.send_message() {
                    if msg.starts_with('/') {
                        println!("[CMD] {}", msg);
                        crate::ui::CHAT.add_message(format!("[CMD] {}", msg), true);
                    } else {
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            net.lock().unwrap().send_chat(&msg);
                        }
                        crate::ui::CHAT.add_message(format!("You: {}", msg), false);
                    }
                }
            }
            
            // Escape - отмена
            if GetAsyncKeyState(VK_ESCAPE) & 1 != 0 {
                crate::ui::CHAT.toggle_input();
            }
            
            // Backspace - удаление
            if GetAsyncKeyState(VK_BACK) & 1 != 0 {
                crate::ui::CHAT.backspace();
            }
        }
    }
}

pub fn install_hooks() -> bool {
    if HOOKS_INSTALLED.load(Ordering::SeqCst) {
        return true;
    }
    
    println!("[Hooks] Keyboard hooks installed!");
    HOOKS_INSTALLED.store(true, Ordering::SeqCst);
    true
}

pub fn remove_hooks() {
    if !HOOKS_INSTALLED.load(Ordering::SeqCst) {
        return;
    }
    println!("[Hooks] Hooks removed");
    HOOKS_INSTALLED.store(false, Ordering::SeqCst);
}
