#![allow(non_snake_case)]

use std::ffi::c_void;
use std::fs::{self, OpenOptions};
use std::io::Write;
use winapi::shared::minwindef::HINSTANCE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Config {
    server_ip: String,
}

static SERVER_IP: Mutex<String> = Mutex::new(String::new());

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rebornmp.log") 
    {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
    }
    println!("{}", msg);
}

#[no_mangle]
pub extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> bool {
    match reason {
        1 => {
            unsafe { DisableThreadLibraryCalls(module); }
            
            log_to_file("========================================");
            log_to_file("RebornMP Client v1.0.0 - DLL ATTACH");
            log_to_file("========================================");
            
            std::thread::spawn(|| {
                // Ждём загрузки игры
                log_to_file("Waiting 8 seconds for game to load...");
                std::thread::sleep(std::time::Duration::from_secs(8));
                
                log_to_file("Loading config...");
                let config_path = std::env::current_dir().unwrap().join("rebornmp_config.json");
                log_to_file(&format!("Config path: {:?}", config_path));
                
                if let Ok(data) = fs::read_to_string(&config_path) {
                    log_to_file(&format!("Config data: {}", data));
                    if let Ok(config) = serde_json::from_str::<Config>(&data) {
                        log_to_file(&format!("Server IP: {}", config.server_ip));
                        *SERVER_IP.lock().unwrap() = config.server_ip.clone();
                        log_to_file("✅ RebornMP successfully initialized!");
                    } else {
                        log_to_file("❌ Failed to parse config JSON");
                    }
                } else {
                    log_to_file("❌ Config file not found!");
                    log_to_file(&format!("Searched at: {:?}", config_path));
                }
                
                log_to_file("========================================");
            });
        }
        0 => {
            log_to_file("RebornMP Client - DLL DETACH");
        }
        _ => {}
    }
    true
}

#[no_mangle]
pub extern "system" fn IsRebornMPLoaded() -> bool {
    true
}

#[no_mangle]
pub extern "system" fn GetServerIP() -> *const i8 {
    let ip = SERVER_IP.lock().unwrap();
    ip.as_ptr() as *const i8
}