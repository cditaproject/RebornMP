// launcher/src/main.rs
// RebornMP Launcher - Complete Version (All Features)

use std::process::{Command, Child};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use winapi::um::winnt::{
    PROCESS_ALL_ACCESS, MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE
};
use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread, GetCurrentProcess};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualFreeEx};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::handleapi::CloseHandle;
use winapi::shared::minwindef::{DWORD, LPVOID, BOOL};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TOKEN_QUERY, TokenElevation};
use winapi::ctypes::c_void;

// ========== ЛОГГЕР С РАЗНЫМИ УРОВНЯМИ ==========
#[derive(PartialEq, PartialOrd)]
enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    SUCCESS = 2,
    WARNING = 3,
    ERROR = 4,
}

struct Logger {
    level: LogLevel,
    file: Option<std::fs::File>,
}

impl Logger {
    fn new(level: LogLevel, log_to_file: bool) -> Self {
        let file = if log_to_file {
            let path = PathBuf::from("launcher_debug.log");
            Some(std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)
                .expect("Failed to create log file"))
        } else {
            None
        };
        Logger { level, file }
    }
    
    fn log(&mut self, level: LogLevel, level_str: &str, args: std::fmt::Arguments) {
        if level >= self.level {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let message = format!("[{}] [{}] {}\n", timestamp, level_str, args);
            
            // Цветной вывод в консоль
            let color = match level {
                LogLevel::DEBUG => "\x1b[36m",
                LogLevel::INFO => "\x1b[32m",
                LogLevel::SUCCESS => "\x1b[92m",
                LogLevel::WARNING => "\x1b[33m",
                LogLevel::ERROR => "\x1b[31m",
                _ => "",
            };
            print!("{}{}\x1b[0m", color, message);
            
            // Запись в файл
            if let Some(file) = &mut self.file {
                use std::io::Write;
                let _ = file.write_all(message.as_bytes());
                let _ = file.flush();
            }
        }
    }
    
    fn debug(&mut self, msg: &str) { self.log(LogLevel::DEBUG, "DEBUG", format_args!("{}", msg)); }
    fn info(&mut self, msg: &str) { self.log(LogLevel::INFO, "INFO", format_args!("{}", msg)); }
    fn success(&mut self, msg: &str) { self.log(LogLevel::SUCCESS, "SUCCESS", format_args!("{}", msg)); }
    fn warning(&mut self, msg: &str) { self.log(LogLevel::WARNING, "WARNING", format_args!("{}", msg)); }
    fn error(&mut self, msg: &str) { self.log(LogLevel::ERROR, "ERROR", format_args!("{}", msg)); }
}

