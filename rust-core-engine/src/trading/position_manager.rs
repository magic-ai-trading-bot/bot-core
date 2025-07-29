use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub symbol: String,
    pub side: String, // "BUY" or "SELL"
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct PositionManager {
    positions: Arc<DashMap<String, Position>>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(DashMap::new()),
        }
    }

    pub fn add_position(&self, position: Position) {
        self.positions.insert(position.symbol.clone(), position);
    }

    pub fn update_position(&self, position: Position) {
        self.positions.insert(position.symbol.clone(), position);
    }

    pub fn remove_position(&self, position_id: &str) -> Option<Position> {
        // Find position by ID (since we're using symbol as key, we need to search)
        for entry in self.positions.iter() {
            if entry.value().id == position_id {
                let symbol = entry.key().clone();
                return self.positions.remove(&symbol).map(|(_, pos)| pos);
            }
        }
        None
    }

    pub fn get_position(&self, symbol: &str) -> Option<Position> {
        self.positions
            .get(symbol)
            .map(|entry| entry.value().clone())
    }

    pub fn has_position(&self, symbol: &str) -> bool {
        self.positions.contains_key(symbol)
    }

    pub fn get_all_positions(&self) -> Vec<Position> {
        self.positions
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_total_unrealized_pnl(&self) -> f64 {
        self.positions
            .iter()
            .map(|entry| entry.value().unrealized_pnl)
            .sum()
    }

    pub fn get_position_count(&self) -> usize {
        self.positions.len()
    }

    pub fn get_positions_by_side(&self, side: &str) -> Vec<Position> {
        self.positions
            .iter()
            .filter(|entry| entry.value().side == side)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_exposure_for_symbol(&self, symbol: &str) -> f64 {
        if let Some(position) = self.get_position(symbol) {
            position.size * position.current_price
        } else {
            0.0
        }
    }

    pub fn get_total_exposure(&self) -> f64 {
        self.positions
            .iter()
            .map(|entry| {
                let pos = entry.value();
                pos.size * pos.current_price
            })
            .sum()
    }
}
