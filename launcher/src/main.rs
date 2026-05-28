// launcher/src/main.rs
// RebornMP Launcher with detailed logging

use std::process::{Command, Child};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// Правильные импорты для winapi
use winapi::um::winnt::{
    PROCESS_ALL_ACCESS, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION,
    PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE
};
use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualFreeEx};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::handleapi::CloseHandle;
use winapi::shared::minwindef::{DWORD, LPVOID, FARPROC, HMODULE, UINT, BOOL};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TOKEN_QUERY, TokenElevation};
use winapi::ctypes::c_void;  // Добавлено для правильного типа

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
            
            let color = match level {
                LogLevel::DEBUG => "\x1b[36m",
                LogLevel::INFO => "\x1b[32m",
                LogLevel::SUCCESS => "\x1b[92m",
                LogLevel::WARNING => "\x1b[33m",
                LogLevel::ERROR => "\x1b[31m",
            };
            print!("{}{}\x1b[0m", color, message);
            
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
    logger.info("RebornMP Launcher v0.1.0 - Debug Mode");
    logger.info("========================================");
    
    if !is_admin() {
        logger.warning("Launcher not running as Administrator!");
        logger.warning("DLL injection may fail. Run as Admin for best results.");
    }
    
    logger.info("Step 1: Locating Grand Theft Auto V...");
    let gta_path = match find_gta5_path() {
        Some(path) => {
            logger.success(&format!("Found GTA V at: {}", path.display()));
            path
        },
        None => {
            logger.error("Could not find Grand Theft Auto V installation!");
            return;
        }
    };
    
    logger.info("Step 2: Locating RebornMP client.dll...");
    let client_dll_path = match find_client_dll() {
        Some(path) => {
            logger.success(&format!("Found client.dll at: {}", path.display()));
            if let Ok(metadata) = std::fs::metadata(&path) {
                logger.debug(&format!("File size: {} bytes", metadata.len()));
            }
            path
        },
        None => {
            logger.error("Could not find RebornMP client.dll!");
            logger.error("Please build the project first: cargo build --release");
            return;
        }
    };
    
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
    
    logger.info("Step 4: Waiting for game to load...");
    thread::sleep(Duration::from_secs(5));
    
    logger.info("Step 5: Injecting client.dll...");
    match inject_dll(&mut game_process, &client_dll_path, &mut logger) {
        Ok(_) => {
            logger.success("Successfully injected client.dll!");
        },
        Err(e) => {
            logger.error(&format!("Injection failed: {}", e));
        }
    }
    
    logger.info("========================================");
    logger.info("Launcher running. Log saved to launcher_debug.log");
    logger.info("Press Ctrl+C to exit");
    logger.info("========================================");
    
    loop {
        thread::sleep(Duration::from_secs(5));
        match game_process.try_wait() {
            Ok(Some(status)) => {
                logger.info(&format!("Game closed with status: {}", status));
                break;
            },
            Ok(None) => continue,
            Err(e) => {
                logger.error(&format!("Error checking game process: {}", e));
                break;
            }
        }
    }
}

// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========

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

// extern блок с unsafe
unsafe extern "system" {
    fn OpenProcessToken(
        ProcessHandle: *mut c_void,
        DesiredAccess: DWORD,
        TokenHandle: *mut *mut c_void,
    ) -> BOOL;
}

fn find_gta5_path() -> Option<PathBuf> {
    let paths = vec![
        PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from("C:\\Program Files\\Epic Games\\GTAV\\PlayGTAV.exe"),
        PathBuf::from("C:\\Program Files\\Rockstar Games\\Grand Theft Auto V\\PlayGTAV.exe"),
        PathBuf::from("D:\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe"),
    ];
    
    for path in paths {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn find_client_dll() -> Option<PathBuf> {
    let paths = vec![
        PathBuf::from(".\\client.dll"),
        PathBuf::from("..\\target\\release\\client.dll"),
        PathBuf::from(".\\target\\release\\client.dll"),
    ];
    
    for path in paths {
        if path.exists() {
            return Some(path.canonicalize().unwrap_or(path));
        }
    }
    None
}

fn launch_gta5(gta_path: &PathBuf, logger: &mut Logger) -> Result<Child, std::io::Error> {
    let game_dir = gta_path.parent().unwrap();
    logger.debug(&format!("Working directory: {}", game_dir.display()));
    
    Command::new(gta_path)
        .current_dir(game_dir)
        .arg("-noBattleEye")
        .arg("-scOfflineOnly")
        .spawn()
}

fn inject_dll(process: &mut Child, dll_path: &PathBuf, logger: &mut Logger) -> Result<(), String> {
    let pid = process.id();
    logger.debug(&format!("Target PID: {}", pid));
    
    unsafe {
        let process_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        
        if process_handle.is_null() {
            let error = GetLastError();
            logger.error(&format!("OpenProcess failed. Error code: {}", error));
            return Err(format!("Failed to open process. Error: {}", error));
        }
        logger.success("Process opened successfully");
        
        let dll_path_str = dll_path.to_str().unwrap();
        logger.debug(&format!("DLL path: {}", dll_path_str));
        
        let dll_path_wide: Vec<u16> = OsStr::new(dll_path_str)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let dll_path_size = dll_path_wide.len() * 2;
        
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
        logger.success(&format!("Wrote {} bytes", bytes_written));
        
        let kernel32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr() as *const i8);
        if kernel32.is_null() {
            logger.error("Failed to get kernel32.dll");
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err("Failed to get kernel32.dll".to_string());
        }
        
        let load_library_addr = GetProcAddress(kernel32, b"LoadLibraryA\0".as_ptr() as *const i8);
        if load_library_addr.is_null() {
            logger.error("Failed to get LoadLibraryA");
            VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
            CloseHandle(process_handle);
            return Err("Failed to get LoadLibraryA".to_string());
        }
        logger.success(&format!("LoadLibraryA at: {:p}", load_library_addr));
        
        let mut thread_id = 0u32;  // ← Исправлено: добавлено mut
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
        
        WaitForSingleObject(thread_handle, INFINITE);
        logger.success("DLL loaded successfully!");
        
        CloseHandle(thread_handle);
        VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
        CloseHandle(process_handle);
    }
    
    Ok(())
}
