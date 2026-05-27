#![allow(non_snake_case)]

mod network;
mod chat;
mod hooks;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::HINSTANCE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::winuser::{MessageBoxA, MB_OK, MB_ICONINFORMATION};

static RUNNING: AtomicBool = AtomicBool::new(true);
static CHAT_INSTANCE: Mutex<Option<chat::Chat>> = Mutex::new(None);

#[no_mangle]
pub extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> bool {
    match reason {
        1 => {
            unsafe { DisableThreadLibraryCalls(module); }
            
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(5));
                
                unsafe {
                    MessageBoxA(
                        std::ptr::null_mut(),
                        "Freemode-MP Client Injected!\n\nPress T to open chat\0".as_ptr() as _,
                        "Freemode-MP\0".as_ptr() as _,
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                
                *CHAT_INSTANCE.lock().unwrap() = Some(chat::Chat::new());
                network::start_client();
                
                let mut last_t_state = false;
                
                while RUNNING.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(50));
                    
                    let t_pressed = hooks::is_key_pressed(0x54); // VK_T
                    let enter_pressed = hooks::is_key_pressed(0x0D); // VK_RETURN
                    let escape_pressed = hooks::is_key_pressed(0x1B); // VK_ESCAPE
                    
                    // Открытие/закрытие чата по T
                    if t_pressed && !last_t_state {
                        let mut chat = CHAT_INSTANCE.lock().unwrap();
                        if let Some(ref mut chat_inst) = *chat {
                            chat_inst.toggle();
                            hooks::set_chat_open(chat_inst.is_open());
                        }
                    }
                    last_t_state = t_pressed;
                    
                    // Обработка ввода в чате
                    if hooks::is_chat_open() {
                        // Здесь можно обрабатывать ввод с клавиатуры
                        // Для простоты пока пропустим
                    }
                    
                    network::update();
                }
            });
        }
        0 => {
            RUNNING.store(false, Ordering::Relaxed);
            network::shutdown();
        }
        _ => {}
    }
    true
}

#[no_mangle]
pub extern "system" fn Connect(server: *const i8) -> bool {
    if server.is_null() { return false; }
    let addr = unsafe { std::ffi::CStr::from_ptr(server).to_string_lossy().into_owned() };
    network::set_server_addr(addr);
    true
}

#[no_mangle]
pub extern "system" fn SendChat(msg: *const i8) -> bool {
    if msg.is_null() { return false; }
    let text = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().into_owned() };
    network::send_chat(text)
}