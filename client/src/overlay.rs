// client/src/overlay.rs
// Простая библиотека для оверлея чата

use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::shared::windef::*;
use std::ptr;
use std::sync::Mutex;

pub struct SimpleOverlay {
    hwnd: Mutex<Option<HWND>>,
    hdc: Mutex<Option<HDC>>,
    font: Mutex<Option<HFONT>>,
}

impl SimpleOverlay {
    pub fn new() -> Self {
        SimpleOverlay {
            hwnd: Mutex::new(None),
            hdc: Mutex::new(None),
            font: Mutex::new(None),
        }
    }
    
    pub fn init(&self) -> bool {
        unsafe {
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if hwnd.is_null() {
                println!("[Overlay] GTA V window not found");
                return false;
            }
            
            *self.hwnd.lock().unwrap() = Some(hwnd);
            *self.hdc.lock().unwrap() = Some(GetDC(hwnd));
            
            let font = CreateFontA(
                16, 0, 0, 0, FW_NORMAL, 0, 0, 0,
                DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY, DEFAULT_PITCH,
                b"Verdana\0".as_ptr() as *const i8
            );
            *self.font.lock().unwrap() = Some(font);
            
            println!("[Overlay] Initialized successfully");
            true
        }
    }
    
    pub fn text(&self, text: &str, x: i32, y: i32, color: u32) {
        unsafe {
            if let (Some(hdc), Some(font)) = (*self.hdc.lock().unwrap(), *self.font.lock().unwrap()) {
                let old_font = SelectObject(hdc, font as HGDIOBJ);
                SetTextColor(hdc, color);
                SetBkMode(hdc, 1); // TRANSPARENT = 1
                TextOutA(hdc, x, y, text.as_ptr() as *const i8, text.len() as i32);
                SelectObject(hdc, old_font);
            }
        }
    }
    
    pub fn rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        unsafe {
            if let Some(hdc) = *self.hdc.lock().unwrap() {
                let brush = CreateSolidBrush(color);
                let old_brush = SelectObject(hdc, brush as HGDIOBJ);
                Rectangle(hdc, x, y, x + w, y + h);
                SelectObject(hdc, old_brush);
                DeleteObject(brush as HGDIOBJ);
            }
        }
    }
    
    pub fn clear_area(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            if let Some(hdc) = *self.hdc.lock().unwrap() {
                let brush = CreateSolidBrush(RGB(0, 0, 0));
                let old_brush = SelectObject(hdc, brush as HGDIOBJ);
                Rectangle(hdc, x, y, x + w, y + h);
                SelectObject(hdc, old_brush);
                DeleteObject(brush as HGDIOBJ);
            }
        }
    }
}
