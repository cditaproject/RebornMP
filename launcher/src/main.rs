use eframe::egui;
use egui::{Color32, RichText};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

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

struct RebornMPLauncher {
    config: Config,
    status: String,
    gta_path: String,
}

impl Default for RebornMPLauncher {
    fn default() -> Self {
        let config = Self::load_config();
        Self {
            status: "Готов к работе".to_string(),
            gta_path: find_gta5_dir(),
            config,
        }
    }
}

fn find_gta5_dir() -> String {
    let paths = [
        r"C:\Program Files (x86)\Steam\steamapps\common\Grand Theft Auto V",
        r"C:\Program Files\Rockstar Games\Grand Theft Auto V",
        r"D:\Steam\steamapps\common\Grand Theft Auto V",
        r"D:\Rockstar Games\Grand Theft Auto V",
    ];
    for path in paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    "".to_string()
}

fn find_rockstar_launcher() -> Option<String> {
    // Основные пути к Launcher.exe
    let paths = [
        r"C:\Program Files\Rockstar Games\Launcher\Launcher.exe",
        r"C:\Program Files (x86)\Rockstar Games\Launcher\Launcher.exe",
        r"D:\Program Files\Rockstar Games\Launcher\Launcher.exe",
        r"E:\Program Files\Rockstar Games\Launcher\Launcher.exe",
    ];
    
    for path in paths {
        if std::path::Path::new(path).exists() {
            println!("✅ Найден Rockstar Launcher: {}", path);
            return Some(path.to_string());
        }
    }
    
    // Если не нашли - ищем в папке Program Files
    let search_dirs = [
        r"C:\Program Files\Rockstar Games\Launcher",
        r"C:\Program Files (x86)\Rockstar Games\Launcher",
        r"D:\Program Files\Rockstar Games\Launcher",
    ];
    
    for dir in search_dirs {
        if std::path::Path::new(dir).exists() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("exe") {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        if name.to_lowercase() == "launcher.exe" {
                            println!("✅ Найден Rockstar Launcher: {:?}", path);
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    println!("❌ Rockstar Launcher не найден!");
    None
}

impl RebornMPLauncher {
    fn load_config() -> Config {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("RebornMP").join("config.json");
            if config_path.exists() {
                if let Ok(data) = fs::read_to_string(config_path) {
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
            let _ = fs::create_dir_all(&config_path);
            let config_file = config_path.join("config.json");
            if let Ok(data) = serde_json::to_string_pretty(&self.config) {
                let _ = fs::write(config_file, data);
            }
        }
    }
    
    fn install_mod(&self) -> Result<(), String> {
        if self.gta_path.is_empty() {
            return Err("GTA V не найдена! Укажите путь вручную.".to_string());
        }
        
        // Сохраняем конфиг для мода
        let mod_config = serde_json::json!({
            "server_ip": self.config.server_ip
        });
        
        let config_path = PathBuf::from(&self.gta_path).join("rebornmp_config.json");
        let config_data = serde_json::to_string_pretty(&mod_config).map_err(|e| e.to_string())?;
        fs::write(&config_path, config_data).map_err(|e| e.to_string())?;
        println!("✅ Конфиг сохранён: {:?}", config_path);
        
        // Копируем d3d11.dll (прокси)
        let proxy_src = "d3d11.dll";
        let proxy_dst = PathBuf::from(&self.gta_path).join("d3d11.dll");
        
        if std::path::Path::new(proxy_src).exists() {
            fs::copy(proxy_src, &proxy_dst).map_err(|e| e.to_string())?;
            println!("✅ Прокси установлен: {:?}", proxy_dst);
        } else {
            return Err("d3d11.dll не найден! Сначала соберите прокси.".to_string());
        }
        
        // Копируем rebornmp.dll (мод)
        let mod_src = "rebornmp.dll";
        let mod_dst = PathBuf::from(&self.gta_path).join("rebornmp.dll");
        
        if std::path::Path::new(mod_src).exists() {
            fs::copy(mod_src, &mod_dst).map_err(|e| e.to_string())?;
            println!("✅ Мод установлен: {:?}", mod_dst);
        } else {
            return Err("rebornmp.dll не найден! Сначала соберите клиент.".to_string());
        }
        
        Ok(())
    }
    
    fn launch_game(&self) -> Result<(), String> {
        let launcher_path = find_rockstar_launcher()
            .ok_or("Rockstar Games Launcher не найден! Установите его с официального сайта.")?;
        
        println!("🚀 Запуск Rockstar Launcher: {}", launcher_path);
        
        // Запускаем лаунчер с параметром для запуска GTA V
        let result = Command::new(&launcher_path)
            .arg("--launch")
            .arg("GTA5")
            .spawn();
        
        match result {
            Ok(_) => {
                println!("✅ GTA V запускается через Rockstar Launcher");
                Ok(())
            }
            Err(e) => {
                // Пробуем без параметров
                println!("Пробуем запустить без параметров...");
                Command::new(&launcher_path)
                    .spawn()
                    .map_err(|e2| format!("Ошибка запуска: {} / {}", e, e2))?;
                println!("✅ Rockstar Launcher запущен. Запустите GTA V вручную.");
                Ok(())
            }
        }
    }
    
    fn start(&mut self) {
        self.status = "📦 Установка мода...".to_string();
        
        if let Err(e) = self.install_mod() {
            self.status = format!("❌ {}", e);
            return;
        }
        
        self.status = "✅ Мод установлен! Запуск Rockstar Launcher...".to_string();
        self.save_config();
        
        // Запускаем игру в отдельном потоке
        let launcher_status = self.status.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            match RebornMPLauncher::launch_game_static() {
                Ok(_) => println!("✅ Игра запущена"),
                Err(e) => println!("❌ {}", e),
            }
        });
        
        self.status = "✅ Rockstar Launcher запущен! Мод загрузится автоматически.".to_string();
    }
    
    fn launch_game_static() -> Result<(), String> {
        let launcher_path = find_rockstar_launcher()
            .ok_or("Rockstar Games Launcher не найден!")?;
        
        Command::new(&launcher_path)
            .arg("--launch")
            .arg("GTA5")
            .spawn()
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}

impl eframe::App for RebornMPLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                
                // Поле для IP сервера
                ui.group(|ui| {
                    ui.label(RichText::new("🌐 СЕРВЕР").strong());
                    ui.horizontal(|ui| {
                        ui.label("IP:");
                        ui.text_edit_singleline(&mut self.config.server_ip);
                    });
                    ui.label(RichText::new("IP адрес сервера для подключения").size(11.0));
                });
                ui.add_space(10.0);
                
                // Путь к GTA V
                ui.group(|ui| {
                    ui.label(RichText::new("📁 ПУТЬ К GTA V").strong());
                    ui.text_edit_singleline(&mut self.gta_path);
                    ui.label(RichText::new("Путь к папке с GTA5.exe").size(11.0));
                });
                ui.add_space(20.0);
                
                // Статус
                ui.colored_label(Color32::GREEN, &self.status);
                
                // Кнопка запуска
                if ui.button("🚀 УСТАНОВИТЬ И ЗАПУСТИТЬ").clicked() {
                    self.start();
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.label(RichText::new("📌 ИНСТРУКЦИЯ").strong());
                ui.label("1. Нажмите кнопку для установки мода");
                ui.label("2. Лаунчер скопирует файлы в папку с GTA V");
                ui.label("3. Автоматически запустится Rockstar Games Launcher");
                ui.label("4. GTA V запустится и мод загрузится");
                ui.label("");
                ui.label(RichText::new("⚠️ Если игра не запускается - запустите Rockstar Launcher вручную").weak());
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 520.0])
            .with_min_inner_size([500.0, 480.0])
            .with_resizable(true)
            .with_title("RebornMP Launcher"),
        ..Default::default()
    };
    
    eframe::run_native(
        "RebornMP Launcher",
        options,
        Box::new(|_cc| Box::new(RebornMPLauncher::default())),
    ).unwrap();
}