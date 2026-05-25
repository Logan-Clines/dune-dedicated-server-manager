use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const ITEMS_JSON: &[u8] = include_bytes!("../../data/items.json");
const VEHICLES_JSON: &[u8] = include_bytes!("../../data/vehicles.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub actor_class: String,
    #[serde(default)]
    pub templates: Vec<String>,
}

static ITEMS: Lazy<Vec<Item>> =
    Lazy::new(|| serde_json::from_slice(ITEMS_JSON).expect("embedded items.json is valid"));
static VEHICLES: Lazy<Vec<Vehicle>> =
    Lazy::new(|| serde_json::from_slice(VEHICLES_JSON).expect("embedded vehicles.json is valid"));

pub fn items() -> &'static [Item] {
    &ITEMS
}

pub fn vehicles() -> &'static [Vehicle] {
    &VEHICLES
}

pub fn search_items(query: &str, limit: u32) -> Vec<Item> {
    let q = query.trim().to_lowercase();
    let cap = limit.clamp(1, 200) as usize;
    let all = items();
    if q.is_empty() {
        return all.iter().take(50.min(cap)).cloned().collect();
    }
    let mut scored: Vec<(u32, &Item)> = all
        .iter()
        .filter_map(|item| {
            let s = score_text(&q, &item.id, &item.name);
            if s > 0 {
                Some((s, item))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(cap).map(|(_, it)| it.clone()).collect()
}

pub fn search_vehicles(query: &str, limit: u32) -> Vec<Vehicle> {
    let q = query.trim().to_lowercase();
    let cap = limit.clamp(1, 100) as usize;
    let all = vehicles();
    if q.is_empty() {
        return all.iter().take(20.min(cap)).cloned().collect();
    }
    let mut scored: Vec<(u32, &Vehicle)> = all
        .iter()
        .filter_map(|v| {
            let s = score_text(&q, &v.id, &v.actor_class);
            if s > 0 {
                Some((s, v))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(cap).map(|(_, v)| v.clone()).collect()
}

/// Faithful port of `scoreText` from `src/admin/data.ts`.
/// 1000 = exact id, 500 = id prefix, 300 = name prefix, 200 = id contains,
/// 100 = name contains, 50 = all words present.
fn score_text(query: &str, id: &str, name: &str) -> u32 {
    let id_lower = id.to_lowercase();
    let name_lower = name.to_lowercase();
    if id_lower == query {
        return 1000;
    }
    let mut score = 0;
    if id_lower.starts_with(query) {
        score = score.max(500);
    }
    if name_lower.starts_with(query) {
        score = score.max(300);
    }
    if score == 0 && id_lower.contains(query) {
        score = 200;
    }
    if score == 0 && name_lower.contains(query) {
        score = 100;
    }
    if score == 0 {
        let words: Vec<&str> = query.split_whitespace().collect();
        if words.len() > 1 && words.iter().all(|w| id_lower.contains(w) || name_lower.contains(w)) {
            score = 50;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_load_and_have_rows() {
        assert!(!items().is_empty());
        assert!(!vehicles().is_empty());
    }

    #[test]
    fn scoring_prefers_exact_id() {
        let id = items()[0].id.clone();
        let results = search_items(&id, 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, id);
    }

    #[test]
    fn empty_query_returns_default_slice() {
        let r = search_items("", 50);
        assert!(!r.is_empty());
        assert!(r.len() <= 50);
    }
}
