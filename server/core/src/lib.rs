// server/core/src/lib.rs
// Компилируется в server_core.dll

use std::ffi::c_void;

// Меняем имя функции с shutdown на другое, чтобы избежать конфликта
#[no_mangle]
pub extern "C" fn core_shutdown() {
    println!("[Rust Core] Server core shutdown");
}

#[no_mangle]
pub extern "C" fn core_init() {
    println!("[Rust Core] Server core initialized");
}

#[no_mangle]
pub extern "C" fn core_process_packet(data: *const u8, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }
    
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    if let Ok(msg) = std::str::from_utf8(slice) {
        println!("[Rust Core] Processing packet: {}", msg);
    }
}

// Экспорт вспомогательных функций
#[no_mangle]
pub extern "C" fn core_get_version() -> *const c_void {
    let version = "1.0.0\0";
    version.as_ptr() as *const c_void
}