// ========== ОСНОВНАЯ ФУНКЦИЯ ==========
fn main() {
    let mut logger = Logger::new(LogLevel::DEBUG, true);
    
    logger.info("========================================");
    logger.info("RebornMP Launcher v0.2.0 - Complete Version");
    logger.info("========================================");
    
    // Проверка прав администратора
    if !is_admin() {
        logger.warning("Launcher not running as Administrator!");
        logger.warning("DLL injection may fail. Run as Admin for best results.");
        logger.debug("To run as Admin: Right-click -> Run as Administrator");
    }
    
    // 1. Поиск GTA V
    logger.info("Step 1: Locating Grand Theft Auto V...");
    let gta_path = match find_gta5_path() {
        Some(path) => {
            logger.success(&format!("Found GTA V at: {}", path.display()));
            path
        },
        None => {
            logger.error("Could not find Grand Theft Auto V installation!");
            logger.error("Search paths checked:");
            logger.error("  - Steam: C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V");
            logger.error("  - Epic: C:\\Program Files\\Epic Games\\GTAV");
            logger.error("  - Rockstar: C:\\Program Files\\Rockstar Games\\Grand Theft Auto V");
            logger.error("  - Registry: HKLM\\Software\\Rockstar Games\\Grand Theft Auto V");
            logger.info("Please install GTA V or specify path manually.");
            return;
        }
    };
    
    // 2. Поиск client.dll
    logger.info("Step 2: Locating RebornMP client.dll...");
    let client_dll_path = match find_client_dll() {
        Some(path) => {
            logger.success(&format!("Found client.dll at: {}", path.display()));
            
            // Проверка размера файла
            if let Ok(metadata) = std::fs::metadata(&path) {
                logger.debug(&format!("File size: {} bytes", metadata.len()));
                if metadata.len() == 0 {
                    logger.error("client.dll is empty! Build failed?");
                    return;
                }
            }
            path
        },
        None => {
            logger.error("Could not find RebornMP client.dll!");
            logger.error("Expected locations:");
            logger.error("  - ./client.dll");
            logger.error("  - ../target/release/client.dll");
            logger.error("  - ./target/release/client.dll");
            logger.info("Please build the project first: cargo build --release");
            return;
        }
    };
    
    // 3. Запуск GTA V
    logger.info("Step 3: Launching Grand Theft Auto V...");
    let mut game_process = match launch_gta5(&gta_path, &mut logger) {
        Ok(process) => {
            logger.success(&format!("Game launched with PID: {}", process.id()));
            process
        },
        Err(e) => {
            logger.error(&format!("Failed to launch GTA V: {}", e));
            return;
        }
    };
    
    // 4. Ожидание загрузки игры
    logger.info("Step 4: Waiting for game to fully load...");
    logger.debug("Waiting 5 seconds for initial loading...");
    thread::sleep(Duration::from_secs(5));
    
    // 5. Инъекция DLL
    logger.info("Step 5: Injecting client.dll into game process...");
    match inject_dll(&mut game_process, &client_dll_path, &mut logger) {
        Ok(_) => {
            logger.success("Successfully injected client.dll!");
            logger.info("The mod should now be active in game.");
        },
        Err(e) => {
            logger.error(&format!("Injection failed: {}", e));
            logger.error("Possible causes:");
            logger.error("  - Antivirus blocking injection");
            logger.error("  - Game is protected (BattlEye)");
            logger.error("  - Insufficient permissions (run as Admin)");
            logger.error("  - Architecture mismatch (x64 required)");
        }
    }
    
    // 6. Завершение
    logger.info("========================================");
    logger.info("Launcher work completed!");
    logger.info("Log saved to: launcher_debug.log");
    logger.info("Press Ctrl+C to exit launcher");
    logger.info("========================================");
    
    // Ждём завершения игры
    logger.debug("Monitoring game process (will exit when game closes)...");
    loop {
        thread::sleep(Duration::from_secs(5));
        if let Ok(exit_code) = game_process.try_wait() {
            if exit_code.is_some() {
                logger.info("Game process has terminated. Exiting...");
                break;
            }
        }
        
        // Проверяем, жив ли процесс (дополнительная проверка)
        unsafe {
            let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, game_process.id());
            if !handle.is_null() {
                let mut exit_code: DWORD = 0;
                winapi::um::processthreadsapi::GetExitCodeProcess(handle, &mut exit_code);
                CloseHandle(handle);
                if exit_code != 259 { // STILL_ACTIVE
                    logger.info("Game process has exited. Exiting...");
                    break;
                }
            }
        }
    }
    
    logger.info("Launcher exiting...");
}

// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========

/// Проверка прав администратора
fn is_admin() -> bool {
    unsafe {
        let mut handle: *mut c_void = std::ptr::null_mut();
        let current_process = GetCurrentProcess();
        
        let result = OpenProcessToken(
            current_process as *mut c_void,
            TOKEN_QUERY,
            &mut handle
        );
        
        if result == 0 || handle.is_null() {
            return false;
        }
        
        let mut elevation = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let token_result = GetTokenInformation(
            handle,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size
        );
        
        CloseHandle(handle);
        token_result != 0 && elevation != 0
    }
}

unsafe extern "system" {
    fn OpenProcessToken(
        ProcessHandle: *mut c_void,
        DesiredAccess: DWORD,
        TokenHandle: *mut *mut c_void,
    ) -> BOOL;
}

/// Поиск пути к GTA V
fn find_gta5_path() -> Option<PathBuf> {
    // Список возможных путей
    let paths = vec![
        PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from("C:\\Program Files\\Epic Games\\GTAV\\PlayGTAV.exe"),
        PathBuf::from("C:\\Program Files\\Rockstar Games\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from("D:\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from("E:\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from(".\\PlayGTAV.exe"),
    ];
    
    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }
    
    // Попытка найти через реестр
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::*;
        
        let hkml = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hkml.open_subkey(r"SOFTWARE\Rockstar Games\Grand Theft Auto V") {
            if let Ok(path) = key.get_value::<String, _>("InstallFolder") {
                let exe_path = PathBuf::from(path).join("PlayGTAV.exe");
                if exe_path.exists() {
                    return Some(exe_path);
                }
            }
        }
    }
    
    None
}

/// Поиск client.dll
fn find_client_dll() -> Option<PathBuf> {
    let paths = vec![
        PathBuf::from(".\\client.dll"),
        PathBuf::from("..\\target\\release\\client.dll"),
        PathBuf::from(".\\target\\release\\client.dll"),
        PathBuf::from("..\\..\\target\\release\\client.dll"),
        PathBuf::from("target\\release\\client.dll"),
    ];
    
    for path in paths {
        if path.exists() {
            return Some(path.canonicalize().unwrap_or(path));
        }
    }
    
    None
}

