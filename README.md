# 🔫 RebornMP

[![Build Status](https://github.com/cditaproject/RebornMP/actions/workflows/build.yml/badge.svg)](https://github.com/cditaproject/RebornMP/actions/workflows/build.yml)
[![Rust](https://img.shields.io/badge/rust-77.4%25-orange)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/typescript-22.6%25-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

## ✨ Особенности

- 🚀 **Высокая производительность** — ядро на Rust
- 💬 **Встроенный чат** с поддержкой команд
- 🔌 **Легкое расширение** — пишите логику на TypeScript
- 🎨 **Современный UI** через WebView2
- 🔒 **Безопасная инъекция** с проверкой прав
- 📦 **Простая сборка** через Cargo и Bun

## 📋 Содержание

- [Требования](#требования)
- [Установка](#установка)
- [Сборка](#сборка)
- [Запуск](#запуск)
- [Структура проекта](#структура-проекта)
- [Команды сервера](#команды-сервера)
- [Разработка](#разработка)
- [Лицензия](#лицензия)

## 🔧 Требования

| Компонент | Версия |
|-----------|--------|
| **Rust** | 1.70+ |
| **Bun** | 1.0+ |
| **GTA V** | v1.0.1868+ |
| **Windows** | 10/11 (x64) |

## 📥 Установка

### 1. Клонирование репозитория

```bash
git clone https://github.com/cditaproject/RebornMP.git
cd RebornMP

📁 Структура проекта
```md
EntityMP/
├── 📄 Cargo.toml                     # Корневой манифест workspace (Root workspace file)
├── 📄 Cargo.lock                     # Закреплённые версии зависимостей (Locked dependencies)
├── 📄 README.md                      # Документация проекта (Project documentation)
├── 📄 .gitignore                     # Игнорируемые файлы Git (Git ignore rules)
│
├── 📁 .cargo/                        # Конфигурация Cargo (Cargo configuration)
│   └── 📄 config.toml                # Настройки сборщика (Build settings)
│
├── 📁 .github/                       # GitHub конфигурация (GitHub configuration)
│   └── 📁 workflows/
│       └── 📄 build.yml              # CI/CD пайплайн (автосборка) - CI/CD pipeline
│
├── 🚀 launcher/                      # ЛАУНЧЕР - внедряет DLL в игру (Launcher - injects DLL)
│   ├── 📄 Cargo.toml                 # Зависимости: winapi, chrono (Dependencies)
│   └── 📁 src/
│       └── 📄 main.rs                # Точка входа, инъекция через CreateRemoteThread (Entry point, injection)
│       # ✅ Статус: ПОЛНОСТЬЮ ГОТОВ (COMPLETED)
│
├── 🎮 client/                        # КЛИЕНТ - DLL внутри игры (Client - in-game DLL)
│   ├── 📄 Cargo.toml                 # crate-type = ["cdylib"] (сборка как DLL)
│   └── 📁 src/
│       ├── 📄 lib.rs                 # DllMain - вход при инъекции (DllMain - entry on injection)
│       ├── 📄 hooks.rs               # Перехват DirectX и функций игры (Hooking DirectX & game funcs)
│       ├── 📄 memory.rs              # Чтение/запись памяти GTA V (GTA V memory R/W)
│       ├── 📄 network.rs             # Связь с сервером (TCP/WebSocket) (Server communication)
│       └── 📄 ui.rs                  # Интерфейс WebView2 / Ultralight (UI rendering)
│       # 🟡 Статус: ЧАСТИЧНО ГОТОВ (PARTIALLY COMPLETED)
│
├── 🌐 server/                        # СЕРВЕР - логика и синхронизация (Server - logic & sync)
│   ├── 📄 package.json               # Зависимости Bun (Bun dependencies)
│   ├── 📄 tsconfig.json              # Конфигурация TypeScript (TS config)
│   ├── 📄 bun.lockb                  # Блокировка версий Bun (Bun lockfile)
│   ├── 📁 core/                      # Ядро на Rust (FFI экспорты для Bun)
│   │   ├── 📄 Cargo.toml             # crate-type = ["cdylib"]
│   │   └── 📁 src/
│   │       └── 📄 lib.rs             # FFI функции, вызываемые из Bun (FFI exports for Bun)
│   └── 📁 src/                       # Логика на TypeScript (TypeScript logic)
│       ├── 📄 index.ts               # Точка входа сервера (Main entry point)
│       ├── 📁 entities/              # Игровые сущности (Game entities)
│       │   ├── 📄 Player.ts          # Игрок: позиция, здоровье, деньги (Player: pos, health, money)
│       │   ├── 📄 Vehicle.ts         # Транспорт: модель, повреждения (Vehicle: model, damage)
│       │   └── 📄 World.ts           # Мир: время, погода, объекты (World: time, weather, objects)
│       ├── 📁 commands/              # Система команд (Command system)
│       │   ├── 📄 chat.ts            # Чат: /me, /ooc, /help (Chat commands)
│       │   ├── 📄 admin.ts           # Админ: /kick, /ban, /goto (Admin commands)
│       │   └── 📄 economy.ts         # Экономика: /pay, /shop, /sell (Economy commands)
│       ├── 📁 systems/               # Игровые системы (Game systems)
│       │   ├── 📄 auth.ts            # Авторизация и аккаунты (Auth & accounts)
│       │   ├── 📄 anticheat.ts       # Античит: проверка памяти (Anti-cheat: memory checks)
│       │   └── 📄 sync.ts            # Синхронизация игроков (Player synchronization)
│       └── 📁 utils/                 # Утилиты (Utilities)
│           ├── 📄 logger.ts          # Логирование на сервере (Server logging)
│           └── 📄 database.ts        # Подключение к БД (Database connection)
│       # 🟡 Статус: БАЗОВАЯ ЛОГИКА ЕСТЬ (BASIC LOGIC DONE)
│
├── 🔗 shared/                        # ОБЩИЙ МОДУЛЬ - клиент + сервер (Shared module)
│   ├── 📄 Cargo.toml                 # crate-type = ["lib"] (статическая библиотека)
│   └── 📁 src/
│       └── 📄 lib.rs                 # Сериализация пакетов (Packet serialization)
│       # 🟡 Статус: ОСНОВА ГОТОВА (BASIC DONE)
│
├── 🔧 sdk/                           # SDK - оффсеты и структуры GTA V (Reversal SDK)
│   ├── 📄 Cargo.toml
│   └── 📁 src/
│       ├── 📄 lib.rs                 # Главный экспорт SDK (Main SDK exports)
│       ├── 📄 offsets.rs             # Смещения памяти GTA V (Memory offsets)
│       ├── 📄 patterns.rs            # Поиск сигнатур (Pattern scanning)
│       ├── 📄 natives.rs             # Нативные вызовы RAGE (Native RAGE calls)
│       ├── 📁 classes/               # Реверс-инжиниринг классов (Reversed C++ classes)
│       │   ├── 📄 ped.rs             # CPed - персонажи (CPed - peds/npcs)
│       │   ├── 📄 vehicle.rs         # CVehicle - транспорт (CVehicle - cars)
│       │   ├── 📄 player.rs          # CPlayerInfo - данные игрока (Player data)
│       │   └── 📄 world.rs           # CWorld - мир (World state)
│       └── 📁 hooks/                 # Хуки для перехвата (System hooks)
│           ├── 📄 directx.rs         # Перехват DirectX (DirectX hooks)
│           └── 📄 script.rs          # Перехват скриптов (Script hooks)
│       # 🟢 Статус: ОФФСЕТЫ В ОСНОВНОМ ЕСТЬ (OFFSETS MOSTLY DONE)
│
├── 📚 docs/                          # ДОКУМЕНТАЦИЯ (Documentation)
│   ├── 📄 API.md                     # Справочник по API (API reference)
│   ├── 📄 BUILD.md                   # Инструкция по сборке (Build guide)
│   └── 📄 CONTRIBUTING.md            # Как помочь проекту (Contributing guide)
│       # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
│
├── 💡 examples/                      # ПРИМЕРЫ КОДА (Code examples)
│   ├── 📄 basic_chat.rs              # Простой чат (Basic chat example)
│   ├── 📄 vehicle_spawn.rs           # Спавн машин (Vehicle spawning)
│   └── 📄 simple_command.ts          # Команда на сервере (Server command)
│       # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
│
└── 🧪 tests/                         # ТЕСТЫ (Tests)
    ├── 📁 integration/               # Интеграционные тесты (Integration tests)
    │   ├── 📄 test_network.rs        # Тест сети (Network test)
    │   └── 📄 test_injection.rs      # Тест инъекции (Injection test)
    └── 📁 unit/                      # Модульные тесты (Unit tests)
        ├── 📄 test_memory.rs         # Тест памяти (Memory test)
        └── 📄 test_packets.rs        # Тест пакетов (Packet test)
        # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
```
