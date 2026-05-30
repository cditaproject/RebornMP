use std::ptr;

// Структура для перехвата функций (как в FiveM)
pub struct Hook {
    target: *mut u8,
    detour: *mut u8,
    original: *mut u8,
}

impl Hook {
    pub fn new(target: *mut u8, detour: *mut u8) -> Self {
        Hook {
            target,
            detour,
            original: ptr::null_mut(),
        }
    }
    
    pub fn enable(&mut self) {
        unsafe {
            // Сохраняем оригинальные байты
            let mut old_protect = 0;
            winapi::um::memoryapi::VirtualProtect(
                self.target as _,
                14, // JMP инструкция
                winapi::um::winnt::PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            );
            
            // Записываем JMP на нашу функцию
            let jmp = create_jmp_instruction(self.detour);
            ptr::copy_nonoverlapping(jmp.as_ptr(), self.target, jmp.len());
        }
    }
}

fn create_jmp_instruction(target: *mut u8) -> Vec<u8> {
    let offset = (target as i64 - (self.target as i64 + 5)) as i32;
    let mut jmp = vec![0xE9]; // JMP opcode
    jmp.extend_from_slice(&offset.to_le_bytes());
    jmp
}