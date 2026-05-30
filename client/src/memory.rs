use std::ptr;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::libloaderapi::GetModuleHandleA;

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Адреса для GTA V 1.0.3788.0
pub const OFFSET_PLAYER_PTR: usize = 0x2A1F2F48;  // Указатель на игрока
pub const OFFSET_POSITION: isize = 0x90;          // Смещение координат
pub const STORY_FLAG_ADDR: usize = 0x2A1F2F50;    // Флаг сюжета

pub fn write_memory<T>(addr: *mut T, value: T) {
    unsafe {
        let mut old_protect = 0;
        VirtualProtect(
            addr as _,
            std::mem::size_of::<T>(),
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );
        ptr::write(addr, value);
        VirtualProtect(addr as _, std::mem::size_of::<T>(), old_protect, &mut old_protect);
    }
}

pub fn get_player_ptr() -> *mut u8 {
    unsafe {
        let base = GetModuleHandleA("GTA5.exe\0".as_ptr() as _);
        let ptr_addr = (base as usize + OFFSET_PLAYER_PTR) as *mut *mut u8;
        *ptr_addr
    }
}

pub fn disable_story() {
    unsafe {
        let base = GetModuleHandleA("GTA5.exe\0".as_ptr() as _);
        let story_addr = (base as usize + STORY_FLAG_ADDR) as *mut u8;
        if !story_addr.is_null() {
            write_memory(story_addr, 0);
            println!("✅ Story mode disabled!");
        }
    }
}

pub fn teleport_to_spawn() {
    unsafe {
        let player = get_player_ptr();
        if !player.is_null() {
            let pos = Vector3 { x: -1038.5, y: -2745.0, z: 20.0 };
            let pos_addr = player.offset(OFFSET_POSITION) as *mut Vector3;
            write_memory(pos_addr, pos);
            println!("✅ Teleported to LS Airport!");
        }
    }
}