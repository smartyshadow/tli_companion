# TLI Log Parsing Research

Исследование лог-файлов Torchlight Infinite для извлечения полезной информации.

## Путь к лог-файлу
```
D:\Steam\steamapps\common\Torchlight Infinite\UE_game\TorchLight\Saved\Logs\UE_game.log
```

---

## 1. ЗАГРУЗКА КАРТЫ (Map Loading)

### Начало/конец загрузки
```
Loading@ BeginLoadingScreen MapName = /Game/Art/Maps/04DD/DD_TanXiZhiQiang000/DD_TanXiZhiQiang000
Loading@ EndLoadingScreen MapName = /Game/Art/Maps/04DD/DD_TanXiZhiQiang000/DD_TanXiZhiQiang000, CostTime = 112.408501
```
- **Применение:** Подсчёт количества карт
- **CostTime:** Время загрузки в мс

### Вход в бой с именем персонажа
```
SwitchBattleAreaUtil:_JoinFight Kripatestyoug:1300
```
- `Kripatestyoug` — имя персонажа
- `1300` — AreaId (область Netherrealm)

---

## 2. ДАННЫЕ КАРТЫ (Map Data)

### Основные параметры
```
+levelType [3]
+battleTag [126]
+ownerPlayerId [879222993438420992]
+maptype [Mystic]           ← тип активности (Netherrealm = Mystic)
+areaId [7904474401379502036]
+mapId [1061307]
+AreaUniqueId [792633534418650675]
+seed [1938722219]
+levelId [4643]
```

### Область и уровень
```
+KeyType [SpAreaId]      → value [1300]
+KeyType [SpAreaLevel]   → value [6]
```

### AreaId маппинг (Netherrealm)
- 1000 — Outlaw Sands?
- 1100 — ?
- 1200 — ?
- 1300 — Whispering Mist?
- 1400 — ?

---

## 3. МОНСТРЫ НА КАРТЕ (Monster Spawner)

### Общее количество
```
MonsterSpawner: total number 268
MonsterSpawner: rarity type 2, number 36     ← Magic монстры
MonsterSpawner: rarity type 3, number 4      ← Rare монстры
MonsterSpawner: affix rarity type 3, number 4
MonsterSpawner: affix rarity type 4, number 36
```

### Детальная разбивка
```
UCBornsUtils: CreateOutSidePoint monsterGroupArray num:54, E_elite:4, E_reinforce:36, E_normal:228
```
- **E_elite:** 4 — элитных монстров
- **E_reinforce:** 36 — усиленных (magic) монстров  
- **E_normal:** 228 — обычных монстров

### Rarity Types (предположение)
- Type 2 = Magic (синие)
- Type 3 = Rare (жёлтые)
- Type 4 = ? (возможно Elite/Boss)

**ПРИМЕНЕНИЕ:** Определение "жирности" карты — больше elite/rare = лучше!

---

## 4. АФФИКСЫ КАРТЫ (Map Modifiers)

### ID модификаторов
```
AddMapModifier(5122003) success 1
AddMapModifier(5212017) success 2
AddMapModifier(5311009) success 3
...
```

### Структура аффикса
```
+Description [<p>+100</p>% additional <e id=507>Drop Quantity</e>]
+DangerNumbers
+Id [7000001]
+Tier [0]
+FId [1141]
```

### Категории модификаторов

#### Негативные (опасные для игрока)
| ID | Описание |
|----|----------|
| 5122003 | +30% Monster Fire Resistance |
| 5212017 | -58% Blessing Duration for you |
| 5311009 | -12% Life Regain and Energy Shield Regain |
| 5422025 | Rare monsters restore 35% Max Life |
| 5521004 | Rare monsters cast Frigid Transmission |

