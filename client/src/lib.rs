// client/src/lib.rs
// RebornMP Client - Full Version (DLL Entry Point + Chat + Network)

use std::ffi::c_void;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;

use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::shared::minwindef::HMODULE;

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
pub extern "system" fn DllMain(hinst: *mut c_void, reason: u32, _reserved: *mut c_void) -> u32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                DisableThreadLibraryCalls(hinst as HMODULE);
            }
            
            println!("[RebornMP] DLL attached to process (PID: {})", std::process::id());
            
            thread::spawn(|| {
                thread::sleep(Duration::from_secs(3));
                initialize_client();
                ui::CHAT.find_game_window();
            });
            1
        }
        DLL_PROCESS_DETACH => {
            println!("[RebornMP] DLL detached from process");
            cleanup_client();
            1
        }
        _ => 1,
    }
}

// ========== ИНИЦИАЛИЗАЦИЯ КЛИЕНТА ==========
fn initialize_client() {
    println!("[RebornMP] Initializing client...");
    
    unsafe {
        let ui = Arc::new(Mutex::new(UIManager::new()));
        UI_MANAGER = Some(ui.clone());
    }
    
    CHAT.add_message("========================================".to_string(), true);
    CHAT.add_message("   RebornMP - GTA V Multiplayer Mod    ".to_string(), true);
    CHAT.add_message("========================================".to_string(), true);
    CHAT.add_message("🔄 Connecting to server...".to_string(), true);
    
    // Устанавливаем хуки для DirectX
    if !hooks::install_hooks() {
        println!("[RebornMP] Failed to install hooks!");
        CHAT.add_message("❌ Failed to install DirectX hooks! Chat may not display.".to_string(), true);
    } else {
        println!("[RebornMP] Hooks installed successfully");
    }
    
    unsafe {
        let network = Arc::new(Mutex::new(NetworkClient::new()));
        NETWORK_CLIENT = Some(network.clone());
        
        let server_ip = "127.0.0.1:3000";
        println!("[RebornMP] Connecting to {}...", server_ip);
        
        if network.lock().unwrap().connect(server_ip) {
            CHAT.add_message("✅ Connected to RebornMP server!".to_string(), true);
            CHAT.add_message("💬 Type /help for list of commands".to_string(), true);
            CHAT.add_message("📌 Press T to open chat".to_string(), true);
            
            start_main_loop();
        } else {
            CHAT.add_message("❌ Failed to connect to server!".to_string(), true);
            CHAT.add_message("⚠️ Make sure server is running on 127.0.0.1:3000".to_string(), true);
            println!("[RebornMP] Connection failed");
        }
    }
}

// ========== ОСНОВНОЙ ЦИКЛ ==========
fn start_main_loop() {
    println!("[RebornMP] Main loop started");
    
    loop {
        unsafe {
            if !GAME_READY {
                if let Some(pos) = memory::get_player_position() {
                    GAME_READY = true;
                    CHAT.add_message(format!("🎮 Game loaded! Position: ({:.1}, {:.1}, {:.1})", pos.0, pos.1, pos.2), true);
                    println!("[RebornMP] Game ready at position: {:?}", pos);
                }
            }
            
            if GAME_READY {
                if let Some(net) = &NETWORK_CLIENT {
                    if let Some(pos) = memory::get_player_position() {
                        net.lock().unwrap().send_position(pos.0, pos.1, pos.2);
                    }
                }
            }
            
            if let Some(net) = &NETWORK_CLIENT {
                let messages = net.lock().unwrap().receive_messages();
                for msg in messages {
                    handle_server_message(&msg);
                }
            }
            
            if let Some(ui) = &UI_MANAGER {
                ui.lock().unwrap().update();
            }
        }
        
        thread::sleep(Duration::from_millis(50));
    }
}

// ========== ОБРАБОТКА СООБЩЕНИЙ ОТ СЕРВЕРА ==========
fn handle_server_message(message: &str) {
    println!("[RebornMP] Server message: {}", message);
    
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
    else if message.contains("\"type\":\"player_joined\"") {
        if let Some(name) = extract_json_value(message, "name") {
            CHAT.add_message(format!("🟢 {} joined the game", name), true);
        }
    }
    else if message.contains("\"type\":\"player_left\"") {
        if let Some(name) = extract_json_value(message, "name") {
            CHAT.add_message(format!("🔴 {} left the game", name), true);
        }
    }
    else {
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
    hooks::remove_hooks();
    println!("[RebornMP] Client cleaned up");
}
