use winapi::um::winuser::GetAsyncKeyState;
use std::sync::atomic::{AtomicBool, Ordering};

static CHAT_OPEN: AtomicBool = AtomicBool::new(false);

pub fn init() {
    println!("[Hooks] Chat system initialized");
}

pub fn is_key_pressed(key: i32) -> bool {
    unsafe {
        // GetAsyncKeyState возвращает SHORT (i16)
        // Нас интересует старший бит (0x8000)
        let state = GetAsyncKeyState(key);
        (state & 0x8000u16 as i16) != 0
    }
}

pub fn is_chat_open() -> bool {
    CHAT_OPEN.load(Ordering::Relaxed)
}

pub fn set_chat_open(open: bool) {
    CHAT_OPEN.store(open, Ordering::Relaxed);
}