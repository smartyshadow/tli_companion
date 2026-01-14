# Анализ EULA: Безопасность TLI Companion
# EULA Analysis: TLI Companion Safety

---

## 🇷🇺 Русская версия

### Источник
Пользовательское соглашение XD Entertainment: https://protocol.xd.com/agreement.html

---

### Что делает TLI Companion?

| Действие | TLI Companion |
|----------|---------------|
| Читает лог-файлы игры | ✅ Да |
| Модифицирует игру | ❌ Нет |
| Взаимодействует с серверами игры | ❌ Нет |
| Даёт игровые преимущества | ❌ Нет |
| Автоматизирует действия в игре | ❌ Нет |
| Внедряется в процесс игры | ❌ Нет |

---

### Анализ запретов из EULA

#### Секция 3.4 — Запрещённые действия

**Цитата 1:**
> "Reverse engineer, translate, adapt, disassemble, decompile, or reduce to any form any Game or Services in whole or in part"
>
> (Запрещено делать реверс-инжиниринг, переводить, адаптировать, дизассемблировать, декомпилировать игру)

**Анализ:**
- ❌ Мы НЕ делаем реверс-инжиниринг
- ❌ Мы НЕ дизассемблируем/декомпилируем код игры
- ✅ Мы читаем **текстовые лог-файлы**, которые игра **сама создаёт** в открытом формате

**Вердикт:** ✅ **Не нарушаем**

---

**Цитата 2:**
> "Copy, reproduce, modify, translate, distribute, transmit, publish, perform, display, or communicate through internet any XD Services, Games, Contents in whole or in part"
>
> (Запрещено копировать, воспроизводить, модифицировать, распространять контент игры)

**Анализ:**
- ❌ Мы НЕ копируем игру
- ❌ Мы НЕ модифицируем игру
- ❌ Мы НЕ распространяем контент игры
- ⚠️ Мы показываем названия предметов — это может считаться "Contents"

**Вердикт:** ⚠️ **Серая зона** (названия предметов)

---

**Цитата 3:**
> "Sell, sub-license, rent, grant a security interest in any XD Services, Games, Contents"
>
> (Запрещено продавать, сублицензировать, сдавать в аренду контент игры)

**Анализ:**
- ❌ Мы НЕ продаём контент игры
- ✅ Приложение полностью бесплатное

**Вердикт:** ✅ **Не нарушаем**

---

#### Секция 5.1 — Правила поведения

**Цитата 4 (КЛЮЧЕВАЯ):**
> "Use, develop, host, or distribute cheats, automation software (bots), modded lobbies, hacks, mods, or any other unauthorized third-party software in connection with XD Services, or engage in any form of cheating, boosting, or exploiting bugs or glitches in XD Services to gain an in-game advantage"
>
> (Запрещено использовать читы, ботов, хаки, моды или другое неавторизованное ПО для получения игрового преимущества)

**Анализ:**
- ❌ Мы НЕ создаём cheats (читы)
- ❌ Мы НЕ создаём automation software (боты)
- ❌ Мы НЕ создаём hacks (хаки)
- ❌ Мы НЕ создаём mods (моды)
- ✅ Мы НЕ даём **"in-game advantage"** — только отображаем статистику

**Что такое "in-game advantage" (игровое преимущество)?**
- Автоматический фарм — ДА, это преимущество
- Авто-уклонение от атак — ДА, это преимущество
- Бот для аукциона — ДА, это преимущество
- **Просмотр статистики дропа — НЕТ, это только информация**

**Вердикт:** ✅ **Не нарушаем** (нет игрового преимущества)

---

**Цитата 5:**
> "Use any unauthorized third-party software that intercepts, collects, reads, or 'mines' information generated or stored by XD Services"
>
> (Запрещено использовать ПО, которое перехватывает, собирает, читает или майнит информацию от сервисов XD)

**Анализ:**
- ⚠️ Мы "read" (читаем) информацию
- ❌ Мы НЕ "intercept" (перехватываем) — логи уже созданы игрой
- ❌ Мы НЕ подключаемся к серверам XD
- ✅ Лог-файл — это **локальный файл на компьютере пользователя**

