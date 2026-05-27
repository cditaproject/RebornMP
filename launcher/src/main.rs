// ============================================
// RebornMP Launcher - С выбором пути к GTA V
// ============================================

use eframe::egui;
use egui::{Color32, RichText};
use std::sync::mpsc;
use std::thread;
use std::path::PathBuf;
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;

mod injector;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    gta5_path: String,
    server_ip: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gta5_path: "".to_string(),
            server_ip: "127.0.0.1:30120".to_string(),
        }
    }
}

struct RebornMPLauncher {
    state: LauncherState,
    server_ip: String,
    gta5_path: String,
    status_text: String,
    _config: Config,
}

#[derive(Clone)]
enum LauncherState {
    Idle,
    Launching,
    Success,
    Error(String),
}

impl Default for RebornMPLauncher {
    fn default() -> Self {
        let config = Self::load_config();
        let mut app = Self {
            state: LauncherState::Idle,
            server_ip: config.server_ip.clone(),
            gta5_path: config.gta5_path.clone(),
            status_text: "Готов к работе".to_string(),
            _config: config,
        };
        
        // Автоматически ищем GTA V если путь не задан
        if app.gta5_path.is_empty() {
            if let Some(found_path) = injector::find_gta5_exe_auto() {
                app.gta5_path = found_path.to_string_lossy().to_string();
                app.save_config();
                app.status_text = format!("Найдена GTA V: {}", app.gta5_path);
            } else {
                app.status_text = "Не найдена GTA V. Укажите путь вручную.".to_string();
            }
        } else {
            app.status_text = format!("Путь к GTA V: {}", app.gta5_path);
        }
        
        app
    }
}

impl RebornMPLauncher {
    fn load_config() -> Config {
        if let Some(proj_dirs) = ProjectDirs::from("com", "rebornmp", "launcher") {
            let config_path = proj_dirs.config_dir().join("config.json");
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
        let config = Config {
            gta5_path: self.gta5_path.clone(),
            server_ip: self.server_ip.clone(),
        };
        
        if let Some(proj_dirs) = ProjectDirs::from("com", "rebornmp", "launcher") {
            if let Err(e) = std::fs::create_dir_all(proj_dirs.config_dir()) {
                eprintln!("Failed to create config dir: {}", e);
                return;
            }
            let config_path = proj_dirs.config_dir().join("config.json");
            if let Ok(data) = serde_json::to_string_pretty(&config) {
                let _ = std::fs::write(config_path, data);
            }
        }
    }
    
    fn select_gta5_path(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("GTA5 Executable", &["exe"])
            .set_title("Выберите GTA5.exe")
            .pick_file() 
        {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().to_lowercase() == "gta5.exe" {
                    self.gta5_path = path.to_string_lossy().to_string();
                    self.status_text = format!("Выбран путь: {}", self.gta5_path);
                    self.save_config();
                } else {
                    self.status_text = "Ошибка: выберите файл GTA5.exe".to_string();
                }
            }
        }
    }
    
    fn launch(&mut self) {
        if self.gta5_path.is_empty() {
            self.state = LauncherState::Error("Не указан путь к GTA5.exe!".to_string());
            self.status_text = "Ошибка: укажите путь к GTA V".to_string();
            return;
        }
        
        // Проверяем существование файла
        let path = PathBuf::from(&self.gta5_path);
        if !path.exists() {
            self.state = LauncherState::Error("GTA5.exe не найден!".to_string());
            self.status_text = format!("❌ Файл не существует: {}", self.gta5_path);
            return;
        }
        
        let gta_path = self.gta5_path.clone();
        let server_ip = self.server_ip.clone();
        
        self.state = LauncherState::Launching;
        self.status_text = "Запуск... Смотрите консоль для деталей".to_string();
        
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = injector::start_with_path(server_ip, gta_path);
            tx.send(result).unwrap();
        });
        
        match rx.recv().unwrap() {
            Ok(_) => {
                self.state = LauncherState::Success;
                self.status_text = "✅ Инжект успешен! Игра запущена.".to_string();
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                self.state = LauncherState::Error(error_msg.clone());
                self.status_text = format!("❌ Ошибка: {}", error_msg);
            }
        }
    }
}

