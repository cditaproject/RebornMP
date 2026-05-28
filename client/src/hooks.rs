// client/src/hooks.rs
// RebornMP Hooks Manager - DirectX Hooking

use std::sync::atomic::{AtomicBool, Ordering};
//use winapi::shared::minwindef::{HMODULE, TRUE, FALSE};
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::winuser::{GetAsyncKeyState, VK_RETURN, VK_BACK};
use winapi::ctypes::c_void;

static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);

const VK_T: i32 = 0x54;

type PresentFunc = unsafe extern "system" fn(*mut c_void, u32, u32) -> i32;
static mut ORIGINAL_PRESENT: Option<PresentFunc> = None;

fn handle_keyboard() {
    unsafe {
        if GetAsyncKeyState(VK_T) & 1 != 0 {
            crate::ui::CHAT.toggle_input();
        }
        
        if crate::ui::CHAT.is_input_open() {
            if GetAsyncKeyState(VK_RETURN) & 1 != 0 {
                if let Some(msg) = crate::ui::CHAT.send_message() {
                    if msg.starts_with('/') {
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            let _ = net.lock().unwrap().send_command(&msg[1..], "");
                        }
                    } else {
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            net.lock().unwrap().send_chat(&msg);
                        }
                    }
                }
            }
            
            if GetAsyncKeyState(VK_BACK) & 1 != 0 {
                crate::ui::CHAT.backspace();
            }
        }
    }
}

unsafe extern "system" fn hk_present(device: *mut c_void, sync_interval: u32, flags: u32) -> i32 {
    handle_keyboard();
    crate::ui::CHAT.render();
    
    if let Some(original) = ORIGINAL_PRESENT {
        return original(device, sync_interval, flags);
    }
    0
}

fn find_present_address() -> Option<*mut c_void> {
    unsafe {
        let dxgi_dll = GetModuleHandleA(b"dxgi.dll\0".as_ptr() as *const i8);
        if dxgi_dll.is_null() {
            println!("[Hooks] Failed to get dxgi.dll handle");
            return None;
        }
        
        println!("[Hooks] dxgi.dll found at {:p}", dxgi_dll);
        Some(dxgi_dll as *mut c_void)
    }
}

unsafe fn install_jmp_hook(target: *mut c_void, hook: *mut c_void) -> bool {
    let mut old_protect = 0;
    
    if VirtualProtect(target as *mut _, 14, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        println!("[Hooks] VirtualProtect failed");
        return false;
    }
    
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x48);
    bytes.push(0xB8);
    let addr = hook as usize;
    bytes.extend_from_slice(&addr.to_le_bytes());
    bytes.push(0xFF);
    bytes.push(0xE0);
    
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, bytes.len());
    
    VirtualProtect(target as *mut _, 14, old_protect, &mut old_protect);
    println!("[Hooks] JMP hook installed at {:p}", target);
    true
}

pub fn install_hooks() -> bool {
    if HOOKS_INSTALLED.load(Ordering::SeqCst) {
        println!("[Hooks] Hooks already installed");
        return true;
    }
    
    println!("[Hooks] Installing DirectX hooks...");
    
    unsafe {
        if let Some(present_addr) = find_present_address() {
            ORIGINAL_PRESENT = Some(std::mem::transmute(present_addr));
            
            if install_jmp_hook(present_addr, hk_present as *mut c_void) {
                HOOKS_INSTALLED.store(true, Ordering::SeqCst);
                println!("[Hooks] DirectX hook installed successfully!");
                crate::ui::CHAT.set_console_mode(false);
                return true;
            }
        }
    }
    
    println!("[Hooks] DirectX hook failed, using console mode");
    crate::ui::CHAT.set_console_mode(true);
    true
}

pub fn remove_hooks() {
    if !HOOKS_INSTALLED.load(Ordering::SeqCst) {
        return;
    }
    
    println!("[Hooks] Removing hooks...");
    HOOKS_INSTALLED.store(false, Ordering::SeqCst);
}