#### Позитивные (увеличивают лут)
| ID | Описание |
|----|----------|
| 7000001 | +100% additional Drop Quantity |
| 1030011 | +5% Drop Quantity |
| 1030000 | In Netherrealm stages, +2% Drop Quantity |
| 3001000 | 1 Dyed Snow Monster will appear |
| 71100910 | 5 additional groups of Magic monster(s) |
| 6100003 | All Magic monsters become Magic Treasure Sentries |
| 1010040 | Embers have 2% chance to be replicated |
| 1010041 | Embers have 8% chance to be replicated |
| 1010050 | Bosses 10% chance to drop 1 additional Ember |
| 1010080 | Bosses 5% chance to drop 5 Netherrealm Resonance |
| 1010081 | Bosses 5% chance to drop 20 Netherrealm Resonance |
| 1010082 | Bosses 5% chance to drop 40 Netherrealm Resonance |

### Нелокализованные аффиксы
Некоторые аффиксы не имеют текстового описания, только ID:
```
+Description [affix_class|description|1001200000]
+Id [200006]
```

**ПРИМЕНЕНИЕ:** 
- Парсить Drop Quantity для оценки карты
- Детектить Dyed Snow Monster (ID: 3001000)
- Детектить Treasure Sentries (ID: 6100003)
- Показывать шансы на бонусный дроп
- Считать суммарный Drop Quantity%

---

## 5. СОЗДАНИЕ МОНСТРОВ В РЕАЛЬНОМ ВРЕМЕНИ

### Лог создания монстра
```
create monster npc id 1140033 rarity 1 affix 794061
create monster npc id 9999832 rarity 2 affix: null
Monster Created: npc id 9999832 rarity 2
```

### Структура
- **npc id** — идентификатор типа монстра (1140033, 1140006, 1140088, 9999832...)
- **rarity** — редкость (1 = normal, 2 = magic, 3 = rare?)
- **affix** — ID аффикса монстра или `null`

**ПРИМЕНЕНИЕ:** 
- Отслеживать спавн монстров в реальном времени
- Считать magic/rare монстров отдельно
- Возможно определить тип Netherrealm по ID монстров

---

## 6. БОЙ С БОССОМ (Boss Fight)

### Начало боя с боссом
```
AudioBGM PushNewState BOSS
Play audio PostEventAsync bgm Boss_Music/Play_Mus_Boss_Sml_Gen01
```
- `Boss_Sml` — маленький босс (stage boss)
- `Boss_Big` — большой босс (main boss)

### Босс убит
```
AudioBGM Destory BOSS
AudioBGM OnExit BOSS
Play_vo_hero_104_kill_boss    ← голос персонажа "убил босса"
```

### Убийство элитника
```
Play_vo_hero_104_kill_elite
```

**ПРИМЕНЕНИЕ:**
- Детектить начало/конец боя с боссом
- Считать убитых боссов
- Измерять время боя с боссом

---

## 7. ПОРТАЛЫ НА КАРТЕ

### Создание портала
```
Create Map Portal cfgId 5 etyId 6093 uId 7
Create Map Portal cfgId 6 etyId 6095 uId 8
```

**ПРИМЕНЕНИЕ:** Детект создания exit-портала = карта пройдена

---

## 8. ОЧКИ БОССА (Boss Points)

### Обновление очков
```
ConsumMgr:ResreshConsumData SpAreaBossPoint_430127 Value 8
```
- `SpAreaBossPoint_430127` — ID типа очков (связан с областью?)
- `Value 8` — новое количество очков

**ПРИМЕНЕНИЕ:** Отслеживание накопленных boss points для Netherrealm

---

## 9. ПЕРЕХОД МЕЖДУ КАРТАМИ

### Выход с карты
```
PageApplyBase@ _UpdateGameEnd: LastSceneName = .../DD_TanXiZhiQiang000 NextSceneName = .../XZ_YuJinZhiXiBiNanSuo200
UGameMgr::ExitLevel()
```

### Время на карте (вычисляемое)
- Старт: `SwitchBattleAreaUtil:_JoinFight {name}:{areaId}` (timestamp)
- Конец: `UGameMgr::ExitLevel()` (timestamp)
- Дельта = время прохождения карты

---

## 11. BEACON И ОБЛАСТЬ

