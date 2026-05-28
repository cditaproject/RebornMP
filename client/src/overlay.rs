// Простая библиотека для оверлея чата
// Не требует ImGui, только Win32 API

use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use std::ptr;
use std::sync::Mutex;

pub struct GameOverlay {
    hwnd: Mutex<Option<HWND>>,
    hdc: Mutex<Option<HDC>>,
    font: Mutex<Option<HFONT>>,
}

impl GameOverlay {
    pub fn new() -> Self {
        GameOverlay {
            hwnd: Mutex::new(None),
            hdc: Mutex::new(None),
            font: Mutex::new(None),
        }
    }
    
    pub fn attach_to_game(&self) -> bool {
        unsafe {
            let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
            if hwnd.is_null() {
                return false;
            }
            
            *self.hwnd.lock().unwrap() = Some(hwnd);
            *self.hdc.lock().unwrap() = Some(GetDC(hwnd));
            
            let font = CreateFontA(
                18, 0, 0, 0, FW_BOLD, 0, 0, 0,
                DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY, DEFAULT_PITCH, 
                b"Verdana\0".as_ptr() as *const i8
            );
            *self.font.lock().unwrap() = Some(font);
            
            true
        }
    }
    
    pub fn draw_text(&self, text: &str, x: i32, y: i32, r: u8, g: u8, b: u8) {
        unsafe {
            if let (Some(hwnd), Some(hdc), Some(font)) = (
                *self.hwnd.lock().unwrap(),
                *self.hdc.lock().unwrap(),
                *self.font.lock().unwrap()
            ) {
                let old_font = SelectObject(hdc, font as HGDIOBJ);
                SetTextColor(hdc, RGB(r as u32, g as u32, b as u32));
                SetBkMode(hdc, TRANSPARENT);
                
                TextOutA(hdc, x, y, text.as_ptr() as *const i8, text.len() as i32);
                
                SelectObject(hdc, old_font);
            }
        }
    }
    
    pub fn draw_rounded_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
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
    
    pub fn begin_frame(&self) {
        unsafe {
            if let (Some(hwnd), Some(hdc)) = (*self.hwnd.lock().unwrap(), *self.hdc.lock().unwrap()) {
                // Очищаем область чата
                let rect = RECT { left: 0, top: 0, right: 400, bottom: 500 };
                FillRect(hdc, &rect, GetStockObject(BLACK_BRUSH) as HBRUSH);
            }
        }
    }
    
    pub fn end_frame(&self) {
        // Ничего не делаем
    }
}