impl eframe::App for RebornMPLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Стилизация окна
        let mut style = (*ctx.style()).clone();
        style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 35);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(45, 45, 50);
        ctx.set_style(style);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Заголовок
                ui.heading(RichText::new("🎮 RebornMP Launcher")
                    .size(28.0)
                    .color(Color32::from_rgb(255, 100, 100)));
                ui.label(RichText::new("Custom Multiplayer for GTA V").size(14.0));
                ui.add_space(20.0);
                
                // Путь к GTA V
                ui.group(|ui| {
                    ui.label(RichText::new("📁 ПУТЬ К GTA V").strong());
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.gta5_path);
                        if ui.button("📂 Обзор").clicked() {
                            self.select_gta5_path();
                        }
                    });
                    ui.label(RichText::new("Выберите GTA5.exe из папки с игрой").size(11.0));
                });
                ui.add_space(10.0);
                
                // Сервер
                ui.group(|ui| {
                    ui.label(RichText::new("🌐 СЕРВЕР").strong());
                    ui.horizontal(|ui| {
                        ui.label("IP:");
                        ui.text_edit_singleline(&mut self.server_ip);
                    });
                    ui.label(RichText::new("Пример: 127.0.0.1:30120").size(11.0));
                });
                ui.add_space(20.0);
                
                // Статус и кнопка
                match &self.state {
                    LauncherState::Idle => {
                        ui.colored_label(Color32::GREEN, format!("✅ {}", self.status_text));
                        if ui.button("🚀 ЗАПУСТИТЬ GTA V").clicked() {
                            self.launch();
                        }
                    }
                    LauncherState::Launching => {
                        ui.colored_label(Color32::YELLOW, format!("⏳ {}", self.status_text));
                        ui.spinner();
                        ui.label("Инжектируем DLL...");
                    }
                    LauncherState::Success => {
                        ui.colored_label(Color32::GREEN, format!("✅ {}", self.status_text));
                        ui.label("Клиент загружен! Можно играть.");
                        if ui.button("🔄 НОВЫЙ ЗАПУСК").clicked() {
                            self.state = LauncherState::Idle;
                            self.status_text = "Готов к работе".to_string();
                        }
                    }
                    LauncherState::Error(msg) => {
                        ui.colored_label(Color32::RED, format!("❌ {}", msg));
                        if ui.button("🔄 ПОВТОРИТЬ").clicked() {
                            self.state = LauncherState::Idle;
                            self.status_text = "Готов к работе".to_string();
                        }
                    }
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.label(RichText::new("📌 Информация").strong());
                ui.label("• Запускайте лаунчер от имени Администратора");
                ui.label("• client.dll должна быть рядом с лаунчером");
                ui.label("• Путь сохраняется автоматически");
                ui.label("• Steam версия: C:\\Program Files (x86)\\Steam\\...");
                ui.label("• Rockstar версия: C:\\Program Files\\Rockstar Games\\...");
            });
        });
        
        // Обновляем UI каждые 100 мс
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

// ============================================
// ГЛАВНАЯ ФУНКЦИЯ - ТОЧКА ВХОДА
// ============================================
fn main() -> Result<(), eframe::Error> {
    simple_logger::init_with_level(log::Level::Info).unwrap_or(());
    
    println!("=========================================");
    println!("🎮 RebornMP Launcher v0.1.0");
    println!("=========================================");
    println!("");
    println!("📌 ВАЖНО: Запустите этот лаунчер от имени Администратора!");
    println!("📌 client.dll должна находиться в той же папке, что и launcher.exe");
    println!("");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 550.0])
            .with_min_inner_size([550.0, 500.0])
            .with_resizable(true)
            .with_title("RebornMP Launcher"),
        ..Default::default()
    };
    
    eframe::run_native(
        "RebornMP Launcher",
        options,
        Box::new(|_cc| Box::new(RebornMPLauncher::default())),
    )
}