/// Запуск GTA V
fn launch_gta5(gta_path: &PathBuf, logger: &mut Logger) -> Result<Child, std::io::Error> {
    logger.debug(&format!("Executable: {}", gta_path.display()));
    logger.debug("Working directory: {}", gta_path.parent().unwrap().display());
    
    let game_dir = gta_path.parent().unwrap();
    
    // Параметры запуска (отключаем BattlEye для одиночной игры)
    let args = ["-noBattleEye", "-scOfflineOnly"];
    logger.debug(&format!("Launch arguments: {:?}", args));
    
    Command::new(gta_path)
        .args(&args)
        .current_dir(game_dir)
        .spawn()
}

/// Инъекция DLL в процесс
fn inject_dll(process: &mut Child, dll_path: &PathBuf, logger: &mut Logger) -> Result<(), String> {
    let pid = process.id();
    logger.debug(&format!("Target PID: {}", pid));
    
    // Открываем процесс с максимальными правами
    unsafe {
        logger.debug("Opening process with full access...");
        let process_handle = OpenProcess(
            PROCESS_ALL_ACCESS,
            0,
            pid
        );
        
        if process_handle.is_null() {
            let error = GetLastError();
            logger.error(&format!("OpenProcess failed. Error code: {}", error));
            return Err(format!("Failed to open process. Error: {}", error));
        }
        logger.success("Process opened successfully");
        
        // Получаем путь к DLL
        let dll_path_str = match dll_path.to_str() {
            Some(path) => path,
            None => return Err("Invalid DLL path".to_string())
        };
        logger.debug(&format!("DLL path: {}", dll_path_str));
        
        // Конвертируем путь в wide string (UTF-16)
        let dll_path_wide: Vec<u16> = OsStr::new(dll_path_str)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let dll_path_size = dll_path_wide.len() * 2;
        
        // Выделяем память в процессе для пути DLL
        logger.debug("Allocating memory in target process...");
        let remote_memory = VirtualAllocEx(
            process_handle,
            std::ptr::null_mut(),
            dll_path_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE
        );
        
        if remote_memory.is_null() {
            let error = GetLastError();
            logger.error(&format!("VirtualAllocEx failed. Error: {}", error));
            CloseHandle(process_handle);
            return Err(format!("Failed to allocate memory. Error: {}", error));
        }
        logger.success(&format!("Memory allocated at: {:p}", remote_memory));
        
        // Записываем путь DLL в память процесса
        logger.debug("Writing DLL path to process memory...");
        let mut bytes_written = 0;
        let result = WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_wide.as_ptr() as LPVOID,
            dll_path_size,
            &mut bytes_written
        );
        
        if result == 0 {
            let error = GetLastError();
            logger.error(&format!("WriteProcessMemory failed. Error: {}", error));
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err(format!("Failed to write memory. Error: {}", error));
        }
        logger.success(&format!("Wrote {} bytes to remote process", bytes_written));
        
        // Получаем адрес LoadLibraryA в kernel32.dll
        logger.debug("Getting LoadLibraryA address...");
        let kernel32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr() as *const i8);
        if kernel32.is_null() {
            logger.error("Failed to get kernel32.dll handle");
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err("Failed to get kernel32.dll".to_string());
        }
        
        let load_library_addr = GetProcAddress(kernel32, b"LoadLibraryA\0".as_ptr() as *const i8);
        if load_library_addr.is_null() {
            logger.error("Failed to get LoadLibraryA address");
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err("Failed to get LoadLibraryA".to_string());
        }
        logger.success(&format!("LoadLibraryA at: {:p}", load_library_addr));
        
        // Создаём удалённый поток для загрузки DLL
        logger.debug("Creating remote thread to load DLL...");
        let mut thread_id = 0u32;
        let thread_handle = CreateRemoteThread(
            process_handle,
            std::ptr::null_mut(),
            0,
            std::mem::transmute(load_library_addr),
            remote_memory,
            0,
            &mut thread_id as *mut DWORD
        );
        
        if thread_handle.is_null() {
            let error = GetLastError();
            logger.error(&format!("CreateRemoteThread failed. Error: {}", error));
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err(format!("Failed to create remote thread. Error: {}", error));
        }
        logger.success(&format!("Remote thread created, ID: {}", thread_id));
        
        // Ждём завершения потока (DLL загрузилась)
        logger.debug("Waiting for remote thread to complete...");
        let wait_result = WaitForSingleObject(thread_handle, INFINITE);
        if wait_result != 0 {
            logger.warning(&format!("WaitForSingleObject returned: {}", wait_result));
        } else {
            logger.success("Remote thread completed - DLL loaded successfully!");
        }
        
        // Очистка
        logger.debug("Cleaning up...");
        CloseHandle(thread_handle);
        VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
        CloseHandle(process_handle);
    }
    
    Ok(())
}
