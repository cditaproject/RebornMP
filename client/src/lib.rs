// client/src/lib.rs
// RebornMP Client - С чатом поверх игры!

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use winapi::um::winnt::DLL_PROCESS_ATTACH;

mod ui;
mod hooks;
mod overlay;

use ui::CHAT;

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(2));
                
                CHAT.add_message("========================================".to_string(), true);
                CHAT.add_message("   RebornMP - GTA V Multiplayer Mod    ".to_string(), true);
                CHAT.add_message("========================================".to_string(), true);
                CHAT.add_message("Press T to open chat".to_string(), true);
                
                hooks::install_hooks();
                
                loop {
                    hooks::process_input();
                    CHAT.render();
                    thread::sleep(Duration::from_millis(50));
                }
            });
            1
        }
        _ => 1,
    }
}
