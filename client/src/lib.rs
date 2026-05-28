// client/src/lib.rs

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::GetAsyncKeyState;

mod webview_chat;

use webview_chat::{init_webview, add_message, show_chat, hide_chat};

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(3));
                
                if init_webview() {
                    add_message("════════════════════════════════════════", true);
                    add_message("     RebornMP - GTA V Multiplayer Mod", true);
                    add_message("════════════════════════════════════════", true);
                    add_message("✅ WebView2 чат готов! Нажми T для ввода", true);
                    
                    let mut chat_open = false;
                    
                    loop {
                        unsafe {
                            if GetAsyncKeyState(0x54) & 1 != 0 {
                                if chat_open {
                                    hide_chat();
                                    chat_open = false;
                                } else {
                                    show_chat();
                                    chat_open = true;
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                } else {
                    println!("[RebornMP] WebView2 failed - check Edge Runtime");
                }
            });
            1
        }
        _ => 1,
    }
}
