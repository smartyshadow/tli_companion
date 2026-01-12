//! Парсер логов Torchlight Infinite
//! 
//! Парсит UE_game.log для извлечения событий:
//! - Подбор предметов (PickItems)
//! - Оценка цен на аукционе (XchgSearchPrice)
//! - Смена карты (MapChange)

use regex::Regex;
use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};
use crate::types::{ItemDropEvent, PriceSearchEvent, MapChangeEvent, MapEventType, LogEvent};
use std::collections::HashMap;
use log::{debug, trace};

/// Парсер логов TLI
pub struct LogParser {
    // Regex для временной метки
    timestamp_re: Regex,
    // Regex для начала блока подбора
    pick_start_re: Regex,
    // Regex для конца блока подбора
    pick_end_re: Regex,
    // Regex для изменения инвентаря
    bag_modify_re: Regex,
    // Regex для отправки запроса цены
    price_send_re: Regex,
    // Regex для получения ответа цены
    price_recv_re: Regex,
    // Regex для ID предмета в запросе цены
    price_refer_re: Regex,
    // Regex для цены в ответе
    price_unit_re: Regex,
    // Regex для дополнительных unitPrices строк в ответе (вложенные +N [price])
    price_unit_cont_re: Regex,
    // Regex для валюты в ответе
    price_currency_re: Regex,
    // Regex для смены карты
    map_change_re: Regex,
    // Regex для убежища
    hideout_re: Regex,
    
    // Состояние парсера
    /// Текущее количество предметов по слотам (для вычисления дельты)
    slot_quantities: HashMap<(i32, i32), i32>,
    /// Слоты которые уже инициализированы (первый подбор не считается)
    initialized_slots: std::collections::HashSet<(i32, i32)>,
    /// Находимся ли в блоке PickItems
    in_pick_block: bool,
    /// Текущий запрос цены (sync_id -> game_id)
    pending_price_requests: HashMap<i32, i64>,
    /// Последний sync_id из SendMessage (для связывания с refer)
    last_price_sync_id: Option<i32>,
}

impl LogParser {
    /// Создать новый парсер
    pub fn new() -> Self {
        Self {
            // Временная метка: [2026.01.12-11.34.07:799]
            timestamp_re: Regex::new(r"\[(\d{4})\.(\d{2})\.(\d{2})-(\d{2})\.(\d{2})\.(\d{2}):(\d{3})\]").unwrap(),
            
            // ItemChange@ ProtoName=PickItems start
            pick_start_re: Regex::new(r"ItemChange@ ProtoName=PickItems start").unwrap(),
            
            // ItemChange@ ProtoName=PickItems end
            pick_end_re: Regex::new(r"ItemChange@ ProtoName=PickItems end").unwrap(),
            
            // BagMgr@:Modfy BagItem PageId = 102 SlotId = 1 ConfigBaseId = 100200 Num = 904
            bag_modify_re: Regex::new(
                r"BagMgr@:Modfy BagItem PageId = (\d+) SlotId = (\d+) ConfigBaseId = (\d+) Num = (\d+)"
            ).unwrap(),
            
            // ----Socket SendMessage STT----XchgSearchPrice----SynId = 4006
            price_send_re: Regex::new(r"----Socket SendMessage STT----XchgSearchPrice----SynId = (\d+)").unwrap(),
            
            // ----Socket RecvMessage STT----XchgSearchPrice----SynId = 4006
            price_recv_re: Regex::new(r"----Socket RecvMessage STT----XchgSearchPrice----SynId = (\d+)").unwrap(),
            
            // +refer [101010037_15] или |       | +refer [200029]
            // Нам нужен только ведущий числовой game_id до '_' (если он есть)
            price_refer_re: Regex::new(r"\+refer \[(\d+)").unwrap(),
            
            // +unitPrices+1 [1.0]
            price_unit_re: Regex::new(r"\+unitPrices\+\d+ \[([\d.]+)\]").unwrap(),

            // continuation: |      | |          +2 [1.0] (формат варьируется по отступам)
            price_unit_cont_re: Regex::new(r"^\s*\|.*\+\d+ \[([\d.]+)\]").unwrap(),
            
            // +currency [100300]
            price_currency_re: Regex::new(r"\+currency \[(\d+)\]").unwrap(),
            
            // PageApplyBase@ _UpdateGameEnd: ... NextSceneName = World'/Game/Art/Maps/...'
            // caps[1] = /Game/Art/Maps/...
            map_change_re: Regex::new(
                r"PageApplyBase@\s*_UpdateGameEnd:.*NextSceneName\s*=\s*World'(/Game/Art/Maps[^']*)'"
            ).unwrap(),
            
            // Убежище: XZ_YuJinZhiXiBiNanSuo200
            hideout_re: Regex::new(r"XZ_YuJinZhiXiBiNanSuo200").unwrap(),
            
            slot_quantities: HashMap::new(),
            initialized_slots: std::collections::HashSet::new(),
            in_pick_block: false,
            pending_price_requests: HashMap::new(),
            last_price_sync_id: None,
        }
    }
    