### Запрос количества beacon'ов
```
MysteryItemMgr@GetBeaconNumByAreaId  AreaId == 1300AreaLv = 6
```
- Показывает сколько beacon'ов доступно для области
- НЕ показывает факт расхода beacon'а

### Вход в область
```
NetGameMgr:OnEnterAreaBegin()
SwitchBattleAreaUtil:JoinFight() EnterArea !!!!!!
EnterArea success!!!!!!
NetGameMgr:OnEnterAreaEnd()
```

---

## 12. ЧТО НЕ НАЙДЕНО (пока)

❌ **Расход beacon/resonator** — нет явного лога "потрачен 1 beacon"  
   - Есть только запрос `GetBeaconNumByAreaId` — показывает сколько доступно, не сколько потрачено  
   - Возможно, расход логируется где-то в других файлах или при другом действии  

❌ **Содержимое инвентаря** — не логируется при открытии  
   - `PCBag Run` — только факт открытия UI, без списка предметов  

❌ **Изменения инвентаря** — delta предметов не видна в этих логах  
   - Нельзя автоматически отследить расходы  

❌ **Точное название карты** — только технический путь  
   - `/Game/Art/Maps/04DD/DD_TanXiZhiQiang000` — нужен маппинг на читаемое имя

---

## 13. UI СОБЫТИЯ (полезно для контекста)

### Открытие/закрытие инвентаря
```
PageApplyBase@ OpenFlow3:OpenView = PCBag
PCBag Run
PCBag Destory
```

### Клики по табам инвентаря
```
PCBagTabButtonItem101_Button_Tab  ← Tab 1
PCBagTabButtonItem102_Button_Tab  ← Tab 2
PCBagTabButtonItem103_Button_Tab  ← Tab 3
```

### Открытие настроек
```
PageApplyBase@ OpenFlow3:OpenView = Setting
Setting Run
Setting Destory
```

---

## 14. ИДЕИ ДЛЯ РЕАЛИЗАЦИИ

### Высокий приоритет
1. **Счётчик карт** — парсить `BeginLoadingScreen`/`EndLoadingScreen`
2. **Тип активности** — `maptype [Mystic]`
3. **Имя персонажа** — `_JoinFight {name}:{areaId}`
4. **Счётчик боссов** — `AudioBGM Destory BOSS`
5. **Время на карте** — от `_JoinFight` до `ExitLevel`

### Средний приоритет
6. **Оценка жирности карты** — по количеству elite/rare монстров
7. **Drop Quantity %** — парсить из описаний аффиксов и суммировать
8. **Детект Dyed Snow Monster** — ID 3001000
9. **Детект Treasure Sentries** — ID 6100003
10. **Boss Points** — `ConsumMgr:ResreshConsumData SpAreaBossPoint`

### Низкий приоритет (требует доп. исследования)
11. **Расход consumables** — нужно искать другие логи
12. **Маппинг AreaId → название** — составить словарь
13. **Маппинг Monster NPC ID → название** — составить словарь
14. **Маппинг Modifier ID → описание** — составить словарь

---

## 15. REGEX ПАТТЕРНЫ

### Загрузка карты
```regex
Loading@ BeginLoadingScreen MapName = (.+)
Loading@ EndLoadingScreen MapName = (.+), CostTime = ([\d.]+)
```

### Вход в бой
```regex
SwitchBattleAreaUtil:_JoinFight (\w+):(\d+)
```

### Тип карты
```regex
\+maptype \[(\w+)\]
```

### Монстры
```regex
MonsterSpawner: total number (\d+)
MonsterSpawner: rarity type (\d+), number (\d+)
UCBornsUtils:.+E_elite:(\d+), E_reinforce:(\d+), E_normal:(\d+)
```

### Модификаторы
```regex
AddMapModifier\((\d+)\) success
\+Description \[(.+)\]
\+Id \[(\d+)\]
```

### Drop Quantity (для подсчёта суммы)
```regex
\+(\d+)% (?:additional )?(?:<e id=507>)?Drop Quantity
```

### Создание монстра
```regex
create monster npc id (\d+) rarity (\d+) affix[:\s]+(\d+|null)
Monster Created: npc id (\d+) rarity (\d+)
```

