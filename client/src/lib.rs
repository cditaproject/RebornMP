// client/src/lib.rs

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::*;

mod webview_chat;

use webview_chat::WEBVIEW_CHAT;

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(3));
                
                let mut chat = WEBVIEW_CHAT.lock().unwrap();
                chat.init();
                chat.add_message("RebornMP Loaded! Press T to chat", true);
                
                loop {
                    unsafe {
                        // Проверяем клавишу T
                        if GetAsyncKeyState(0x54) & 1 != 0 {
                            let mut chat = WEBVIEW_CHAT.lock().unwrap();
                            chat.show_chat();
                        }
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            });
            1
        }
        _ => 1,
    }
}