    /// Парсить временную метку из строки лога
    fn parse_timestamp(&self, line: &str) -> Option<DateTime<Utc>> {
        let caps = self.timestamp_re.captures(line)?;
        
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        let day: u32 = caps.get(3)?.as_str().parse().ok()?;
        let hour: u32 = caps.get(4)?.as_str().parse().ok()?;
        let min: u32 = caps.get(5)?.as_str().parse().ok()?;
        let sec: u32 = caps.get(6)?.as_str().parse().ok()?;
        let millis: u32 = caps.get(7)?.as_str().parse().ok()?;
        
        let naive = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, day)?,
            chrono::NaiveTime::from_hms_milli_opt(hour, min, sec, millis)?
        );
        
        Some(Utc.from_utc_datetime(&naive))
    }
    
    /// Парсить одну строку лога
    /// Возвращает Option<LogEvent> если строка содержит интересное событие
    pub fn parse_line(&mut self, line: &str) -> Option<LogEvent> {
        // Проверяем начало/конец блока PickItems
        if self.pick_start_re.is_match(line) {
            self.in_pick_block = true;
            debug!(">>> Entered PickItems block");
            return None;
        }
        
        if self.pick_end_re.is_match(line) {
            self.in_pick_block = false;
            debug!("<<< Exited PickItems block");
            return None;
        }
        
        // Парсим изменение инвентаря (только внутри блока PickItems)
        if self.in_pick_block {
            debug!("Processing line in PickItems block: {}", &line[..line.len().min(100)]);
            if let Some(event) = self.parse_bag_modify(line) {
                return Some(LogEvent::ItemDrop(event));
            }
        }
        
        // Парсим запрос цены (SendMessage)
        if let Some(sync_id) = self.parse_price_send(line) {
            // Сохраняем sync_id для связывания с refer в следующих строках
            self.last_price_sync_id = Some(sync_id);
            debug!("Price request started: sync_id={}", sync_id);
            return None;
        }
        
        // Если есть refer в строке — связываем с последним sync_id
        if let Some(caps) = self.price_refer_re.captures(line) {
            if let Ok(game_id) = caps.get(1).unwrap().as_str().parse::<i64>() {
                if let Some(sync_id) = self.last_price_sync_id {
                    self.pending_price_requests.insert(sync_id, game_id);
                    debug!("Price request item: sync_id={}, game_id={}", sync_id, game_id);
                }
            }
        }
        
        // Парсим ответ с ценами
        if let Some(event) = self.parse_price_recv(line) {
            return Some(LogEvent::PriceSearch(event));
        }
        
        // Парсим смену карты
        if let Some(event) = self.parse_map_change(line) {
            return Some(LogEvent::MapChange(event));
        }
        
        None
    }

    /// Warm-up режим: обновляет baseline количества по слотам без генерации событий.
    ///
    /// Нужен, чтобы после старта приложения (когда мы tail'им с конца файла) у нас был
    /// baseline для стаков — иначе первый дроп в слот часто теряется.
    pub fn warmup_line(&mut self, line: &str) {
        if let Some(caps) = self.bag_modify_re.captures(line) {
            let page_id: i32 = match caps.get(1).and_then(|m| m.as_str().parse().ok()) {
                Some(v) => v,
                None => return,
            };
            let slot_id: i32 = match caps.get(2).and_then(|m| m.as_str().parse().ok()) {
                Some(v) => v,
                None => return,
            };
            let new_quantity: i32 = match caps.get(4).and_then(|m| m.as_str().parse().ok()) {
                Some(v) => v,
                None => return,
            };

            let slot_key = (page_id, slot_id);
            self.slot_quantities.insert(slot_key, new_quantity);
            self.initialized_slots.insert(slot_key);
        }
    }
    
    /// Парсить изменение инвентаря
    fn parse_bag_modify(&mut self, line: &str) -> Option<ItemDropEvent> {
        let caps = self.bag_modify_re.captures(line)?;
        
        let page_id: i32 = caps.get(1)?.as_str().parse().ok()?;
        let slot_id: i32 = caps.get(2)?.as_str().parse().ok()?;
        let game_id: i64 = caps.get(3)?.as_str().parse().ok()?;
        let new_quantity: i32 = caps.get(4)?.as_str().parse().ok()?;
        
        let slot_key = (page_id, slot_id);
        
        // Проверяем, инициализирован ли этот слот
        if !self.initialized_slots.contains(&slot_key) {
            // Первое событие для этого слота
            self.initialized_slots.insert(slot_key);
            
            // Мы не знаем базовое количество предмета у пользователя до начала трекинга.
            // Вместо того, чтобы “терять” первый дроп, считаем минимально возможный дроп = 1,
            // если текущее количество > 0. Дальше будем считать строго по дельте.
            self.slot_quantities.insert(slot_key, new_quantity);

            if new_quantity <= 0 {
                trace!("Initialized empty slot {:?} (game_id={})", slot_key, game_id);
                return None;
            }

            let timestamp = self.parse_timestamp(line).unwrap_or_else(Utc::now);
            debug!(
                "First seen slot {:?} baseline={}, counting minimal drop=1 (game_id={})",
                slot_key, new_quantity, game_id
            );

            return Some(ItemDropEvent {
                game_id,
                quantity: 1,
                timestamp,
                page_id,
                slot_id,
            });
        }
        
        let old_quantity = self.slot_quantities.get(&slot_key).copied().unwrap_or(0);
        
        // Вычисляем дельту (сколько реально подобрали)
        let delta = new_quantity - old_quantity;
        
        // Обновляем кэш количества
        self.slot_quantities.insert(slot_key, new_quantity);
        
        // Если дельта <= 0, это не подбор, а использование/перемещение
        if delta <= 0 {
            trace!("Skipping non-pickup event: game_id={}, delta={}", game_id, delta);
            return None;
        }
        
        let timestamp = self.parse_timestamp(line).unwrap_or_else(Utc::now);
        
        debug!("Item picked up: game_id={}, quantity={}, page={}, slot={}", 
               game_id, delta, page_id, slot_id);
        
        Some(ItemDropEvent {
            game_id,
            quantity: delta,
            timestamp,
            page_id,
            slot_id,
        })
    }
    
    /// Парсить отправку запроса цены
    fn parse_price_send(&mut self, line: &str) -> Option<i32> {
        let caps = self.price_send_re.captures(line)?;
        let sync_id: i32 = caps.get(1)?.as_str().parse().ok()?;
        Some(sync_id)
    }
    
    /// Парсить ответ с ценами
    fn parse_price_recv(&mut self, line: &str) -> Option<PriceSearchEvent> {
        let caps = self.price_recv_re.captures(line)?;
        let sync_id: i32 = caps.get(1)?.as_str().parse().ok()?;
        
        // Получаем game_id из pending запросов
        let game_id = self.pending_price_requests.remove(&sync_id)?;
        
        let timestamp = self.parse_timestamp(line).unwrap_or_else(Utc::now);
        
        // Цены будут в следующих строках, создаём событие с пустыми ценами
        // Они будут заполнены при парсинге следующих строк
        debug!("Price response received: sync_id={}, game_id={}", sync_id, game_id);
        
        Some(PriceSearchEvent {
            game_id,
            prices: Vec::new(), // Будут заполнены отдельно
            currency_id: 100300, // По умолчанию Flame Elementium
            timestamp,
            sync_id,
        })
    }
    
    /// Парсить смену карты
    fn parse_map_change(&self, line: &str) -> Option<MapChangeEvent> {
        let caps = self.map_change_re.captures(line)?;

        let timestamp = self.parse_timestamp(line).unwrap_or_else(Utc::now);

            // NextSceneName (для будущих фич — маппинг карты/типа контента)
            // Если не удалось — fallback на всю строку.
            let scene_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| line.to_string());

            // Определяем тип события строго по NextSceneName, иначе можно ошибочно матчить hideout в LastSceneName.
            let event_type = if self.hideout_re.is_match(&scene_name) {
                MapEventType::ExitToHideout
            } else {
                MapEventType::EnterMap
            };

        debug!("Map change: {:?} -> {}", event_type, scene_name);

        Some(MapChangeEvent {
            event_type,
            scene_name,
            timestamp,
        })
    }
    
    /// Парсить блок ответа с ценами (многострочный)
    /// Вызывается после получения PriceSearchEvent для извлечения цен
    pub fn parse_price_block(&self, lines: &[&str]) -> (Vec<f64>, i64) {
        let mut prices = Vec::new();
        let mut currency_id: i64 = 100300;
        
        for line in lines {
            // Извлекаем цены
            for caps in self.price_unit_re.captures_iter(line) {
                if let Ok(price) = caps.get(1).unwrap().as_str().parse::<f64>() {
                    prices.push(price);
                }
            }

            // В некоторых логах цены идут продолжением строк без "unitPrices+" (| | +2 [..]).
            if let Some(caps) = self.price_unit_cont_re.captures(line) {
                if let Ok(price) = caps.get(1).unwrap().as_str().parse::<f64>() {
                    prices.push(price);
                }
            }
            
            // Извлекаем валюту
            if let Some(caps) = self.price_currency_re.captures(line) {
                if let Ok(cid) = caps.get(1).unwrap().as_str().parse::<i64>() {
                    currency_id = cid;
                }
            }
        }
        
        (prices, currency_id)
    }
    
    /// Сбросить состояние слотов (при начале новой сессии)
    pub fn reset_slot_cache(&mut self) {
        self.slot_quantities.clear();
        self.initialized_slots.clear();
        self.pending_price_requests.clear();
        self.in_pick_block = false;
        self.last_price_sync_id = None;
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    
    #[test]
    fn test_parse_bag_modify() {
        let mut parser = LogParser::new();
        parser.in_pick_block = true;
        
        let line = "[2026.01.12-11.34.07:799][980]GameLog: Display: [Game] BagMgr@:Modfy BagItem PageId = 102 SlotId = 1 ConfigBaseId = 100200 Num = 50";
        
        let event = parser.parse_line(line);
        assert!(event.is_some());
        
        if let Some(LogEvent::ItemDrop(drop)) = event {
            assert_eq!(drop.game_id, 100200);
            // Первый раз видим слот — baseline неизвестен, считаем минимальный дроп = 1
            assert_eq!(drop.quantity, 1);
            assert_eq!(drop.page_id, 102);
            assert_eq!(drop.slot_id, 1);
        } else {
            panic!("Expected ItemDrop event");
        }
    }
    
    #[test]
    fn test_parse_timestamp() {
        let parser = LogParser::new();
        let line = "[2026.01.12-11.34.07:799]";
        
        let ts = parser.parse_timestamp(line);
        assert!(ts.is_some());
        
        let ts = ts.unwrap();
        assert_eq!(ts.year(), 2026);
        assert_eq!(ts.month(), 1);
        assert_eq!(ts.day(), 12);
    }
    
    #[test]
    fn test_delta_calculation() {
        let mut parser = LogParser::new();
        parser.in_pick_block = true;
        
        // Первый раз видим слот: baseline неизвестен, считаем минимальный дроп = 1
        let line1 = "[2026.01.12-11.34.07:799][980]GameLog: Display: [Game] BagMgr@:Modfy BagItem PageId = 102 SlotId = 1 ConfigBaseId = 100200 Num = 50";
        let event1 = parser.parse_line(line1);
        
        if let Some(LogEvent::ItemDrop(drop)) = event1 {
            assert_eq!(drop.quantity, 1);
        }
        
        // Второй подбор: 50 -> 75 = +25
        let line2 = "[2026.01.12-11.34.08:799][980]GameLog: Display: [Game] BagMgr@:Modfy BagItem PageId = 102 SlotId = 1 ConfigBaseId = 100200 Num = 75";
        let event2 = parser.parse_line(line2);
        
        if let Some(LogEvent::ItemDrop(drop)) = event2 {
            assert_eq!(drop.quantity, 25);
        }
    }
}
