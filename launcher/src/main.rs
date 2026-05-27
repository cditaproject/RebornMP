use eframe::egui;
use egui::{Color32, RichText};
use serde::{Serialize, Deserialize};
use std::thread;

mod injector;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    server_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_ip: "127.0.0.1:30120".to_string(),
        }
    }
}

struct RebornMP {
    config: Config,
    status: String,
    state: AppState,
    game_running: bool,
}

#[derive(Clone)]
enum AppState {
    Idle,
    Injecting,
    Success,
    Error(String),
}

impl Default for RebornMP {
    fn default() -> Self {
        let config = Self::load_config();
        Self {
            status: "Готов к работе".to_string(),
            state: AppState::Idle,
            game_running: false,
            config,
        }
    }
}

impl RebornMP {
    fn load_config() -> Config {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("RebornMP").join("config.json");
            if config_path.exists() {
                if let Ok(data) = std::fs::read_to_string(config_path) {
                    if let Ok(config) = serde_json::from_str(&data) {
                        return config;
                    }
                }
            }
        }
        Config::default()
    }
    
    fn save_config(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("RebornMP");
            let _ = std::fs::create_dir_all(&config_path);
            let config_file = config_path.join("config.json");
            if let Ok(data) = serde_json::to_string_pretty(&self.config) {
                let _ = std::fs::write(config_file, data);
            }
        }
    }
    
    fn check_game(&mut self) {
        self.game_running = injector::find_gta5_pid().is_some();
    }
    
    fn inject(&mut self) {
        self.state = AppState::Injecting;
        self.status = "Инжекция...".to_string();
        
        let server_ip = self.config.server_ip.clone();
        
        thread::spawn(move || {
            match injector::inject_to_running(server_ip) {
                Ok(_) => {
                    println!("✅ Инжект успешен!");
                }
                Err(e) => {
                    println!("❌ Ошибка: {}", e);
                }
            }
        });
        
        self.state = AppState::Success;
        self.status = "Мод загружен!".to_string();
    }
}

impl eframe::App for RebornMP {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Проверяем запущена ли игра
        self.check_game();
        
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 35);
        ctx.set_style(style);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                ui.heading(RichText::new("🎮 RebornMP")
                    .size(28.0)
                    .color(Color32::from_rgb(255, 100, 100)));
                ui.label(RichText::new("Multiplayer для GTA V").size(14.0));
                ui.add_space(20.0);
                
                // Статус игры
                if self.game_running {
                    ui.colored_label(Color32::GREEN, "✅ GTA V ЗАПУЩЕНА");
                } else {
                    ui.colored_label(Color32::RED, "❌ GTA V НЕ ЗАПУЩЕНА");
                    ui.label("Сначала запустите GTA V и загрузитесь в сюжетку");
                }
                ui.add_space(10.0);
                
                // Сервер
                ui.group(|ui| {
                    ui.label(RichText::new("🌐 СЕРВЕР").strong());
                    ui.horizontal(|ui| {
                        ui.label("IP:");
                        ui.text_edit_singleline(&mut self.config.server_ip);
                    });
                    ui.label(RichText::new("IP адрес сервера для подключения").size(11.0));
                });
                ui.add_space(20.0);
                
                // Статус и кнопка
                match &self.state {
                    AppState::Idle => {
                        ui.colored_label(Color32::GREEN, format!("✅ {}", self.status));
                        
                        if self.game_running {
                            if ui.button("💉 ИНЖЕКТИРОВАТЬ МОД").clicked() {
                                self.inject();
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("💉 ИНЖЕКТИРОВАТЬ МОД"));
                            ui.label("Запустите GTA V чтобы активировать кнопку");
                        }
                    }
                    AppState::Injecting => {
                        ui.colored_label(Color32::YELLOW, format!("💉 {}", self.status));
                        ui.spinner();
                    }
                    AppState::Success => {
                        ui.colored_label(Color32::GREEN, format!("✅ {}", self.status));
                        if ui.button("🔄 НОВЫЙ ИНЖЕКТ").clicked() {
                            self.state = AppState::Idle;
                            self.status = "Готов к работе".to_string();
                        }
                    }
                    AppState::Error(msg) => {
                        ui.colored_label(Color32::RED, format!("❌ {}", msg));
                        if ui.button("🔄 Повторить").clicked() {
                            self.state = AppState::Idle;
                            self.status = "Готов к работе".to_string();
                        }
                    }
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.label(RichText::new("📌 ИНСТРУКЦИЯ").strong());
                ui.label("1. Запустите GTA V (одиночная игра)");
                ui.label("2. Полностью загрузитесь в сюжетку");
                ui.label("3. Нажмите «ИНЖЕКТИРОВАТЬ МОД»");
                ui.label("");
                ui.label(RichText::new("⚠️ Важно!").strong());
                ui.label("• client.dll должна быть рядом с лаунчером");
                ui.label("• Запускайте лаунчер от Администратора");
            });
        });
        
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 520.0])
            .with_min_inner_size([450.0, 480.0])
            .with_resizable(true)
            .with_title("RebornMP Launcher"),
        ..Default::default()
    };
    
    eframe::run_native(
        "RebornMP Launcher",
        options,
        Box::new(|_cc| Box::new(RebornMP::default())),
    ).unwrap();
}