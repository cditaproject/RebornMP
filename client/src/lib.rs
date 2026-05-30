mod console;
mod network;

use std::ffi::c_void;
use std::thread;
use std::time::Duration;

#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        1 => {
            console::init_console();
            console::log("========================================");
            console::log("     RebornMP - Client Injected!");
            console::log("========================================");
            
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(2));
                network::start_client();
            });
        }
        _ => ()
    }
    1
}