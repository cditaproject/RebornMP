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
RebornMP/
├── 📄 Cargo.toml                      # Корневой манифест workspace
├── 📄 Cargo.lock                      # Закреплённые версии зависимостей
├── 📄 README.md                       # Документация проекта
├── 📄 .gitignore                      # Игнорируемые файлы Git
├── 📄 package.json                    # (Корневой, если есть)
│
├── 📁 .cargo/                         # Конфигурация Cargo
│   └── 📄 config.toml                 # Настройки сборщика (линковка, цели)
│
├── 📁 .github/                        # Конфигурация GitHub
│   └── 📁 workflows/
│       └── 📄 build.yml               # CI/CD пайплайн (автосборка)
│
├── 🚀 **launcher/**                   # ЛАУНЧЕР (внедряет DLL)
│   ├── 📄 Cargo.toml                  # Зависимости: winapi, chrono
│   └── 📁 src/
│       └── 📄 main.rs                 # Точка входа, инъекция через CreateRemoteThread
│       # ✅ Статус: ПОЛНОСТЬЮ ГОТОВ
│
├── 🎮 **client/**                     # КЛИЕНТ (DLL внутри игры)
│   ├── 📄 Cargo.toml                  # crate-type = ["cdylib"]
│   └── 📁 src/
│       ├── 📄 lib.rs                  # DllMain, точка входа при инъекции
│       ├── 📄 hooks.rs                # Перехват DirectX и игровых функций
│       ├── 📄 memory.rs               # Чтение/запись памяти GTA V
│       ├── 📄 network.rs              # Связь с сервером (TCP/WebSocket)
│       └── 📄 ui.rs                   # Интерфейс (WebView2 / консоль)
│       # 🟡 Статус: ЧАСТИЧНО ГОТОВ (есть всё необходимое)
│
├── 🌐 **server/**                     # СЕРВЕР (Bun + Rust Core)
│   ├── 📄 package.json                # Зависимости Bun (TypeScript)
│   ├── 📄 tsconfig.json               # Конфигурация TypeScript
│   ├── 📄 bun.lockb                   # Блокировка версий Bun
│   ├── 📁 core/                       # Высокопроизводительное ядро на Rust
│   │   ├── 📄 Cargo.toml              # crate-type = ["cdylib"]
│   │   └── 📁 src/
│   │       └── 📄 lib.rs              # FFI-функции для вызова из Bun
│   └── 📁 src/                        # Логика на TypeScript
│       ├── 📄 index.ts                # Точка входа сервера
│       ├── 📁 entities/               # Игровые сущности
│       │   ├── 📄 Player.ts           # Игрок: позиция, здоровье, деньги
│       │   ├── 📄 Vehicle.ts          # Транспорт: модель, повреждения
│       │   └── 📄 World.ts            # Мир: время, погода, объекты
│       ├── 📁 commands/               # Система команд
│       │   ├── 📄 chat.ts             # /me, /ooc, /help
│       │   ├── 📄 admin.ts            # /kick, /ban, /goto
│       │   └── 📄 economy.ts          # /pay, /shop, /sell
│       ├── 📁 systems/                # Игровые системы
│       │   ├── 📄 auth.ts             # Авторизация и аккаунты
│       │   ├── 📄 anticheat.ts        # Античит: проверка памяти
│       │   └── 📄 sync.ts             # Синхронизация игроков
│       └── 📁 utils/                  # Утилиты
│           ├── 📄 logger.ts           # Логирование на сервере
│           └── 📄 database.ts         # Подключение к БД (SQLite/PostgreSQL)
│       # 🟡 Статус: БАЗОВАЯ ЛОГИКА ЕСТЬ
│
├── 🔗 **shared/**                     # ОБЩИЙ МОДУЛЬ (клиент + сервер)
│   ├── 📄 Cargo.toml                  # crate-type = ["lib"]
│   └── 📁 src/
│       └── 📄 lib.rs                  # Сериализация пакетов, общие структуры
│       # 🟡 Статус: ОСНОВА ГОТОВА
│
├── 🔧 **sdk/**                        # SDK (оффсеты и структуры GTA V)
│   ├── 📄 Cargo.toml
│   └── 📁 src/
│       ├── 📄 lib.rs                  # Главный экспорт SDK
│       ├── 📄 offsets.rs              # Смещения памяти GTA V
│       ├── 📄 patterns.rs             # Поиск сигнатур
│       ├── 📄 natives.rs              # Нативные вызовы RAGE
│       ├── 📁 classes/                # Реверс-инжиниринг классов
│       │   ├── 📄 ped.rs              # CPed
│       │   ├── 📄 vehicle.rs          # CVehicle
│       │   ├── 📄 player.rs           # CPlayerInfo
│       │   └── 📄 world.rs            # CWorld
│       └── 📁 hooks/                  # Хуки для перехвата
│           ├── 📄 directx.rs          # Перехват DirectX
│           └── 📄 script.rs           # Перехват скриптов
│       # 🟢 Статус: ОФФСЕТЫ В ОСНОВНОМ ЕСТЬ
│
├── 📚 **docs/**                       # ДОКУМЕНТАЦИЯ (в планах)
│   ├── 📄 API.md                      # Справочник по API
│   ├── 📄 BUILD.md                    # Инструкция по сборке
│   └── 📄 CONTRIBUTING.md             # Как помочь проекту
│       # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
│
├── 💡 **examples/**                    # ПРИМЕРЫ КОДА (в планах)
│   ├── 📄 basic_chat.rs
│   ├── 📄 vehicle_spawn.rs
│   └── 📄 simple_command.ts
│       # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
│
└── 🧪 **tests/**                       # ТЕСТЫ (в планах)
    ├── 📁 integration/
    │   ├── 📄 test_network.rs
    │   └── 📄 test_injection.rs
    └── 📁 unit/
        ├── 📄 test_memory.rs
        └── 📄 test_packets.rs
        # ⚪ Статус: НЕ ГОТОВО (NOT STARTED)
```