### Бой с боссом
```regex
AudioBGM PushNewState BOSS
AudioBGM Destory BOSS
Play_vo_hero_\d+_kill_boss
Play_vo_hero_\d+_kill_elite
```

### Выход с карты
```regex
UGameMgr::ExitLevel\(\)
```

### Порталы
```regex
Create Map Portal cfgId (\d+) etyId (\d+) uId (\d+)
```

### Boss Points
```regex
ConsumMgr:ResreshConsumData SpAreaBossPoint_(\d+) Value (\d+)
```

---

## 16. 🔥 ПОДБОР ПРЕДМЕТОВ (ВАЖНО!)

### Полная последовательность событий

#### 1. Предмет выпадает на землю
```
UDropInstanceMgr@ AddInstance! InstanceId = 28 InstanceIndex = 0
UDropInstanceMgr@ AddInstance! InstanceId = 29 InstanceIndex = 1
```
- **InstanceId** — уникальный ID дроп-экземпляра
- **InstanceIndex** — позиция в очереди (0, 1, 2...)
- ❌ **НЕТ информации о типе предмета!**

#### 2. Игрок подбирает предмет(ы)
```
ItemChange@ ProtoName=PickItems start
ItemChange@ Update Id=5028_edea9558-... BagNum=979 in PageId=102 SlotId=6
BagMgr@:Modfy BagItem PageId = 102 SlotId = 6 ConfigBaseId = 5028 Num = 979
ItemChange@ Update Id=5140_36679342-... BagNum=240 in PageId=102 SlotId=23
BagMgr@:Modfy BagItem PageId = 102 SlotId = 23 ConfigBaseId = 5140 Num = 240
ItemChange@ ProtoName=PickItems end
UDropInstanceMgr@ RemoveInstances!
```

### Расшифровка полей
- **ProtoName=PickItems** — тип события (подбор)
- **ConfigBaseId** — ID типа предмета!
- **Num** — **ТЕКУЩЕЕ** количество в инвентаре (НЕ сколько подобрали!)
- **PageId = 102** — вкладка инвентаря (102 = основные ресурсы?)
- **SlotId** — слот в инвентаре

### ⚠️ КАК ВЫЧИСЛИТЬ СКОЛЬКО ПОДОБРАЛИ

**Num — это текущий баланс, НЕ дельта!**

Для подсчёта подобранного нужно:
1. Хранить словарь `{ConfigBaseId: last_Num}`
2. При каждом `BagMgr@:Modfy` вычислять: `picked = current_Num - last_Num`
3. Обновлять `last_Num = current_Num`

### Пример анализа ConfigBaseId = 100200 (FE)
```
Num = 428 → 432 → 434 → 438 → 440 → 443 → 445 → 446 → 448 → 450
Дельты:  +4   +2    +4    +2    +3    +2    +1    +2    +2
```
**Всего подобрано: 22 FE за эту сессию**

### Подбор нескольких предметов за раз
Если игрок подобрал несколько предметов одновременно, они ВСЕ логируются между `start` и `end`:
```
ItemChange@ ProtoName=PickItems start
BagMgr@:Modfy ... ConfigBaseId = 5028 Num = 979    ← предмет 1
BagMgr@:Modfy ... ConfigBaseId = 5140 Num = 240    ← предмет 2
ItemChange@ ProtoName=PickItems end
```

### Найденные ConfigBaseId
| ConfigBaseId | Предмет | Slot |
|--------------|---------|------|
| 100200 | Flame Elementium (FE) — основная валюта! | 2 |
| 5028 | ? (валюта/ресурс) | 6 |
| 5140 | ? (валюта/ресурс) | 23 |
| 5080 | ? (валюта/ресурс) | 8 |

### Звуки подбора (для определения типа)
```
Play_UI_Drop_PickUp_Fire       ← огненный предмет (FE?)
Play_UI_PickUp_Currency_Crystal ← валюта Crystal
```

