use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::PathBuf;
use std::ptr;
use std::mem;
use std::thread;
use std::time::Duration;
use winapi::um::{
    winnt::*,
    handleapi::CloseHandle,
    memoryapi::{VirtualAllocEx, WriteProcessMemory},
    processthreadsapi::{OpenProcess, CreateRemoteThread},
    synchapi::WaitForSingleObject,
    tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
};
use winreg::RegKey;
use winreg::enums::*;

const INFINITE: u32 = 0xFFFFFFFF;

const SHELLCODE: &[u8] = &[
    0x48, 0x89, 0x5C, 0x24, 0x08, 0x57, 0x48, 0x83, 0xEC, 0x20,
    0x48, 0x8B, 0xD9, 0x48, 0x8B, 0xF1, 0x48, 0x8B, 0x01, 0xFF,
    0xD0, 0x48, 0x8B, 0x5C, 0x24, 0x30, 0x48, 0x83, 0xC4, 0x20,
    0x5F, 0xC3,
];

pub fn start_with_path(server_ip: String, _gta_path: String) -> Result<()> {
    log::info!("=========================================");
    log::info!("RebornMP Launcher v0.1.0 (Single Player Mode)");
    log::info!("=========================================");
    log::info!("Сервер: {}", server_ip);
    
    std::env::set_var("REBORNMP_SERVER", server_ip);
    
    // Запускаем игру
    log::info!("🚀 Запускаем GTA V...");
    launch_game()?;
    
    // Ждём появления GTA5.exe
    log::info!("⏳ Ожидаем появление GTA5.exe (до 90 секунд)...");
    let pid = wait_for_gta5_process()?;
    
    log::info!("🎯 Найден GTA5.exe! PID: {}", pid);
    log::info!("⏳ Даём игре 15 секунд на загрузку...");
    
    for i in (1..=15).rev() {
        log::info!("   Ожидание... {} секунд", i);
        thread::sleep(Duration::from_secs(1));
    }
    
    log::info!("💉 Инжектируем client.dll...");
    inject_dll(pid)?;
    
    log::info!("=========================================");
    log::info!("✅ Мод успешно загружен!");
    log::info!("=========================================");
    
    Ok(())
}

/// Запуск игры через PlayGTAV.exe
fn launch_game() -> Result<()> {
    let play_gta_path = find_play_gta_exe()?;
    log::info!("📌 PlayGTAV.exe: {:?}", play_gta_path);
    
    kill_all_gta_processes();
    thread::sleep(Duration::from_secs(2));
    
    log::info!("🎮 Запускаем с параметрами: -scOfflineOnly -noBattlEye -disableBE");
    
    let game = Command::new(&play_gta_path)
        .arg("-scOfflineOnly")
        .arg("-noBattlEye")
        .arg("-disableBE")
        .spawn();
    
    match game {
        Ok(process) => {
            log::info!("✅ PlayGTAV.exe запущен, PID: {}", process.id());
            std::mem::forget(process);
            Ok(())
        }
        Err(e) => Err(anyhow!("Ошибка запуска: {}", e)),
    }
}

/// Ожидание появления процесса GTA5.exe
fn wait_for_gta5_process() -> Result<u32> {
    for attempt in 1..=90 {
        thread::sleep(Duration::from_secs(1));
        
        if let Some(pid) = find_gta5_pid() {
            log::info!("✅ Найден GTA5.exe на попытке {}", attempt);
            return Ok(pid);
        }
        
        if attempt % 10 == 0 {
            log::info!("   Поиск GTA5.exe... {} секунд", attempt);
        }
    }
    
    Err(anyhow!("GTA5.exe не появился после 90 секунд"))
}

/// Поиск PID процесса GTA5.exe (с правильной обрезкой строк)
fn find_gta5_pid() -> Option<u32> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return None;
        }
        
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        
        if Process32FirstW(snap, &mut entry) == 1 {
            loop {
                // Находим длину строки (до первого нулевого символа)
                let mut len = 0;
                for i in 0..entry.szExeFile.len() {
                    if entry.szExeFile[i] == 0 {
                        break;
                    }
                    len += 1;
                }
                
                // Конвертируем только до нулевого символа
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                
                if name.to_lowercase() == "gta5.exe" {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snap);
                    return Some(pid);
                }
                
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        None
    }
}

