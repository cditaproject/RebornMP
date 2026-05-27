use anyhow::{Result, anyhow};
use std::ptr;
use std::mem;
use winapi::um::{
    winnt::*,
    handleapi::CloseHandle,
    memoryapi::{VirtualAllocEx, WriteProcessMemory},
    processthreadsapi::{OpenProcess, CreateRemoteThread},
    synchapi::WaitForSingleObject,
    tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
};

const INFINITE: u32 = 0xFFFFFFFF;

const SHELLCODE: &[u8] = &[
    0x48, 0x89, 0x5C, 0x24, 0x08, 0x57, 0x48, 0x83, 0xEC, 0x20,
    0x48, 0x8B, 0xD9, 0x48, 0x8B, 0xF1, 0x48, 0x8B, 0x01, 0xFF,
    0xD0, 0x48, 0x8B, 0x5C, 0x24, 0x30, 0x48, 0x83, 0xC4, 0x20,
    0x5F, 0xC3,
];

/// Инжект в уже запущенную игру
pub fn inject_to_running(server_ip: String) -> Result<()> {
    println!("🔍 Поиск процесса GTA5.exe...");
    
    let pid = find_gta5_pid()
        .ok_or_else(|| anyhow!("GTA V не запущена! Запустите игру сначала."))?;
    
    println!("🎯 Найден GTA5.exe! PID: {}", pid);
    println!("💉 Инжектируем client.dll...");
    
    std::env::set_var("REBORNMP_SERVER", &server_ip);
    
    inject_dll(pid)?;
    
    println!("✅ Инжект успешен!");
    Ok(())
}

/// Поиск PID процесса GTA5.exe (публичная функция)
pub fn find_gta5_pid() -> Option<u32> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return None;
        }
        
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        
        if Process32FirstW(snap, &mut entry) == 1 {
            loop {
                let mut len = 0;
                for i in 0..entry.szExeFile.len() {
                    if entry.szExeFile[i] == 0 {
                        break;
                    }
                    len += 1;
                }
                
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

/// Инжект DLL в процесс
fn inject_dll(pid: u32) -> Result<()> {
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
    if handle.is_null() {
        return Err(anyhow!("Не удалось открыть процесс. Запустите лаунчер от администратора!"));
    }
    println!("✅ Процесс открыт");
    
    // Ищем client.dll
    let dll_paths = ["client.dll", "./client.dll", "./target/release/client.dll"];
    let mut dll_data = None;
    
    for path in dll_paths {
        if let Ok(data) = std::fs::read(path) {
            dll_data = Some(data);
            println!("✅ Найдена DLL по пути: {}", path);
            break;
        }
    }
    
    let dll_data = dll_data.ok_or_else(|| anyhow!("client.dll не найдена!"))?;
    println!("📦 DLL размер: {} байт", dll_data.len());
    
    // Выделяем память
    let remote_mem = unsafe {
        VirtualAllocEx(handle, ptr::null_mut(), dll_data.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
    };
    if remote_mem.is_null() {
        return Err(anyhow!("Не удалось выделить память в GTA V"));
    }
    println!("📦 Память выделена: {:p}", remote_mem);
    
    // Копируем DLL
    let mut bytes_written = 0;
    unsafe {
        WriteProcessMemory(handle, remote_mem, dll_data.as_ptr() as _, dll_data.len(), &mut bytes_written);
    }
    println!("📦 DLL скопирована ({} байт)", bytes_written);
    
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
    
    println!("💉 Инжект успешен!");
    Ok(())
}