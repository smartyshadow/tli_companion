// Локальный кэш известных предметов
// В будущем будет загружаться с сервера

export interface ItemData {
  game_id: number;
  name_cn: string;
  name_en: string | null;
  name_ru: string | null;
  category: string;
  icon?: string;
}

// Базовый маппинг из БД (загружен вручную)
// TODO: загружать динамически с сервера
export const ITEMS_MAP: Record<number, ItemData> = {
  // === ВАЛЮТА ===
  100200: { game_id: 100200, name_cn: "初火灵砂", name_en: "Flame Sand", name_ru: "Огненный песок", category: "currency" },
  100300: { game_id: 100300, name_cn: "初火源质", name_en: "Flame Elementium", name_ru: "Огненный элементиум", category: "currency" },
  5011: { game_id: 5011, name_cn: "遗忘之水", name_en: "Elixir of Oblivion", name_ru: "Эликсир забвения", category: "currency" },
  5028: { game_id: 5028, name_cn: "异界回响", name_en: "Otherworld Echo", name_ru: "Эхо иномирья", category: "currency" },
  5029: { game_id: 5029, name_cn: "逆转发条", name_en: "Winding Key", name_ru: "Заводной ключ", category: "currency" },
  
  // === МАТЕРИАЛЫ ПАМЯТИ ===
  5140: { game_id: 5140, name_cn: "追忆碎絮", name_en: "Memory Scrap", name_ru: "Обрывок памяти", category: "memory_material" },
  5143: { game_id: 5143, name_cn: "追忆游丝-稀有", name_en: "Memory Thread - Rare", name_ru: "Нить памяти (редкая)", category: "memory_material" },
  5144: { game_id: 5144, name_cn: "追忆游丝-卓越", name_en: "Memory Thread - Epic", name_ru: "Нить памяти (эпическая)", category: "memory_material" },
  
  // === МАТЕРИАЛЫ ЭКИПИРОВКИ ===
  5080: { game_id: 5080, name_cn: "能量核心", name_en: "Energy Core", name_ru: "Ядро энергии", category: "equipment_material" },
  200029: { game_id: 200029, name_cn: "稀世灰烬", name_en: "Rare Ember", name_ru: "Редкий пепел", category: "equipment_material" },
  
  // === МАТЕРИАЛЫ КУБА ===
  5201: { game_id: 5201, name_cn: "神格残片", name_en: "Divinity Fragment", name_ru: "Осколок божественности", category: "cube_material" },
  5202: { game_id: 5202, name_cn: "神格契约-残片", name_en: "Divinity Pact Fragment", name_ru: "Фрагмент божественного пакта", category: "cube_material" },
  5210: { game_id: 5210, name_cn: "神威辉石", name_en: "Divine Radiant Stone", name_ru: "Божественный камень", category: "cube_material" },
  5220: { game_id: 5220, name_cn: "升华之楔（魔法）", name_en: "Transcendence Wedge (Magic)", name_ru: "Клин возвышения (магия)", category: "cube_material" },
  5230: { game_id: 5230, name_cn: "升华之楔（稀有）", name_en: "Transcendence Wedge (Rare)", name_ru: "Клин возвышения (редкий)", category: "cube_material" },
  5240: { game_id: 5240, name_cn: "归一之楔", name_en: "Unifying Wedge", name_ru: "Объединяющий клин", category: "cube_material" },
  5250: { game_id: 5250, name_cn: "升华之楔（传奇）", name_en: "Transcendence Wedge (Legendary)", name_ru: "Клин возвышения (легендарный)", category: "cube_material" },
  
  // === СПЕЦИАЛЬНЫЕ ПРЕДМЕТЫ ===
  5030: { game_id: 5030, name_cn: "孪生倒影", name_en: "Twin Reflection", name_ru: "Двойное отражение", category: "special_item" },
  5031: { game_id: 5031, name_cn: "传奇降生之鸭", name_en: "Sprout of Legends", name_ru: "Росток легенд", category: "special_item" },
  
  // === БИЛЕТЫ AETERNA ===
  5310: { game_id: 5310, name_cn: "迷城残响-瞬息", name_en: "Aeterna Reverberation - Fleeting", name_ru: "Эхо Тайнограда - Мгновение", category: "gameplay_ticket" },
  5311: { game_id: 5311, name_cn: "迷城残响-永恒", name_en: "Aeterna Reverberation - Eternal", name_ru: "Эхо Тайнограда - Вечность", category: "gameplay_ticket" },
  
  // === MEMORY FLUORESCENCE (карты памяти) ===
  1001: { game_id: 1001, name_cn: "星星鹅火", name_en: "Starfire Goose", name_ru: "Звёздный гусь", category: "memory_fluorescence" },
  6002: { game_id: 6002, name_cn: "寒渊的秘密", name_en: "Secret of Cold Abyss", name_ru: "Тайна холодной бездны", category: "memory_fluorescence" },
  
  // === КАРТЫ/БИЛЕТЫ ===
  400006: { game_id: 400006, name_cn: "信标", name_en: "Beacon", name_ru: "Маяк", category: "map_ticket" },
  400014: { game_id: 400014, name_cn: "沸涌炎海的信标（时刻7）", name_en: "Blistering Lava Sea Beacon T7", name_ru: "Маяк кипящего моря T7", category: "map_ticket" },
};

// Получить название предмета
export function getItemName(gameId: number, lang: 'en' | 'ru' | 'cn' = 'ru'): string {
  const item = ITEMS_MAP[gameId];
  if (!item) {
    return `ID: ${gameId}`;
  }
  
  switch (lang) {
    case 'ru':
      return item.name_ru || item.name_en || item.name_cn;
    case 'en':
      return item.name_en || item.name_cn;
    case 'cn':
      return item.name_cn;
    default:
      return item.name_cn;
  }
}

// Получить информацию о предмете
export function getItemInfo(gameId: number): ItemData | null {
  return ITEMS_MAP[gameId] || null;
}

// Проверить известен ли предмет
export function isKnownItem(gameId: number): boolean {
  return gameId in ITEMS_MAP;
}