### Regex
```regex
UDropInstanceMgr@ AddInstance! InstanceId = (\d+) InstanceIndex = (\d+)
UDropInstanceMgr@ RemoveInstances!
ItemChange@ ProtoName=PickItems (start|end)
BagMgr@:Modfy BagItem PageId = (\d+) SlotId = (\d+) ConfigBaseId = (\d+) Num = (\d+)
```

### 🎯 ВЫВОДЫ

| Вопрос | Ответ |
|--------|-------|
| Можно ли понять сколько подобрали? | ✅ Да, вычислением дельты Num |
| Различить стак vs единичный предмет? | ⚠️ Частично — по дельте (дельта > 1 = стак) |
| Логируется выпадение на землю? | ✅ Да, но без типа предмета |
| Логируется тип предмета при подборе? | ✅ Да, ConfigBaseId |

---

## 17. ДОПОЛНИТЕЛЬНЫЕ НАХОДКИ

### Опыт персонажа
```
ExpMgr@UpdateExp Percent:16336415 92
ExpMgr@UpdateExp Percent:21480568 92
```
- Первое число — кумулятивный опыт
- Второе число (92) — уровень персонажа

**ПРИМЕНЕНИЕ:** Отслеживание прогресса опыта, детект level-up

### TCP Ping (задержка сети)
```
TCP Ping Result: 56
TCP Ping Result: 72
```
**ПРИМЕНЕНИЕ:** Мониторинг качества соединения, детект лагов

### Диалоги с NPC
```
DialogueMgr@:ShowDialogue STT! DialogueId = 627233976
DialogueMgr@:Show STT! DialogueId, DialogueDesc = 627233976 -> "Conflicts carry the weight..."
DialogueMgr@ShowDialogueView END[CallBack]! DialogueId = 627233976
```
**ПРИМЕНЕНИЕ:** Детект разговоров с NPC

### Magic Cube (награды сезона 3)
```
PageApplyBase@ OpenFlow3:OpenView = S3GamePlayMagicCube
S3GamePlayMagicCube Run
MagicCubeRewardItem:PlayAni IconExRewardAnim Start
MagicCubeRewardItem:PlayAni IconPitchGetAnim Start   ← получение предмета
MagicCubeRewardItem:PlayAni IconBonusAnim Start      ← бонусная награда
```
**ПРИМЕНЕНИЕ:** Детект получения наград из Magic Cube

### Netherrealm Area Data
```
MysteryModel@CreateMysticAreaItemData AreaId = 1200
MysteryModel@CreateMysticAreaItemData AreaCurRank = 6   ← текущий ранг области!
MysteryCardMgr@GetAreaBuffCards AreaId == 1300
```
**ПРИМЕНЕНИЕ:** Получение ранга области Netherrealm

### Открытие UI страниц
```
PageApplyBase@ OpenFlow3:OpenView = Mystery
PageApplyBase@ OpenFlow3:OpenView = MysteryArea
PageApplyBase@ OpenFlow3:OpenView = MysteryMapDetail
PageApplyBase@ OpenFlow3:OpenView = PCBag
PageApplyBase@ OpenFlow3:OpenView = Setting
```
**ПРИМЕНЕНИЕ:** Детект какую панель открыл игрок

---

## 17. REGEX ПАТТЕРНЫ (дополнительные)

### Опыт
```regex
ExpMgr@UpdateExp Percent:(\d+) (\d+)
```

### Пинг
```regex
TCP Ping Result: (\d+)
```

### Диалоги
```regex
DialogueMgr@:ShowDialogue STT! DialogueId = (\d+)
DialogueMgr@ShowDialogueView END
```

### Magic Cube
```regex
PageApplyBase@ OpenFlow3:OpenView = S3GamePlayMagicCube
MagicCubeRewardItem:PlayAni (\w+) (Start|End)
```

### Netherrealm Area
```regex
MysteryModel@CreateMysticAreaItemData AreaId = (\d+)
MysteryModel@CreateMysticAreaItemData AreaCurRank = (\d+)
```

---

*Последнее обновление: 2026-01-14 (добавлено: полный анализ подбора предметов, выпадение на землю, вычисление дельты)*
