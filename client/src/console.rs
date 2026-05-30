use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;

static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));

pub fn init_console() {
    // Создаём консоль Windows
    unsafe {
        winapi::um::consoleapi::AllocConsole();
        let _ = std::fs::File::create("CONOUT$").map(|mut f| {
            writeln!(f, "========================================").ok();
            writeln!(f, "     RebornMP - Console").ok();
            writeln!(f, "========================================").ok();
        });
    }
    
    // Также пишем в файл
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rebornmp.log")
        .ok();
    let mut guard = LOG_FILE.lock().unwrap();
    *guard = file;
}

pub fn log(msg: &str) {
    // Вывод в консоль
    println!("[RebornMP] {}", msg);
    
    // Вывод в файл
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
        }
    }
}