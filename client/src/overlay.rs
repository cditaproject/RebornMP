// client/src/overlay.rs
// Простая библиотека для оверлея чата (без Mutex)

use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::shared::windef::*;
use std::ptr;
use std::cell::RefCell;

thread_local! {
    static OVERLAY_DATA: RefCell<OverlayState> = RefCell::new(OverlayState {
        hwnd: ptr::null_mut(),
        hdc: ptr::null_mut(),
        font: ptr::null_mut(),
        initialized: false,
    });
}

struct OverlayState {
    hwnd: HWND,
    hdc: HDC,
    font: HFONT,
    initialized: bool,
}

pub struct SimpleOverlay;

impl SimpleOverlay {
    pub fn new() -> Self {
        SimpleOverlay
    }
    
    pub fn init(&self) -> bool {
        OVERLAY_DATA.with(|data| {
            let mut state = data.borrow_mut();
            if state.initialized {
                return true;
            }
            
            unsafe {
                let hwnd = FindWindowA(ptr::null(), b"Grand Theft Auto V\0".as_ptr() as *const i8);
                if hwnd.is_null() {
                    println!("[Overlay] GTA V window not found");
                    return false;
                }
                
                state.hwnd = hwnd;
                state.hdc = GetDC(hwnd);
                
                state.font = CreateFontA(
                    16, 0, 0, 0, FW_NORMAL, 0, 0, 0,
                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                    DEFAULT_QUALITY, DEFAULT_PITCH,
                    b"Verdana\0".as_ptr() as *const i8
                );
                
                state.initialized = true;
                println!("[Overlay] Initialized successfully");
                true
            }
        })
    }
    
    pub fn text(&self, text: &str, x: i32, y: i32, color: u32) {
        OVERLAY_DATA.with(|data| {
            let state = data.borrow();
            if !state.initialized { return; }
            
            unsafe {
                let old_font = SelectObject(state.hdc, state.font as HGDIOBJ);
                SetTextColor(state.hdc, color);
                SetBkMode(state.hdc, 1); // TRANSPARENT
                TextOutA(state.hdc, x, y, text.as_ptr() as *const i8, text.len() as i32);
                SelectObject(state.hdc, old_font);
            }
        });
    }
    
    pub fn rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        OVERLAY_DATA.with(|data| {
            let state = data.borrow();
            if !state.initialized { return; }
            
            unsafe {
                let brush = CreateSolidBrush(color);
                let old_brush = SelectObject(state.hdc, brush as HGDIOBJ);
                Rectangle(state.hdc, x, y, x + w, y + h);
                SelectObject(state.hdc, old_brush);
                DeleteObject(brush as HGDIOBJ);
            }
        });
    }
    
    pub fn clear_area(&self, x: i32, y: i32, w: i32, h: i32) {
        OVERLAY_DATA.with(|data| {
            let state = data.borrow();
            if !state.initialized { return; }
            
            unsafe {
                let brush = CreateSolidBrush(RGB(0, 0, 0));
                let old_brush = SelectObject(state.hdc, brush as HGDIOBJ);
                Rectangle(state.hdc, x, y, x + w, y + h);
                SelectObject(state.hdc, old_brush);
                DeleteObject(brush as HGDIOBJ);
            }
        });
    }
}
