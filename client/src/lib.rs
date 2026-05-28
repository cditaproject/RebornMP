// client/src/lib.rs

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use winapi::um::winnt::DLL_PROCESS_ATTACH;

mod ui;
mod hooks;
mod webview2_bridge;

use ui::CHAT;

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(2));
                
                CHAT.add_message("RebornMP Loaded! Press T to chat".to_string(), true);
                CHAT.add_message("WebView2 чат готов к работе!".to_string(), true);
                
                loop {
                    // Обработка клавиш
                    unsafe {
                        if winapi::um::winuser::GetAsyncKeyState(0x54) & 1 != 0 { // VK_T
                            CHAT.toggle_input();
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
