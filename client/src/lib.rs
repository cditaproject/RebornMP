#![allow(non_snake_case)]

mod hooks;
mod memory;
mod network;
mod ui;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use winapi::shared::minwindef::HINSTANCE;  // ← правильный путь
use winapi::um::libloaderapi::DisableThreadLibraryCalls;

static RUNNING: AtomicBool = AtomicBool::new(true);
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static STARTUP: Once = Once::new();

#[no_mangle]
pub extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> bool {
    match reason {
        1 => { // DLL_PROCESS_ATTACH
            unsafe { DisableThreadLibraryCalls(module); }
            
            // Запускаем в отдельном потоке с задержкой
            std::thread::spawn(|| {
                // Даём игре время на стабильную загрузку
                std::thread::sleep(std::time::Duration::from_secs(3));
                
                STARTUP.call_once(|| {
                    initialize_client();
                });
            });
        }
        0 => { // DLL_PROCESS_DETACH
            RUNNING.store(false, Ordering::Relaxed);
            cleanup_client();
        }
        _ => {}
    }
    true
}

fn initialize_client() {
    if INITIALIZED.load(Ordering::Relaxed) {
        return;
    }
    
    // Инициализируем логгер
    let _ = simple_logger::init_with_level(log::Level::Info);
    
    log::info!("=========================================");
    log::info!("RebornMP Client v{}", env!("CARGO_PKG_VERSION"));
    log::info!("=========================================");
    
    // Инициализация компонентов с защитой от паники
    let init_result = std::panic::catch_unwind(|| {
        let _ = memory::init();
        let _ = hooks::init();
        let _ = network::init();
        let _ = ui::init();
    });
    
    match init_result {
        Ok(_) => {
            INITIALIZED.store(true, Ordering::Relaxed);
            log::info!("✅ Все компоненты инициализированы");
            
            // Запускаем главный цикл
            main_loop();
        }
        Err(e) => {
            log::error!("❌ Ошибка инициализации: {:?}", e);
        }
    }
}

fn main_loop() {
    log::info!("🔄 Запущен главный цикл");
    
    while RUNNING.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Безопасный вызов обновлений
        let _ = std::panic::catch_unwind(|| {
            let _ = network::update();
        });
        
        let _ = std::panic::catch_unwind(|| {
            let _ = ui::render();
        });
        
        let _ = std::panic::catch_unwind(|| {
            let _ = hooks::on_frame();
        });
        
        if !memory::is_game_alive() {
            log::info!("Игра завершена, выходим...");
            break;
        }
    }
    
    cleanup_client();
}

fn cleanup_client() {
    log::info!("🛑 Остановка клиента...");
    
    let _ = std::panic::catch_unwind(|| {
        let _ = network::shutdown();
        let _ = ui::shutdown();
        let _ = hooks::cleanup();
    });
    
    log::info!("👋 Client shutdown complete");
}

// Экспортируемые функции для взаимодействия с лаунчером
#[no_mangle]
pub extern "system" fn Connect(server: *const i8) -> bool {
    if server.is_null() {
        return false;
    }
    
    let addr = unsafe {
        std::ffi::CStr::from_ptr(server)
            .to_string_lossy()
            .into_owned()
    };
    
    log::info!("📡 Запрос подключения к серверу: {}", addr);
    network::connect(&addr)
}

#[no_mangle]
pub extern "system" fn SendChat(msg: *const i8) -> bool {
    if msg.is_null() {
        return false;
    }
    
    let text = unsafe {
        std::ffi::CStr::from_ptr(msg)
            .to_string_lossy()
            .into_owned()
    };
    
    network::send_chat(&text)
}

#[no_mangle]
pub extern "system" fn GetPing() -> u32 {
    network::get_ping()
}