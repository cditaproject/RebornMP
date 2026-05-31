use winapi::um::memoryapi::{VirtualProtect, VirtualQueryEx, ReadProcessMemory};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{PROCESS_ALL_ACCESS, PAGE_EXECUTE_READWRITE, MEM_COMMIT, MEMORY_BASIC_INFORMATION};
use winapi::shared::ntdef::HANDLE;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ptr;

static PLAYER_PTR_ADDR: AtomicUsize = AtomicUsize::new(0);
static HEALTH_ADDR: AtomicUsize = AtomicUsize::new(0);
pub static OFFSET_POSITION: isize = 0x90;

pub fn init() {
    println!("[Memory] 🔍 Scanning for GTA V addresses...");
    
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() {
            println!("[Memory] ❌ Failed to open process");
            return;
        }
        
        let player_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00];
        if let Some(addr) = find_pattern(&process, &player_sig) {
            let offset = *(addr.offset(3) as *const i32);
            let player_ptr = (addr as usize + 7 + offset as usize) as *mut *mut u8;
            PLAYER_PTR_ADDR.store(player_ptr as usize, Ordering::Relaxed);
            println!("[Memory] ✅ Found player pointer at: 0x{:X}", player_ptr as usize);
        } else {
            println!("[Memory] ⚠️ Player pointer signature not found");
        }
        
        let health_sig = [0x48, 0x8B, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x48, 0x85, 0xC9, 0x74, 0x00, 0x8B, 0x81];
        if let Some(addr) = find_pattern(&process, &health_sig) {
            let offset = *(addr.offset(3) as *const i32);
            let health_ptr = (addr as usize + 7 + offset as usize + 0x280) as *mut u32;
            HEALTH_ADDR.store(health_ptr as usize, Ordering::Relaxed);
            println!("[Memory] ✅ Found health address at: 0x{:X}", health_ptr as usize);
        }
        
        CloseHandle(process);
    }
}

fn find_pattern(process: &HANDLE, pattern: &[u8]) -> Option<*mut u8> {
    unsafe {
        let mut addr: *mut u8 = 0x400000 as *mut u8;
        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        
        while VirtualQueryEx(*process, addr as _, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
            if mbi.State == MEM_COMMIT && (mbi.Protect & PAGE_EXECUTE_READWRITE) != 0 {
                let mut buffer = vec![0u8; mbi.RegionSize];
                let mut bytes_read = 0;
                
                if ReadProcessMemory(*process, mbi.BaseAddress, buffer.as_mut_ptr() as _, buffer.len(), &mut bytes_read) != 0 {
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
        println!("⚠️ Player pointer not found, retry scanning...");
        init();
        let addr = PLAYER_PTR_ADDR.load(Ordering::Relaxed);
        if addr == 0 { return std::ptr::null_mut(); }
        unsafe { *((addr) as *mut *mut u8) }
    } else {
        unsafe { *((addr) as *mut *mut u8) }
    }
}

pub fn disable_story() {
    println!("📖 Disabling story mode...");
    
    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, std::process::id());
        if process.is_null() { 
            println!("❌ Failed to open process");
            return; 
        }
        
        let story_sig = [0x80, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x74, 0x00];
        if let Some(addr) = find_pattern(&process, &story_sig) {
            let offset = *(addr.offset(2) as *const i32);
            let story_addr = (addr as usize + 6 + offset as usize) as *mut u8;
            let mut old = 0;
            VirtualProtect(story_addr as _, 1, PAGE_EXECUTE_READWRITE, &mut old);
            *story_addr = 0;
            VirtualProtect(story_addr as _, 1, old, &mut old);
            println!("✅ Story flag cleared at 0x{:X}", story_addr as usize);
        } else {
            println!("⚠️ Story flag signature not found");
        }
        
        CloseHandle(process);
    }
}

pub fn teleport_to_spawn() {
    println!("📍 Attempting to teleport...");
    
    for attempt in 1..=30 {
        let player = get_player_ptr();
        if !player.is_null() {
            unsafe {
                let pos_addr = player.offset(OFFSET_POSITION) as *mut [f32; 3];
                let mut old = 0;
                VirtualProtect(pos_addr as _, 12, PAGE_EXECUTE_READWRITE, &mut old);
                *pos_addr = [-1038.5, -2745.0, 20.0];
                VirtualProtect(pos_addr as _, 12, old, &mut old);
                println!("✅ Teleported to LS Airport! (attempt {})", attempt);
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    
    println!("⚠️ Teleport failed after 6 seconds - player not ready");
}

pub fn get_health() -> u32 {
    let addr = HEALTH_ADDR.load(Ordering::Relaxed);
    if addr == 0 { return 0; }
    unsafe { *(addr as *mut u32) }
}

pub fn write_memory(addr: *mut u8, bytes: &[u8]) {
    unsafe {
        let mut old_protect = 0;
        VirtualProtect(addr as _, bytes.len(), PAGE_EXECUTE_READWRITE, &mut old_protect);
        ptr::copy_nonoverlapping(bytes.as_ptr(), addr, bytes.len());
        VirtualProtect(addr as _, bytes.len(), old_protect, &mut old_protect);
    }
}