// client/src/hooks.rs
// RebornMP Hooks Manager - With Keyboard Input

use winapi::um::winuser::{GetAsyncKeyState, VK_RETURN, VK_BACK, VK_ESCAPE};
use std::sync::atomic::{AtomicBool, Ordering};

const VK_T: i32 = 0x54;
static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn install_hooks() -> bool {
    if HOOKS_INSTALLED.load(Ordering::SeqCst) {
        return true;
    }
    println!("[Hooks] Keyboard hooks installed");
    HOOKS_INSTALLED.store(true, Ordering::SeqCst);
    true
}

pub fn process_input() {
    unsafe {
        if GetAsyncKeyState(VK_T) & 1 != 0 {
            crate::ui::CHAT.toggle_input();
        }
        
        if crate::ui::CHAT.is_input_open() {
            if GetAsyncKeyState(VK_RETURN) & 1 != 0 {
                if let Some(msg) = crate::ui::CHAT.send_message() {
                    if !msg.starts_with('/') {
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            net.lock().unwrap().send_chat(&msg);
                        }
                    }
                    crate::ui::CHAT.add_message(format!("You: {}", msg), false);
                }
            }
            
            if GetAsyncKeyState(VK_ESCAPE) & 1 != 0 {
                crate::ui::CHAT.toggle_input();
            }
            
            if GetAsyncKeyState(VK_BACK) & 1 != 0 {
                crate::ui::CHAT.backspace();
            }
        }
    }
}

pub fn remove_hooks() {
    HOOKS_INSTALLED.store(false, Ordering::SeqCst);
}
