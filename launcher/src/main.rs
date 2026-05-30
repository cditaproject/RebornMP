use std::process::Command;
use std::path::Path;
use std::thread;
use std::time::Duration;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualProtect, VirtualQueryEx, ReadProcessMemory};
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::um::winnt::{PROCESS_ALL_ACCESS, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE};
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

fn main() {
    println!("========================================");
    println!("     RebornMP Launcher v1.0");
    println!("========================================\n");
    
    println!("[1/5] Поиск GTA V...");
    let game_path = find_game_path();
    println!("      Найден: {}", game_path);
    
    println!("[2/5] Запуск GTA V с параметрами...");
    let _game = Command::new(&game_path)
        .arg("-scOfflineOnly")
        .arg("-noSocialClub")
        .arg("-skipPatcherCheck")
        .arg("-nobattleye")
        .arg("-useLevelFast")
        .arg("-verify")
        .arg("-ignoreDifferentVideoCard")
        .arg("-fullscreen")
        .spawn()
        .expect("Не удалось запустить GTA V");
    
    println!("[3/5] Ожидание загрузки игры и авторизации Social Club...");
    println!("      ⏳ Ждём 30 секунд...");
    
    // Ожидание 30 секунд для полной загрузки и авторизации
    for i in (1..=30).rev() {
        print!("\r      Осталось {} секунд...", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("\r      ✅ Игра загружена, продолжаем...        ");
    
    let pid = find_process_id("GTA5.exe");
    if pid == 0 {
        eprintln!("Ошибка: не найден процесс GTA5.exe");
        return;
    }
    println!("      PID процесса: {}", pid);
    
    println!("[4/5] Патчинг Social Club...");
    patch_social_club(pid);
    
    println!("[5/5] Инъекция client.dll...");
    let dll_path = get_dll_path();
    println!("      DLL: {}", dll_path);
    
    match inject_dll(pid, &dll_path) {
        Ok(_) => println!("✅ DLL успешно инжектирована!"),
        Err(e) => println!("❌ Ошибка инъекции: {}", e),
    }
    
    println!("\nНажмите Enter для выхода...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn find_game_path() -> String {
    let paths = [
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe",
        "D:\\SteamLibrary\\steamapps\\common\\Grand Theft Auto V\\PlayGTAV.exe",
        "C:\\Program Files\\Rockstar Games\\Grand Theft Auto V\\PlayGTAV.exe",
    ];
    
    for path in paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }
    
    println!("Не удалось найти GTA V. Введите путь к PlayGTAV.exe:");
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

fn patch_social_club(pid: u32) {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
        if process.is_null() {
            println!("      Не удалось открыть процесс для патча");
            return;
        }
        
        println!("      Поиск паттерна Social Club...");
        
        let pattern = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9];
        
        if let Some(addr) = find_pattern(process, &pattern) {
            let mut old_protect = 0;
            VirtualProtect(addr as _, 3, PAGE_EXECUTE_READWRITE, &mut old_protect);
            
            let patch = [0xB0, 0x01, 0xC3];
            std::ptr::copy_nonoverlapping(patch.as_ptr(), addr as *mut u8, patch.len());
            
            VirtualProtect(addr as _, 3, old_protect, &mut old_protect);
            println!("      ✅ Social Club патч применён!");
        } else {
            println!("      ⚠️ Паттерн не найден, пропускаем");
        }
        
        CloseHandle(process);
    }
}

fn find_pattern(process: HANDLE, pattern: &[u8]) -> Option<*mut u8> {
    unsafe {
        let mut addr: *mut u8 = 0x400000 as *mut u8;
        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        
        while VirtualQueryEx(process, addr as _, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
            if mbi.State == MEM_COMMIT && (mbi.Protect & PAGE_READWRITE) != 0 {
                let mut buffer = vec![0u8; mbi.RegionSize];
                let mut bytes_read = 0;
                
                if ReadProcessMemory(process, mbi.BaseAddress, buffer.as_mut_ptr() as _, buffer.len(), &mut bytes_read) != 0 {
                    for i in 0..bytes_read.saturating_sub(pattern.len()) {
                        if buffer[i..i + pattern.len()] == pattern[..] {
                            return Some((mbi.BaseAddress as usize + i) as *mut u8);
                        }
                    }
                }
            }
            addr = (mbi.BaseAddress as usize + mbi.RegionSize) as *mut u8;
        }
        None
    }
}