/// Поиск PlayGTAV.exe
fn find_play_gta_exe() -> Result<PathBuf> {
    let paths = [
        r"C:\Program Files (x86)\Steam\steamapps\common\Grand Theft Auto V\PlayGTAV.exe",
        r"C:\Program Files\Rockstar Games\Grand Theft Auto V\PlayGTAV.exe",
        r"D:\Steam\steamapps\common\Grand Theft Auto V\PlayGTAV.exe",
    ];
    
    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            log::info!("✅ Найден PlayGTAV.exe: {:?}", p);
            return Ok(p);
        }
    }
    
    if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam") {
        if let Ok(steam_path) = key.get_value::<String, _>("InstallPath") {
            let gta_path = PathBuf::from(steam_path)
                .join("steamapps")
                .join("common")
                .join("Grand Theft Auto V")
                .join("PlayGTAV.exe");
            if gta_path.exists() {
                log::info!("✅ Найден PlayGTAV.exe через реестр Steam");
                return Ok(gta_path);
            }
        }
    }
    
    Err(anyhow!("PlayGTAV.exe не найден!"))
}

/// Убить процессы GTA V
fn kill_all_gta_processes() {
    let processes = ["GTA5.exe", "PlayGTAV.exe", "subprocess.exe"];
    for proc_name in processes {
        let _ = Command::new("taskkill")
            .args(&["/f", "/im", proc_name])
            .output();
    }
    log::info!("✅ Старые процессы завершены");
}

pub fn find_gta5_exe_auto() -> Option<PathBuf> {
    let paths = [
        r"C:\Program Files (x86)\Steam\steamapps\common\Grand Theft Auto V\GTA5.exe",
        r"C:\Program Files\Rockstar Games\Grand Theft Auto V\GTA5.exe",
    ];
    
    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Инжект DLL в процесс
fn inject_dll(pid: u32) -> Result<()> {
    log::info!("🔓 Открываем процесс GTA5.exe (PID: {})...", pid);
    
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
    if handle.is_null() {
        return Err(anyhow!("Не удалось открыть процесс. Запустите лаунчер от администратора!"));
    }
    log::info!("✅ Процесс открыт");
    
    // Ищем client.dll
    let dll_paths = ["client.dll", "./client.dll", "./target/release/client.dll"];
    let mut dll_data = None;
    
    for path in dll_paths {
        if let Ok(data) = std::fs::read(path) {
            dll_data = Some(data);
            log::info!("✅ Найдена DLL по пути: {}", path);
            break;
        }
    }
    
    let dll_data = dll_data.ok_or_else(|| anyhow!("client.dll не найдена!"))?;
    log::info!("📦 DLL размер: {} байт", dll_data.len());
    
    // Выделяем память
    let remote_mem = unsafe {
        VirtualAllocEx(handle, ptr::null_mut(), dll_data.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
    };
    if remote_mem.is_null() {
        return Err(anyhow!("Не удалось выделить память в GTA V"));
    }
    log::info!("📦 Память выделена: {:p}", remote_mem);
    
    // Копируем DLL
    let mut bytes_written = 0;
    unsafe {
        WriteProcessMemory(handle, remote_mem, dll_data.as_ptr() as _, dll_data.len(), &mut bytes_written);
    }
    log::info!("📦 DLL скопирована ({} байт)", bytes_written);
    
    // Выделяем память под shellcode
    let sc_mem = unsafe {
        VirtualAllocEx(handle, ptr::null_mut(), SHELLCODE.len(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)
    };
    if sc_mem.is_null() {
        return Err(anyhow!("Не удалось выделить память под shellcode"));
    }
    
    unsafe {
        WriteProcessMemory(handle, sc_mem, SHELLCODE.as_ptr() as _, SHELLCODE.len(), &mut bytes_written);
    }
    
    // Создаём удалённый поток
    let thread = unsafe {
        CreateRemoteThread(handle, ptr::null_mut(), 0, Some(std::mem::transmute(sc_mem)), remote_mem, 0, ptr::null_mut())
    };
    if thread.is_null() {
        return Err(anyhow!("Не удалось создать поток в GTA V"));
    }
    
    unsafe {
        WaitForSingleObject(thread, INFINITE);
        CloseHandle(thread);
        CloseHandle(handle);
    }
    
    log::info!("💉 Инжект успешен!");
    Ok(())
}