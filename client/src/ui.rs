// client/src/ui.rs
// Используем нашу самописную библиотеку

use crate::overlay::GameOverlay;

lazy_static! {
    pub static ref OVERLAY: GameOverlay = GameOverlay::new();
    pub static ref CHAT: ChatManager = ChatManager::new();
}

impl ChatManager {
    pub fn render(&self) {
        if !OVERLAY.attach_to_game() {
            return;
        }
        
        OVERLAY.begin_frame();
        
        // Фон чата
        OVERLAY.draw_rounded_rect(10, 50, 380, 300, 0x80000000);
        
        let messages = self.messages.lock().unwrap();
        let start = if messages.len() > 12 { messages.len() - 12 } else { 0 };
        
        let mut y = 70;
        for msg in messages.iter().skip(start) {
            let (r, g, b) = if msg.is_system {
                (255, 200, 100)
            } else {
                (100, 255, 100)
            };
            
            let prefix = if msg.is_system { "[SYS]" } else "[CHAT]" };
            let text = format!("{} {}", prefix, msg.text);
            OVERLAY.draw_text(&text, 20, y, r, g, b);
            y += 25;
        }
        
        if *self.is_open.lock().unwrap() {
            let input = self.get_input_text();
            OVERLAY.draw_rounded_rect(10, y + 10, 380, 35, 0xCC000000);
            OVERLAY.draw_text(&format!("> {}", input), 20, y + 20, 255, 255, 255);
        }
        
        OVERLAY.end_frame();
    }
}
