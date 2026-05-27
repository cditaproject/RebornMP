#![allow(non_snake_case)]

use std::ffi::c_void;
use winapi::shared::minwindef::HINSTANCE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;

#[no_mangle]
pub extern "system" fn DllMain(_module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> bool {
    match reason {
        1 => {
            unsafe { DisableThreadLibraryCalls(_module); }
            // Абсолютно ничего не делаем в DllMain
        }
        0 => {}
        _ => {}
    }
    true
}

// Минимальные экспортируемые функции
#[no_mangle]
pub extern "system" fn Connect() -> bool {
    true
}

#[no_mangle]
pub extern "system" fn SendChat() -> bool {
    true
}

#[no_mangle]
pub extern "system" fn GetPing() -> u32 {
    0
}