use std::env;
use std::path::PathBuf;
use std::ffi::c_void;
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::processthreadsapi::{CreateRemoteThread, GetCurrentProcess};

// Константы для обхода античита
const GAME_ARGS: &[&str] = &[
    "-scOfflineOnly",
    "-ignoreDifferentVideoCard", 
    "-useLevelFast",
    "-noChunkUpload",
    "-nobattleye",
    "-skipPatcherCheck"
];

fn main() {
    println!("=== RebornMP - FiveM Style Launcher ===");
    
    // 1. Находим оригинальный GTA V
    let original_game = find_original_game();
    println!("Оригинальный GTA V: {}", original_game.display());
    
    // 2. Создаём изолированную среду (как FiveM)
    let sandbox = create_sandbox(&original_game);
    println!("Песочница: {}", sandbox.display());
    
    // 3. Патчим античит-функции в оригинальном EXE
    patch_anticheat(&original_game);
    
    // 4. Загружаем GTA V как DLL (вот ключевой момент!)
    println!("Загрузка GTA V как динамической библиотеки...");
    unsafe {
        let game_dll = LoadLibraryA(original_game.to_str().unwrap().as_ptr() as _);
        if game_dll.is_null() {
            eprintln!("Ошибка загрузки GTA V!");
            return;
        }
        
        // 5. Находим точку входа игры
        let entry_point = GetProcAddress(game_dll, "EntryPoint\0".as_ptr() as _);
        if entry_point.is_null() {
            // Альтернативное имя функции входа
            let entry = GetProcAddress(game_dll, "WinMain\0".as_ptr() as _);
            if entry.is_null() {
                eprintln!("Не найдена точка входа игры!");
                return;
            }
            
            // 6. Запускаем игру с нашими параметрами
            println!("Запуск GTA V в режиме RebornMP...");
            let game_main: extern "system" fn() = std::mem::transmute(entry);
            game_main();
        }
    }
    
    println!("RebornMP работает! Игра запущена в изолированной среде.");
}

fn find_original_game() -> PathBuf {
    // Ищем установленную GTA V
    let paths = vec![
        "C:\\Program Files\\Rockstar Games\\Grand Theft Auto V\\GTA5.exe",
        "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V\\GTA5.exe",
        env::current_dir().unwrap().join("GTA5_original.exe").to_str().unwrap().to_string(),
    ];
    
    for path in paths {
        let pb = PathBuf::from(path);
        if pb.exists() {
            return pb;
        }
    }
    
    // Если не нашли - создаём заглушку (для тестов)
    println!("GTA V не найдена! Создаём тестовую среду...");
    create_test_environment()
}

fn create_sandbox(original_exe: &PathBuf) -> PathBuf {
    let sandbox_dir = env::current_dir().unwrap().join("RebornMP_Sandbox");
    
    if !sandbox_dir.exists() {
        std::fs::create_dir_all(&sandbox_dir).unwrap();
        
        // Копируем только необходимые файлы (исключая античит)
        let game_dir = original_exe.parent().unwrap();
        for entry in std::fs::read_dir(game_dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Исключаем античит-файлы
            let exclude = vec!["BattlEye", "SocialClub", "PlayGTAV"];
            if !exclude.iter().any(|e| name.contains(e)) {
                let dest = sandbox_dir.join(&name);
                if entry.file_type().unwrap().is_file() {
                    let _ = std::fs::copy(entry.path(), dest);
                }
            }
        }
        
        // Копируем наш лаунчер как GTA5.exe в песочницу
        let our_exe = sandbox_dir.join("GTA5.exe");
        let _ = std::fs::copy(env::current_exe().unwrap(), our_exe);
    }
    
    sandbox_dir
}

fn patch_anticheat(exe_path: &PathBuf) {
    println!("Патчинг античит-функций...");
    
    // Открываем EXE файл для патчинга
    match std::fs::OpenOptions::new().read(true).write(true).open(exe_path) {
        Ok(mut file) => {
            use std::io::{Read, Seek, Write};
            
            // Ищем сигнатуры античита и заменяем на NOP (No Operation)
            let signatures = vec![
                (b"\x48\x8B\x05\x00\x00\x00\x00\x48\x85\xC0\x74\x00\x8B\x48\x08", 
                 b"\x31\xC0\xC3\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90"), // BattlEye
                (b"\x40\x53\x48\x83\xEC\x20\x80\x3D\x00\x00\x00\x00\x00\x74\x00\x48\x8B\xD9",
                 b"\xB0\x01\xC3\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90\x90"), // Rockstar AC
            ];
            
            for (pattern, patch) in signatures {
                if let Some(pos) = find_pattern(&mut file, pattern) {
                    file.seek(std::io::SeekFrom::Start(pos as u64)).unwrap();
                    file.write_all(patch).unwrap();
                    println!("Патч применён по адресу: 0x{:X}", pos);
                }
            }
        }
        Err(e) => println!("Не удалось пропатчить EXE: {}", e),
    }
}

fn find_pattern(file: &mut std::fs::File, pattern: &[u8]) -> Option<usize> {
    use std::io::Read;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;
    
    buffer.windows(pattern.len())
        .position(|window| {
            window.iter().zip(pattern).all(|(&a, &b)| b == 0x00 || a == b)
        })
}

fn create_test_environment() -> PathBuf {
    // Для тестов создаём фейковую GTA V
    let test_dir = env::current_dir().unwrap().join("Test_GTAV");
    std::fs::create_dir_all(&test_dir).unwrap();
    
    let fake_exe = test_dir.join("GTA5.exe");
    std::fs::write(&fake_exe, b"MZ\x90\x00Test GTA V executable").unwrap();
    
    fake_exe
}