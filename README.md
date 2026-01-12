# TLI Companion

Desktop-приложение для автоматического трекинга фарма в **Torchlight Infinite**.

## Возможности

- 🎯 **Автоматический подсчёт дропа** из логов игры
- 💰 **Отслеживание цен** с аукциона в реальном времени
- ⏱️ **Таймер карт** и расчёт дохода в час
- 🔄 **Синхронизация** с kripika.com

## Требования

- **Rust** 1.70+ (для сборки)
- **Node.js** 18+ (для frontend)
- **Torchlight Infinite** с включёнными логами

### Включение логов в игре

1. Запустите игру
2. Откройте консоль (`)
3. Введите `log.enable`

Логи будут записываться в:
```
[Steam Path]/steamapps/common/Torchlight Infinite/UE_game/TorchLight/Saved/Logs/UE_game.log
```

## Разработка

```bash
# Установка зависимостей
npm install

# Запуск в режиме разработки
npm run tauri dev

# Сборка release версии
npm run tauri build
```

## Архитектура

```
tli-companion/
├── src/                    # React frontend
│   ├── App.tsx            # Главный компонент
│   └── styles.css         # Глобальные стили
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs        # Точка входа
│   │   ├── log_parser.rs  # Парсер логов
│   │   ├── file_watcher.rs # Отслеживание файлов
│   │   ├── state.rs       # Состояние приложения
│   │   ├── commands.rs    # IPC команды
│   │   └── types.rs       # Типы данных
│   └── Cargo.toml         # Зависимости Rust
└── package.json           # Зависимости JS
```

## IPC Commands

### Из JavaScript в Rust

```typescript
// Найти путь к логам автоматически
const path = await invoke<string | null>("find_log_file");

// Начать сессию
await invoke("start_session", { presetId: "some-preset-id" });

// Получить статистику
const stats = await invoke<SessionStats>("get_session_stats");

// Получить дропы
const drops = await invoke<AggregatedDrop[]>("get_drops");

// Завершить сессию
const finalStats = await invoke<SessionStats>("end_session");
```

### События от Rust в JavaScript

```typescript
import { listen } from "@tauri-apps/api/event";

// Новый дроп
listen<ItemDropEvent>("item-drop", (event) => {
  console.log("Dropped:", event.payload);
});

// Обновление цен
listen<PriceSearchEvent>("price-update", (event) => {
  console.log("Price:", event.payload);
});

// Смена карты
listen<MapChangeEvent>("map-change", (event) => {
  console.log("Map:", event.payload);
});

// Обновление статистики
listen<SessionStats>("stats-update", (event) => {
  console.log("Stats:", event.payload);
});
```

## Лицензия

MIT
