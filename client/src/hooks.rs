// client/src/hooks.rs
// RebornMP Hooks Manager - DirectX Hooking

use std::sync::atomic::{AtomicBool, Ordering};
use winapi::shared::minwindef::{HMODULE, TRUE, FALSE};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::winuser::{GetAsyncKeyState, VK_T, VK_RETURN, VK_BACK};

static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);

// Тип функции Present из DirectX 11
type PresentFunc = unsafe extern "system" fn(*mut std::ffi::c_void, u32, u32) -> i32;

// Оригинальная функция (будет сохранена)
static mut ORIGINAL_PRESENT: Option<PresentFunc> = None;

// Наш обработчик клавиш
fn handle_keyboard() {
    unsafe {
        // Проверяем клавишу T для открытия чата
        if GetAsyncKeyState(VK_T) & 1 != 0 {
            crate::ui::CHAT.toggle_input();
        }
        
        // Если чат открыт, обрабатываем ввод
        if crate::ui::CHAT.is_input_open() {
            // Enter - отправка сообщения
            if GetAsyncKeyState(VK_RETURN) & 1 != 0 {
                if let Some(msg) = crate::ui::CHAT.send_message() {
                    if msg.starts_with('/') {
                        // Команда
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            net.lock().unwrap().send_command(&msg[1..], "");
                        }
                    } else {
                        // Обычный чат
                        if let Some(net) = unsafe { &crate::NETWORK_CLIENT } {
                            net.lock().unwrap().send_chat(&msg);
                        }
                    }
                }
            }
            
            // Backspace - удаление символа
            if GetAsyncKeyState(VK_BACK) & 1 != 0 {
                crate::ui::CHAT.backspace();
            }
            
            // TODO: Обработка обычных символов
            // В реальном хуке нужно перехватывать символы через WM_CHAR
        }
    }
}

// Функция-перехватчик для DirectX Present
unsafe extern "system" fn hk_present(device: *mut std::ffi::c_void, sync_interval: u32, flags: u32) -> i32 {
    // Обрабатываем клавиатуру
    handle_keyboard();
    
    // Отрисовываем чат
    crate::ui::CHAT.render();
    
    // Вызываем оригинальную функцию
    if let Some(original) = ORIGINAL_PRESENT {
        return original(device, sync_interval, flags);
    }
    0
}

// Поиск адреса Present в DXGI (DirectX 11)
fn find_present_address() -> Option<*mut std::ffi::c_void> {
    unsafe {
        // Получаем handle dxgi.dll
        let dxgi_dll = GetModuleHandleA(b"dxgi.dll\0".as_ptr() as *const i8);
        if dxgi_dll.is_null() {
            println!("[Hooks] Failed to get dxgi.dll handle");
            return None;
        }
        
        // Для DirectX 11 адрес Present находится в vtable IDXGISwapChain
        // Это сложный поиск, пока используем заглушку
        // В реальном проекте нужно найти точный адрес через pattern scan
        
        println!("[Hooks] dxgi.dll found at {:p}", dxgi_dll);
        Some(dxgi_dll as *mut std::ffi::c_void)
    }
}

// Установка хука (jmp-инъекция)
unsafe fn install_jmp_hook(target: *mut std::ffi::c_void, hook: *mut std::ffi::c_void) -> bool {
    let mut old_protect = 0;
    
    // Меняем защиту памяти для записи
    if VirtualProtect(target, 14, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        println!("[Hooks] VirtualProtect failed");
        return false;
    }
    
    // Машина байты для jmp
    // mov rax, hook_address
    // jmp rax
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x48); // mov rax,
    bytes.push(0xB8);
    let addr = hook as usize;
    bytes.push(addr as u8);
    bytes.push((addr >> 8) as u8);
    bytes.push((addr >> 16) as u8);
    bytes.push((addr >> 24) as u8);
    bytes.push((addr >> 32) as u8);
    bytes.push((addr >> 40) as u8);
    bytes.push((addr >> 48) as u8);
    bytes.push((addr >> 56) as u8);
    bytes.push(0xFF); // jmp
    bytes.push(0xE0); // rax
    
    // Записываем байты
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), target as *mut u8, bytes.len());
    
    // Восстанавливаем защиту
    VirtualProtect(target, 14, old_protect, &mut old_protect);
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
            // Сохраняем оригинальную функцию
            ORIGINAL_PRESENT = Some(std::mem::transmute(present_addr));
            
            // Устанавливаем хук
            if install_jmp_hook(present_addr, hk_present as *mut std::ffi::c_void) {
                HOOKS_INSTALLED.store(true, Ordering::SeqCst);
                println!("[Hooks] DirectX hook installed successfully!");
                crate::ui::CHAT.set_console_mode(false);
                return true;
            }
        }
    }
    
    // Если не удалось установить хук, используем консольный режим
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
    
    // TODO: Восстановить оригинальные байты
}
