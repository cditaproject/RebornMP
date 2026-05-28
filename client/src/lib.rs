// client/src/lib.rs
// RebornMP Client - Full Version (DLL Entry Point + Chat + Network)

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;

use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

mod hooks;
mod memory;
mod network;
mod ui;

use network::NetworkClient;
use ui::UIManager;
use ui::CHAT;

// Глобальные состояния
static mut NETWORK_CLIENT: Option<Arc<Mutex<NetworkClient>>> = None;
static mut UI_MANAGER: Option<Arc<Mutex<UIManager>>> = None;
static mut GAME_READY: bool = false;

// ========== DLL ENTRY POINT ==========
#[no_mangle]
pub extern "system" fn DllMain(_hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("[RebornMP] DLL attached to process");
            
            // Запускаем клиент в отдельном потоке
            thread::spawn(|| {
                initialize_client();
            });
            1
        }
        DLL_PROCESS_DETACH => {
            println!("[RebornMP] DLL detached");
            cleanup_client();
            1
        }
        _ => 1,
    }
}

// ========== ИНИЦИАЛИЗАЦИЯ КЛИЕНТА ==========
fn initialize_client() {
    println!("[RebornMP] Initializing client...");
    
    // Ждём загрузки игры
    thread::sleep(Duration::from_secs(3));
    
    // Инициализируем UI
    unsafe {
        let ui = Arc::new(Mutex::new(UIManager::new()));
        UI_MANAGER = Some(ui.clone());
        
        // Показываем сообщение о подключении
        CHAT.add_message("🔄 Connecting to RebornMP server...".to_string(), true);
    }
    
    // Подключаемся к серверу
    unsafe {
        let network = Arc::new(Mutex::new(NetworkClient::new()));
        NETWORK_CLIENT = Some(network.clone());
        
        let server_ip = "127.0.0.1:3000";
        println!("[RebornMP] Connecting to {}...", server_ip);
        
        if network.lock().unwrap().connect(server_ip) {
            CHAT.add_message("✅ Connected to RebornMP server!".to_string(), true);
            CHAT.add_message("💬 Type /help for commands".to_string(), true);
            
            // Запускаем основной цикл
            main_loop();
        } else {
            CHAT.add_message("❌ Failed to connect to server!".to_string(), true);
            println!("[RebornMP] Connection failed");
        }
    }
}

// ========== ОСНОВНОЙ ЦИКЛ ==========
fn main_loop() {
    println!("[RebornMP] Main loop started");
    
    loop {
        unsafe {
            // Получаем позицию игрока из памяти (если игра загружена)
            if !GAME_READY {
                if let Some(pos) = memory::get_player_position() {
                    GAME_READY = true;
                    CHAT.add_message("🎮 Game loaded! Position tracking active.".to_string(), true);
                    println!("[RebornMP] Game ready at position: {:?}", pos);
                }
            }
            
            // Отправляем позицию на сервер
            if GAME_READY {
                if let Some(net) = &NETWORK_CLIENT {
                    if let Some(pos) = memory::get_player_position() {
                        net.lock().unwrap().send_position(pos.0, pos.1, pos.2);
                    }
                }
            }
            
            // Получаем сообщения от сервера
            if let Some(net) = &NETWORK_CLIENT {
                let messages = net.lock().unwrap().receive_messages();
                for msg in messages {
                    handle_server_message(&msg);
                }
            }
            
            // Обновляем UI
            if let Some(ui) = &UI_MANAGER {
                ui.lock().unwrap().update();
            }
        }
        
        thread::sleep(Duration::from_millis(50)); // 20 FPS синхронизация
    }
}

// ========== ОБРАБОТКА СООБЩЕНИЙ ОТ СЕРВЕРА ==========
fn handle_server_message(message: &str) {
    println!("[RebornMP] Server message: {}", message);
    
    // Парсим JSON
    if message.contains("\"type\":\"chat\"") {
        if let Some(text) = extract_json_value(message, "message") {
            CHAT.add_message(text, false);
        }
    }
    else if message.contains("\"type\":\"init\"") {
        if let Some(name) = extract_json_value(message, "name") {
            CHAT.add_message(format!("🏙️ Welcome to RebornMP, {}!", name), true);
        }
        if let Some(money) = extract_json_value(message, "money") {
            CHAT.add_message(format!("💰 Your balance: ${}", money), true);
        }
    }
    else {
        // Неизвестный тип сообщения
        CHAT.add_message(message.to_string(), true);
    }
}

// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========
fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":\"", key);
    if let Some(start) = json.find(&search) {
        let rest = &json[start + search.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    
    let search = format!("\"{}\":", key);
    if let Some(start) = json.find(&search) {
        let rest = &json[start + search.len()..];
        let end = rest.find(',').or_else(|| rest.find('}')).unwrap_or(rest.len());
        let value = rest[..end].trim();
        if let Ok(num) = value.parse::<i32>() {
            return Some(num.to_string());
        }
    }
    
    None
}

fn cleanup_client() {
    unsafe {
        if let Some(net) = &NETWORK_CLIENT {
            net.lock().unwrap().disconnect();
        }
    }
    println!("[RebornMP] Client cleaned up");
}
