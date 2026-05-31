use winapi::um::memoryapi::{VirtualProtect, VirtualQueryEx, ReadProcessMemory};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{PROCESS_ALL_ACCESS, PAGE_EXECUTE_READWRITE, MEM_COMMIT, MEMORY_BASIC_INFORMATION};
use winapi::shared::ntdef::HANDLE;
use std::sync::atomic::{AtomicUsize, Ordering};

// Найденные адреса (заполнятся при сканировании)
static PLAYER_PTR_ADDR: AtomicUsize = AtomicUsize::new(0);
static STORY_FLAG_ADDR: AtomicUsize = AtomicUsize::new(0);
pub static OFFSET_POSITION: isize = 0x90;  // Смещение координат (редко меняется)

pub fn init() {
    println!("[Memory] Scanning for GTA V addresses using FiveM signatures...");
    
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() {
            println!("[Memory] Failed to open process");
            return;
        }
        
        // Сигнатура из FiveM для указателя на игрока (CPlayerInfo)
        let player_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00];
        if let Some(addr) = find_pattern(process, &player_sig) {
            let offset = *(addr.offset(3) as *const i32);
            let player_ptr = (addr as usize + 7 + offset as usize) as *mut *mut u8;
            PLAYER_PTR_ADDR.store(player_ptr as usize, Ordering::Relaxed);
            println!("[Memory] Found player pointer at: 0x{:X}", player_ptr as usize);
        } else {
            println!("[Memory] Player pointer signature not found");
        }
        
        // Сигнатура для флага сюжета (отключает миссии)
        let story_sig = [0x80, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x74, 0x00];
        if let Some(addr) = find_pattern(process, &story_sig) {
            let offset = *(addr.offset(2) as *const i32);
            let story_addr = (addr as usize + 6 + offset as usize) as *mut u8;
            STORY_FLAG_ADDR.store(story_addr as usize, Ordering::Relaxed);
            println!("[Memory] Found story flag at: 0x{:X}", story_addr as usize);
        } else {
            println!("[Memory] Story flag signature not found");
        }
        
        CloseHandle(process);
    }
}
pub fn exit_story_mode() {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() {
            return;
        }
        
        // Сигнатура для переключения режима игры (из FiveM)
        let mode_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x80, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x00];
        if let Some(addr) = find_pattern(process, &mode_sig) {
            // Переключаем в свободный режим
            let mode_ptr = (addr as usize + 0x10) as *mut u8;
            *mode_ptr = 1;
            println!("✅ Switched to free mode!");
        }
        
        CloseHandle(process);
    }
}
fn find_pattern(process: HANDLE, pattern: &[u8]) -> Option<*mut u8> {
    unsafe {
        let mut addr: *mut u8 = 0x400000 as *mut u8;
        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        
        while VirtualQueryEx(process, addr as _, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
            if mbi.State == MEM_COMMIT {
                let mut buffer = vec![0u8; mbi.RegionSize];
                let mut bytes_read = 0;
                
                if ReadProcessMemory(process, mbi.BaseAddress, buffer.as_mut_ptr() as _, buffer.len(), &mut bytes_read) != 0 {
                    'search: for i in 0..bytes_read.saturating_sub(pattern.len()) {
                        for j in 0..pattern.len() {
                            if pattern[j] != 0x00 && buffer[i + j] != pattern[j] {
                                continue 'search;
                            }
                        }
                        return Some((mbi.BaseAddress as usize + i) as *mut u8);
                    }
                }
            }
            addr = (mbi.BaseAddress as usize + mbi.RegionSize) as *mut u8;
        }
        None
    }
}

