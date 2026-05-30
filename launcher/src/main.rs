use std::process::Command;
use std::path::Path;
use std::thread;
use std::time::Duration;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::um::winnt::{PROCESS_ALL_ACCESS, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};

fn main() {
    println!("=== RebornMP Launcher ===");
    println!("Поиск GTA V...");
    
    let game_path = find_game_path();
    println!("Найден GTA V: {}", game_path);
    
    println!("Запуск GTA V...");
    let _game = Command::new(&game_path)
        .arg("-scOfflineOnly")      // Офлайн режим
        .arg("-skipPatcherCheck")   // Пропустить проверку лаунчера
        .arg("-noSocialClub")       // Отключить Social Club (работает не всегда)
        .arg("-nobattleye")         // Отключить BattlEye
        .arg("-useLevelFast")       // Быстрая загрузка
        .arg("-verify")              // Пропустить проверку файлов
        .spawn()
        .expect("Не удалось запустить GTA V");
    
    println!("GTA V запущен, ожидаем загрузки...");
    thread::sleep(Duration::from_secs(5));
    
    let pid = find_process_id("GTA5.exe");
    if pid == 0 {
        eprintln!("Ошибка: не найден процесс GTA5.exe");
        return;
    }
    println!("Найден процесс GTA V (PID: {})", pid);
    
    let dll_path = get_dll_path();
    println!("Инъекция DLL: {}", dll_path);
    
    match inject_dll(pid, &dll_path) {
        Ok(_) => println!("✅ DLL успешно инжектирована!"),
        Err(e) => println!("❌ Ошибка инъекции: {}", e),
    }
    
    println!("Нажмите Enter для выхода...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn find_game_path() -> String {
    let paths = [
        "C:\\Program Files\\Rockstar Games\\Grand Theft Auto V\\GTA5.exe",
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V\\GTA5.exe",
        "D:\\SteamLibrary\\steamapps\\common\\Grand Theft Auto V\\GTA5.exe",
    ];
    
    for path in paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }
    
    println!("Не удалось найти GTA V. Введите путь к GTA5.exe:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn find_process_id(name: &str) -> u32 {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() {
            return 0;
        }
        
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        
        if Process32First(snapshot, &mut entry) == TRUE {
            loop {
                let name_bytes: Vec<u16> = entry.szExeFile
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u16)
                    .collect();
                let found = String::from_utf16_lossy(&name_bytes);
                
                if found.to_lowercase() == name.to_lowercase() {
                    CloseHandle(snapshot);
                    return entry.th32ProcessID;
                }
                if Process32Next(snapshot, &mut entry) != TRUE {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        0
    }
}

fn get_dll_path() -> String {
    let paths = [
        "target\\release\\client.dll",
        "..\\client\\target\\release\\client.dll",
        "client.dll",
    ];
    
    for path in paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }
    
    println!("Введите путь к client.dll:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn inject_dll(pid: u32, dll_path: &str) -> Result<(), String> {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
        if process.is_null() {
            return Err(format!("OpenProcess failed: {}", GetLastError()));
        }
        
        let dll_wide: Vec<u16> = dll_path.encode_utf16().chain(Some(0)).collect();
        let size = dll_wide.len() * 2;
        
        let remote = VirtualAllocEx(process, std::ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if remote.is_null() {
            CloseHandle(process);
            return Err(format!("VirtualAllocEx failed: {}", GetLastError()));
        }
        
        let mut written = 0;
        WriteProcessMemory(process, remote, dll_wide.as_ptr() as _, size, &mut written);
        
        let kernel32 = GetModuleHandleW("kernel32.dll\0".encode_utf16().collect::<Vec<u16>>().as_ptr());
        let load_library = GetProcAddress(kernel32, "LoadLibraryW\0".as_ptr() as _);
        
        let thread = CreateRemoteThread(process, std::ptr::null_mut(), 0, Some(std::mem::transmute(load_library)), remote, 0, std::ptr::null_mut());
        if thread.is_null() {
            CloseHandle(process);
            return Err(format!("CreateRemoteThread failed: {}", GetLastError()));
        }
        
        WaitForSingleObject(thread, 0xFFFFFFFF);
        CloseHandle(thread);
        CloseHandle(process);
        Ok(())
    }
}