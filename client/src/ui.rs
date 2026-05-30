use std::sync::Mutex;
use once_cell::sync::Lazy;

pub struct Console {
    pub visible: bool,
    input_buffer: String,
    messages: Vec<String>,
    scroll_to_bottom: bool,
}

impl Console {
    pub fn new() -> Self {
        Self {
            visible: false,
            input_buffer: String::new(),
            messages: vec!["[RebornMP] Console ready. Press F8 to toggle.".to_string()],
            scroll_to_bottom: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn add_message(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
        self.scroll_to_bottom = true;
    }

    pub fn render(&mut self) {
        if !self.visible { return; }

        // Здесь будет отрисовка через ImGui
        // Требует доступа к imgui::Context
    }
}

pub static CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| Mutex::new(Console::new()));