**Важное уточнение:** 
"XD Services" относится к серверам и онлайн-сервисам XD, а не к локальным файлам на компьютере игрока. Лог-файл создаётся самой игрой и хранится локально.

**Вердикт:** ⚠️ **Серая зона** (формально читаем данные)

---

**Цитата 6:**
> "Modify any file or any other part of Games and/or XD Services that XD does not specifically authorize You to modify"
>
> (Запрещено модифицировать любые файлы игры)

**Анализ:**
- ❌ Мы НЕ модифицируем никакие файлы игры
- ✅ Мы только ЧИТАЕМ логи (read-only)
- ✅ Игра сама создаёт логи при включении опции "Enable log"

**Вердикт:** ✅ **Не нарушаем**

---

### Прецедент: Torch+ в Китае

В Китае существует официально известное приложение **Torch+ (火炬+)**, которое:
- Читает лог-файлы игры
- Показывает статистику фарма
- Отслеживает дроп предметов
- **НЕ заблокировано разработчиками XD**

Это создаёт важный прецедент того, что XD **толерантны** к подобным приложениям статистики.

---

### Итоговая таблица

| Пункт EULA | Нарушаем? | Комментарий |
|------------|-----------|-------------|
| 3.4(1) Реверс-инжиниринг | ✅ Нет | Читаем готовые текстовые логи |
| 3.4(2) Копирование контента | ⚠️ Серая зона | Используем названия предметов |
| 3.4(3) Продажа контента | ✅ Нет | Приложение бесплатное |
| 5.1(6) Читы/боты | ✅ Нет | Нет игрового преимущества |
| 5.1(8) Сбор данных | ⚠️ Серая зона | Читаем локальные логи |
| 5.1(9) Модификация файлов | ✅ Нет | Ничего не модифицируем |

---

### Почему это безопасно?

1. **Нет игрового преимущества** — приложение не играет за вас, не автоматизирует действия, не даёт преимуществ в бою или на аукционе

2. **Только чтение** — мы не модифицируем никакие файлы игры, только читаем логи

3. **Локальные данные** — мы работаем с файлами на вашем компьютере, не взаимодействуем с серверами XD

4. **Прецедент Torch+** — аналогичное приложение работает в Китае без блокировки

5. **Пользователь контролирует** — логирование включается самим игроком в настройках игры

---

### Риски

1. **Теоретический риск** — XD может в любой момент изменить политику и начать блокировать подобные приложения

2. **Серые зоны EULA** — некоторые пункты можно интерпретировать по-разному

3. **Рекомендация** — используйте на свой страх и риск, как и любое стороннее ПО

---

### Вывод

TLI Companion **не нарушает основные запреты EULA**:
- ❌ Не делает реверс-инжиниринг
- ❌ Не модифицирует файлы игры  
- ❌ Не взаимодействует с серверами
- ❌ Не даёт игрового преимущества
- ❌ Не автоматизирует действия
- ✅ Только читает локальные лог-файлы

---
---
---

## 🇬🇧 English Version

### Source
XD Entertainment User Agreement: https://protocol.xd.com/agreement.html

---

### What does TLI Companion do?

| Action | TLI Companion |
|--------|---------------|
| Reads game log files | ✅ Yes |
| Modifies the game | ❌ No |
| Interacts with game servers | ❌ No |
| Provides in-game advantages | ❌ No |
| Automates in-game actions | ❌ No |
| Injects into game process | ❌ No |

---

### EULA Restrictions Analysis

#### Section 3.4 — Prohibited Actions

**Quote 1:**
> "Reverse engineer, translate, adapt, disassemble, decompile, or reduce to any form any Game or Services in whole or in part"

**Analysis:**
- ❌ We do NOT reverse engineer
- ❌ We do NOT disassemble/decompile game code
- ✅ We read **plain text log files** that the game **creates itself** in an open format

**Verdict:** ✅ **No violation**

---

**Quote 2:**
> "Copy, reproduce, modify, translate, distribute, transmit, publish, perform, display, or communicate through internet any XD Services, Games, Contents in whole or in part"

**Analysis:**
- ❌ We do NOT copy the game
- ❌ We do NOT modify the game
- ❌ We do NOT distribute game content
- ⚠️ We display item names — this might be considered "Contents"

**Verdict:** ⚠️ **Gray area** (item names)

