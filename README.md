# 🔄 RebornMP

**RebornMP** — это мультиплеерный фреймворк для Grand Theft Auto V, созданный как современная альтернатива FiveM, RageMP и alt:V.
Проект строится на производительном ядре **Rust** (в хуках и памяти игры) и гибкой серверной логике на **TypeScript** (через Bun).

Основная цель — дать сообществу максимально отзывчивый, безопасный и удобный инструмент для создания RP, DM, Arcade режимов.

## 🧬 Архитектура (Как это работает)

Проект разделен на 5 логических частей, связанных воедино:

- **`client/`** (Rust, `cdylib`): Инжектируемая DLL.
    - *Чем занимается:* Перехват функций игры (хуки DirectX, сеть, скрипты), чтение/запись памяти, отрисовка UI (WebView2).
- **`server/`** (Rust Core + Bun/TS): Серверная часть.
    - *Чем занимается:* Rust ядро управляет сокетами и памятью, а скрипты на TypeScript (через `index.ts`) обрабатывают логику (покупки машин, работа фракций).
- **`shared/`** (Rust, `lib`): Общая библиотека.
    - *Чем занимается:* Описание структуры пакетов (пакетов), которые летят между клиентом и сервером.
- **`sdk/`** (Rust, `lib`): Reversal SDK.
    - *Чем занимается:* Это «карта игры». Содержит смещения памяти, структуры C++ классов GTA V (CVehicle, CPed и т.д.) и сигнатуры для поиска функций.
- **`launcher/`** (Rust, `bin`): Лаунчер.
    - *Чем занимается:* Запускает GTA5.exe, ожидает загрузки и внедряет (инжектит) `client.dll`.

## 🛠️ Текущее состояние (Alpha)

Проект на стадии активной разработки. Базовая связка «Лаунчер -> Инжект -> Подключение к серверу» работает.

- [x] Каркас всех модулей
- [x] Пример хука (паттерн поиска функций)
- [x] Связь `Client <-> Server` через кастомные пакеты
- [x] Базовый лаунчер с инжектом
- [ ] Система событий (на подобие `alt.on`)
- [ ] API машин/игроков в TS
- [ ] Рендер UI

## 🚀 Быстрый старт (для разработчиков)

Хотите помочь проекту? Вот инструкция по сборке.

### Требования
1.  **Rust** (последний stable): [https://rustup.rs/](https://rustup.rs/)
2.  **Bun** (для сервера): `npm install -g bun`
3.  **Visual Studio 2022** с компонентами «Разработка классических приложений на C++» (нужен для сборки DLL и поиска Windows SDK).
4.  Лицензионная копия **GTA V** (Steam/Rockstar Launcher).

### Сборка проекта

1.  **Клонируйте репозиторий:**
    ```bash
    git clone https://github.com/cditaproject/RebornMP.git
    cd RebornMP
Сборка всех Rust-компонентов:

bash
# Собираем клиент (client.dll), лаунчер и серверное ядро
cargo build --release
Настройка сервера:

bash
cd server
bun install
# Запуск сервера в режиме разработки
bun run index.ts
Убедитесь, что порт 7777 открыт или измените его в конфиге.

Запуск клиента:

bash
cd ../
# Запускаем лаунчер, указав путь к GTA5.exe
cargo run --bin launcher -- "D:\Games\GTAV\GTA5.exe"
🧪 Проверка работоспособности
Если вы всё сделали верно:

Откроется GTA V.

Вы должны увидеть сообщение в консоли лаунчера: [RebornMP] Client injected successfully.

В игре появится тестовый текст или консоль (F8) с подключением к серверу.

📌 Roadmap (План развития)
Чтобы проект развивался, сосредоточьтесь на следующих этапах:

Этап 1. Стабилизация ядра (Сейчас тут)

Добиться стабильной работы хуков (нет вылетов при спавне).

Добавить краш-репорты (минидампы) с клиента.

Реализовать «хэндшейк» (рукопожатие) клиент-сервер с шифрованием (XOR/AES).

Этап 2. API транспортных средств и игрока

Экспорт функций из Rust в TS: getPlayerPosition(), setPlayerHealth().

Спавн машин, синхронизация позиций (авторитет сервера).

Система событий (как в RageMP).

Этап 3. Интерфейс и удобство

Интеграция WebView2 (рендер HTML/CSS/JS поверх игры).

Команды в чат (/me, /car).

Система ресурсов (загрузка клиентских скриптов с сервера).

Этап 4. Релиз MVP (Minimum Viable Product)

Документация по API.

Пример RP-сервера в комплекте.

Релиз на GitHub с готовыми сборками.

🤝 Как помочь (Contributing)
Любая помощь приветствуется. Вот где она нужна больше всего:

Reversing (Самый важный навык):

Изучите папку sdk/. Найдите актуальные смещения для GTA v1.0.XXX.

Обновите структуры CVehicle, CPed, CPlayerInfo.

Rust разработка:

Улучшите менеджер памяти в client/src/memory.rs (паттерн-скан).

Оптимизируйте сериализацию пакетов в shared/src/lib.rs (используйте bincode или speedy).

TypeScript / Backend:

Напишите примеры ресурсов (ресурс для спавна машин).

Реализуйте событие onPlayerConnect.

Тестирование:

Соберите проект и просто играйте. Сообщайте о вылетах в Issues.

Как предложить свои изменения:

Сделайте Fork проекта.

Создайте ветку: git checkout -b feature/amazing_hook.

Сделайте коммит: git commit -m 'Добавил крутой хук'.

Push: git push origin feature/amazing_hook.

Откройте Pull Request (PR).
```md
EntityMP/
├── Cargo.toml               # Root Workspace file uniting all Rust crates
├── client/                  # (DLL) Client-side library injected into the game
│   ├── Cargo.toml           # crate-type = ["cdylib"]
│   └── src/
│       ├── lib.rs           # DllMain, DLL entry point
│       ├── hooks.rs         # Hooking/intercepting game functions (DirectX, logic)
│       ├── memory.rs        # Reading/writing game process memory
│       ├── network.rs       # Network communication with the server
│       └── ui.rs            # UI rendering integration (WebView2 / Ultralight)
├── server/                  # Server-side application (Rust Core + Bun/Node TS)
│   ├── core/                # Server core logic in Rust
│   │   ├── Cargo.toml       # crate-type = ["cdylib"]
│   │   └── src/lib.rs       # Exports functions via FFI for Bun to consume
│   ├── package.json         # Package manifest and configuration for the Bun server
│   └── src/                 # Server scripts written in TypeScript
│       └── index.ts         # Main entry point for the server
├── shared/                  # (LIB) Shared library used by both client and server
│   ├── Cargo.toml           # crate-type = ["lib"]
│   └── src/lib.rs           # Packet serialization, shared data types and structures
├── sdk/                     # (LIB) Reversal SDK containing game engine structures and memory offsets
│   ├── Cargo.toml
│   └── src/lib.rs           # Memory addresses, patterns, and reversed C++ classes
└── launcher/                # (EXE) Launcher tool to start the game and inject client.dll
├── Cargo.toml
└── src/main.rs          # Spawns the game process and injects client.dll
```