pub fn get_player_ptr() -> *mut u8 {
    let addr = PLAYER_PTR_ADDR.load(Ordering::Relaxed);
    if addr == 0 { 
        println!("⚠️ Player pointer not found, use Cheat Engine to find it");
        return std::ptr::null_mut();
    }
    unsafe {
        *((addr) as *mut *mut u8)
    }
}
pub fn write_memory(addr: *mut u8, bytes: &[u8]) {
    unsafe {
        let mut old_protect = 0;
        VirtualProtect(
            addr as _,
            bytes.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr, bytes.len());
        VirtualProtect(addr as _, bytes.len(), old_protect, &mut old_protect);
    }
}
pub fn full_story_bypass() {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() { return; }
        
        // 1. Отключаем флаг активного сюжета
        let story_active_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00];
        if let Some(addr) = find_pattern(process, &story_active_sig) {
            let offset = *(addr.offset(3) as *const i32);
            let flag_ptr = (addr as usize + 7 + offset as usize) as *mut u8;
            *flag_ptr = 0;
            println!("✅ Story active flag cleared!");
        }
        
        // 2. Патчим функцию запуска миссий
        let mission_start_sig = [0x40, 0x53, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x8B, 0xD9, 0xE8];
        if let Some(addr) = find_pattern(process, &mission_start_sig) {
            // Заменяем на `ret` (функция ничего не делает)
            let patch = [0xC3, 0x90, 0x90, 0x90];
            write_memory(addr, &patch);
            println!("✅ Mission start patched!");
        }
        
        // 3. Отключаем загрузку сюжетных скриптов
        let script_load_sig = [0x48, 0x89, 0x5C, 0x24, 0x08, 0x48, 0x89, 0x74, 0x24, 0x10, 0x57];
        if let Some(addr) = find_pattern(process, &script_load_sig) {
            write_memory(addr, &[0xC3, 0x90]);
            println!("✅ Story scripts disabled!");
        }
        
        CloseHandle(process);
    }
}
pub fn disable_story() {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() { return; }

        // Надёжный патч из FiveM для переключения в свободный режим
        let pattern = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00];
        
        if let Some(addr) = find_pattern(process, &pattern) {
            // Получаем адрес указателя на глобальный флаг
            let offset = *(addr.offset(3) as *const i32);
            let flag_ptr = (addr as usize + 7 + offset as usize) as *mut u8;
            
            // Меняем значение на 0 (свободный режим)
            *flag_ptr = 0;
            println!("✅ Free mode enabled!");
        }
        CloseHandle(process);
    }
}
pub fn force_free_mode() {
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() {
            return;
        }
        
        // Сигнатура для флага, который переключает режим игры
        let mode_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x80, 0xB9, 0x00, 0x00, 0x00, 0x00, 0x00];
        if let Some(addr) = find_pattern(process, &mode_sig) {
            // Принудительно включаем свободный режим
            let mode_ptr = (addr as usize + 0x10) as *mut u8;
            *mode_ptr = 1;
            println!("✅ Free mode forced!");
        }
        
        CloseHandle(process);
    }
}
pub fn teleport_to_spawn() {
    println!("📍 Attempting to teleport...");
    
    // Ждём, пока указатель на игрока станет доступным (до 5 секунд)
    for attempt in 1..=50 {
        unsafe {
            let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
            if process.is_null() {
                continue;
            }
            
            let player_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00];
            if let Some(addr) = find_pattern(process, &player_sig) {
                let offset = *(addr.offset(3) as *const i32);
                let player_ptr_addr = (addr as usize + 7 + offset as usize) as *mut *mut u8;
                let player = *player_ptr_addr;
                
                if !player.is_null() {
                    let pos_addr = player.offset(OFFSET_POSITION) as *mut [f32; 3];
                    let mut old = 0;
                    VirtualProtect(pos_addr as _, 12, PAGE_EXECUTE_READWRITE, &mut old);
                    *pos_addr = [-1038.5, -2745.0, 20.0];
                    VirtualProtect(pos_addr as _, 12, old, &mut old);
                    println!("✅ Teleported to LS Airport! (attempt {})", attempt);
                    CloseHandle(process);
                    return;
                }
            }
            CloseHandle(process);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("⚠️ Teleport failed after 5 seconds - player not ready");
}