#![allow(non_snake_case)]

use std::ffi::c_void;
use std::sync::Once;
use winapi::shared::minwindef::HINSTANCE;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA, FreeLibrary};
use winapi::um::errhandlingapi::GetLastError;

static mut ORIGINAL_DLL: HINSTANCE = std::ptr::null_mut();
static STARTUP: Once = Once::new();

const ORIGINAL_DLL_PATH: &str = "C:\\Windows\\System32\\d3d11.dll";

#[no_mangle]
pub extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> bool {
    match reason {
        1 => {
            println!("[Proxy] DLL_PROCESS_ATTACH");
            unsafe {
                // Загружаем оригинальную d3d11.dll
                let path = std::ffi::CString::new(ORIGINAL_DLL_PATH).unwrap();
                ORIGINAL_DLL = LoadLibraryA(path.as_ptr());
                
                if ORIGINAL_DLL.is_null() {
                    let error = GetLastError();
                    println!("[Proxy] Failed to load original d3d11.dll: {}", error);
                    return false;
                }
                
                println!("[Proxy] Original d3d11.dll loaded at {:p}", ORIGINAL_DLL);
                
                // Загружаем наш мод
                STARTUP.call_once(|| {
                    let mod_path = std::ffi::CString::new("rebornmp.dll").unwrap();
                    println!("[Proxy] Loading rebornmp.dll...");
                    let mod_handle = LoadLibraryA(mod_path.as_ptr());
                    if !mod_handle.is_null() {
                        println!("[Proxy] ✅ RebornMP loaded successfully at {:p}!", mod_handle);
                    } else {
                        let error = GetLastError();
                        println!("[Proxy] ❌ Failed to load rebornmp.dll: Error {}", error);
                    }
                });
            }
        }
        0 => {
            println!("[Proxy] DLL_PROCESS_DETACH");
            unsafe {
                if !ORIGINAL_DLL.is_null() {
                    FreeLibrary(ORIGINAL_DLL);
                }
            }
        }
        2 => {
            println!("[Proxy] DLL_THREAD_ATTACH");
        }
        3 => {
            println!("[Proxy] DLL_THREAD_DETACH");
        }
        _ => {}
    }
    true
}

#[no_mangle]
pub extern "system" fn D3D11CreateDevice(
    adapter: *mut c_void,
    driver_type: u32,
    software: *mut c_void,
    flags: u32,
    feature_levels: *mut u32,
    feature_levels_count: u32,
    sdk_version: u32,
    device: *mut *mut c_void,
    feature_level: *mut u32,
    context: *mut *mut c_void,
) -> u32 {
    unsafe {
        println!("[Proxy] D3D11CreateDevice called");
        let func: extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut u32, u32, u32, *mut *mut c_void, *mut u32, *mut *mut c_void) -> u32 = 
            std::mem::transmute(GetProcAddress(ORIGINAL_DLL, "D3D11CreateDevice\0".as_ptr() as _));
        func(adapter, driver_type, software, flags, feature_levels, feature_levels_count, sdk_version, device, feature_level, context)
    }
}

#[no_mangle]
pub extern "system" fn D3D11CreateDeviceAndSwapChain(
    adapter: *mut c_void,
    driver_type: u32,
    software: *mut c_void,
    flags: u32,
    feature_levels: *mut u32,
    feature_levels_count: u32,
    sdk_version: u32,
    swap_chain_desc: *mut c_void,
    swap_chain: *mut *mut c_void,
    device: *mut *mut c_void,
    feature_level: *mut u32,
    context: *mut *mut c_void,
) -> u32 {
    unsafe {
        println!("[Proxy] D3D11CreateDeviceAndSwapChain called");
        let func: extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut u32, u32, u32, *mut c_void, *mut *mut c_void, *mut *mut c_void, *mut u32, *mut *mut c_void) -> u32 = 
            std::mem::transmute(GetProcAddress(ORIGINAL_DLL, "D3D11CreateDeviceAndSwapChain\0".as_ptr() as _));
        func(adapter, driver_type, software, flags, feature_levels, feature_levels_count, sdk_version, swap_chain_desc, swap_chain, device, feature_level, context)
    }
}