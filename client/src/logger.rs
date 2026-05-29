// client/src/logger.rs
// Подробное логирование в файл

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::time::SystemTime;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init_logger() {
    let mut file = LOG_FILE.lock().unwrap();
    *file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("rebornmp_client.log")
        .ok();
    
    log("=== RebornMP Client Started ===");
    log(&format!("Time: {:?}", SystemTime::now()));
}

pub fn log(msg: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let line = format!("[{}] {}\n", timestamp, msg);
    
    // В консоль
    print!("{}", line);
    
    // В файл
    if let Ok(mut file) = LOG_FILE.lock() {
        if let Some(f) = file.as_mut() {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
}