---

**Quote 3:**
> "Sell, sub-license, rent, grant a security interest in any XD Services, Games, Contents"

**Analysis:**
- ❌ We do NOT sell game content
- ✅ The application is completely free

**Verdict:** ✅ **No violation**

---

#### Section 5.1 — Code of Conduct

**Quote 4 (KEY):**
> "Use, develop, host, or distribute cheats, automation software (bots), modded lobbies, hacks, mods, or any other unauthorized third-party software in connection with XD Services, or engage in any form of cheating, boosting, or exploiting bugs or glitches in XD Services to gain an in-game advantage"

**Analysis:**
- ❌ We do NOT create cheats
- ❌ We do NOT create automation software (bots)
- ❌ We do NOT create hacks
- ❌ We do NOT create mods
- ✅ We do NOT provide **"in-game advantage"** — only display statistics

**What is "in-game advantage"?**
- Auto-farming — YES, this is an advantage
- Auto-dodging attacks — YES, this is an advantage
- Auction bot — YES, this is an advantage
- **Viewing drop statistics — NO, this is just information**

**Verdict:** ✅ **No violation** (no in-game advantage)

---

**Quote 5:**
> "Use any unauthorized third-party software that intercepts, collects, reads, or 'mines' information generated or stored by XD Services"

**Analysis:**
- ⚠️ We "read" information
- ❌ We do NOT "intercept" — logs are already created by the game
- ❌ We do NOT connect to XD servers
- ✅ Log file is a **local file on the user's computer**

**Important clarification:** 
"XD Services" refers to XD's servers and online services, not local files on the player's computer. The log file is created by the game itself and stored locally.

**Verdict:** ⚠️ **Gray area** (technically reading data)

---

**Quote 6:**
> "Modify any file or any other part of Games and/or XD Services that XD does not specifically authorize You to modify"

**Analysis:**
- ❌ We do NOT modify any game files
- ✅ We only READ logs (read-only)
- ✅ The game creates logs when "Enable log" option is turned on

**Verdict:** ✅ **No violation**

---

### Precedent: Torch+ in China

In China, there is a well-known application **Torch+ (火炬+)** that:
- Reads game log files
- Shows farming statistics
- Tracks item drops
- **Is NOT blocked by XD developers**

This sets an important precedent that XD is **tolerant** of such statistics applications.

---

### Summary Table

| EULA Section | Violation? | Comment |
|--------------|------------|---------|
| 3.4(1) Reverse engineering | ✅ No | We read ready-made text logs |
| 3.4(2) Copying content | ⚠️ Gray area | We use item names |
| 3.4(3) Selling content | ✅ No | App is free |
| 5.1(6) Cheats/bots | ✅ No | No in-game advantage |
| 5.1(8) Data collection | ⚠️ Gray area | We read local logs |
| 5.1(9) File modification | ✅ No | We modify nothing |

---

### Why is it safe?

1. **No in-game advantage** — the app doesn't play for you, doesn't automate actions, doesn't give advantages in combat or at auction

2. **Read-only** — we don't modify any game files, only read logs

3. **Local data** — we work with files on your computer, don't interact with XD servers

4. **Torch+ precedent** — a similar app operates in China without being blocked

5. **User control** — logging is enabled by the player in game settings

---

### Risks

1. **Theoretical risk** — XD may change their policy at any time and start blocking such applications

2. **EULA gray areas** — some clauses can be interpreted differently

3. **Recommendation** — use at your own risk, like any third-party software

---

### Conclusion

TLI Companion **does not violate the main EULA prohibitions**:
- ❌ Does not reverse engineer
- ❌ Does not modify game files
- ❌ Does not interact with servers
- ❌ Does not provide in-game advantages
- ❌ Does not automate actions
- ✅ Only reads local log files

---

## Disclaimer / Дисклеймер

**🇷🇺 Русский:**
Данный анализ носит информационный характер и не является юридической консультацией. Используйте TLI Companion на свой страх и риск. Разработчики не несут ответственности за возможные последствия использования приложения.

**🇬🇧 English:**
This analysis is for informational purposes only and does not constitute legal advice. Use TLI Companion at your own risk. The developers are not responsible for any consequences of using the application.
