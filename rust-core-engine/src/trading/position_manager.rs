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

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
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

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_position(
        id: &str,
        symbol: &str,
        side: &str,
        size: f64,
        entry_price: f64,
        current_price: f64,
    ) -> Position {
        Position {
            id: id.to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            size,
            entry_price,
            current_price,
            unrealized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            timestamp: 1234567890,
        }
    }

    #[test]
    fn test_new_position_manager() {
        let manager = PositionManager::new();
        assert_eq!(manager.get_position_count(), 0);
    }

    #[test]
    fn test_default_position_manager() {
        let manager = PositionManager::default();
        assert_eq!(manager.get_position_count(), 0);
    }

    #[test]
    fn test_add_position() {
        let manager = PositionManager::new();
        let position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);

        manager.add_position(position.clone());

        assert_eq!(manager.get_position_count(), 1);
        assert!(manager.has_position("BTCUSDT"));

        let retrieved = manager.get_position("BTCUSDT").unwrap();
        assert_eq!(retrieved.id, "pos1");
        assert_eq!(retrieved.size, 0.1);
    }

    #[test]
    fn test_add_multiple_positions() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));
        manager.add_position(create_test_position(
            "pos3", "BNBUSDT", "BUY", 5.0, 300.0, 310.0,
        ));

        assert_eq!(manager.get_position_count(), 3);
    }

    #[test]
    fn test_update_position() {
        let manager = PositionManager::new();
        let mut position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);

        manager.add_position(position.clone());

        // Update the position with new price
        position.current_price = 52000.0;
        position.unrealized_pnl = 200.0;
        manager.update_position(position);

        let updated = manager.get_position("BTCUSDT").unwrap();
        assert_eq!(updated.current_price, 52000.0);
        assert_eq!(updated.unrealized_pnl, 200.0);
    }

    #[test]
    fn test_remove_position_by_id() {
        let manager = PositionManager::new();
        let position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);

        manager.add_position(position);
        assert_eq!(manager.get_position_count(), 1);

        let removed = manager.remove_position("pos1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "pos1");
        assert_eq!(manager.get_position_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_position() {
        let manager = PositionManager::new();
        let removed = manager.remove_position("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_get_position() {
        let manager = PositionManager::new();
        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));

        let position = manager.get_position("BTCUSDT");
        assert!(position.is_some());
        assert_eq!(position.unwrap().symbol, "BTCUSDT");

        let no_position = manager.get_position("ETHUSDT");
        assert!(no_position.is_none());
    }

    #[test]
    fn test_has_position() {
        let manager = PositionManager::new();
        assert!(!manager.has_position("BTCUSDT"));

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        assert!(manager.has_position("BTCUSDT"));
        assert!(!manager.has_position("ETHUSDT"));
    }

    #[test]
    fn test_get_all_positions() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));

        let all_positions = manager.get_all_positions();
        assert_eq!(all_positions.len(), 2);
    }

    #[test]
    fn test_get_total_unrealized_pnl() {
        let manager = PositionManager::new();

        let mut pos1 = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);
        pos1.unrealized_pnl = 100.0;

        let mut pos2 = create_test_position("pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0);
        pos2.unrealized_pnl = 100.0;

        let mut pos3 = create_test_position("pos3", "BNBUSDT", "BUY", 5.0, 300.0, 295.0);
        pos3.unrealized_pnl = -25.0;

        manager.add_position(pos1);
        manager.add_position(pos2);
        manager.add_position(pos3);

        assert_eq!(manager.get_total_unrealized_pnl(), 175.0);
    }

    #[test]
    fn test_get_total_unrealized_pnl_zero() {
        let manager = PositionManager::new();
        assert_eq!(manager.get_total_unrealized_pnl(), 0.0);
    }

    #[test]
    fn test_get_total_unrealized_pnl_negative() {
        let manager = PositionManager::new();

        let mut pos1 = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 48000.0);
        pos1.unrealized_pnl = -200.0;

        let mut pos2 = create_test_position("pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 3100.0);
        pos2.unrealized_pnl = -100.0;

        manager.add_position(pos1);
        manager.add_position(pos2);

        assert_eq!(manager.get_total_unrealized_pnl(), -300.0);
    }

    #[test]
    fn test_get_position_count() {
        let manager = PositionManager::new();
        assert_eq!(manager.get_position_count(), 0);

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        assert_eq!(manager.get_position_count(), 1);

        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));
        assert_eq!(manager.get_position_count(), 2);

        manager.remove_position("pos1");
        assert_eq!(manager.get_position_count(), 1);
    }

    #[test]
    fn test_get_positions_by_side_buy() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));
        manager.add_position(create_test_position(
            "pos3", "BNBUSDT", "BUY", 5.0, 300.0, 310.0,
        ));

        let buy_positions = manager.get_positions_by_side("BUY");
        assert_eq!(buy_positions.len(), 2);

        for pos in buy_positions {
            assert_eq!(pos.side, "BUY");
        }
    }

    #[test]
    fn test_get_positions_by_side_sell() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));
        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));
        manager.add_position(create_test_position(
            "pos3", "BNBUSDT", "SELL", 5.0, 300.0, 290.0,
        ));

        let sell_positions = manager.get_positions_by_side("SELL");
        assert_eq!(sell_positions.len(), 2);

        for pos in sell_positions {
            assert_eq!(pos.side, "SELL");
        }
    }

    #[test]
    fn test_get_positions_by_side_empty() {
        let manager = PositionManager::new();

        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));

        let sell_positions = manager.get_positions_by_side("SELL");
        assert_eq!(sell_positions.len(), 0);
    }

    #[test]
    fn test_get_exposure_for_symbol() {
        let manager = PositionManager::new();

        let position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);
        manager.add_position(position);

        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert_eq!(exposure, 0.1 * 51000.0); // size * current_price
        assert_eq!(exposure, 5100.0);
    }

    #[test]
    fn test_get_exposure_for_symbol_nonexistent() {
        let manager = PositionManager::new();
        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert_eq!(exposure, 0.0);
    }

    #[test]
    fn test_get_exposure_for_symbol_zero_price() {
        let manager = PositionManager::new();

        let position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 0.0);
        manager.add_position(position);

        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert_eq!(exposure, 0.0);
    }

    #[test]
    fn test_get_total_exposure() {
        let manager = PositionManager::new();

        // BTC: 0.1 * 51000 = 5100
        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));

        // ETH: 1.0 * 2900 = 2900
        manager.add_position(create_test_position(
            "pos2", "ETHUSDT", "SELL", 1.0, 3000.0, 2900.0,
        ));

        // BNB: 5.0 * 310 = 1550
        manager.add_position(create_test_position(
            "pos3", "BNBUSDT", "BUY", 5.0, 300.0, 310.0,
        ));

        let total_exposure = manager.get_total_exposure();
        assert_eq!(total_exposure, 9550.0);
    }

    #[test]
    fn test_get_total_exposure_empty() {
        let manager = PositionManager::new();
        assert_eq!(manager.get_total_exposure(), 0.0);
    }

    #[test]
    fn test_position_with_stop_loss_and_take_profit() {
        let manager = PositionManager::new();

        let mut position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);
        position.stop_loss = Some(49000.0);
        position.take_profit = Some(55000.0);

        manager.add_position(position);

        let retrieved = manager.get_position("BTCUSDT").unwrap();
        assert_eq!(retrieved.stop_loss, Some(49000.0));
        assert_eq!(retrieved.take_profit, Some(55000.0));
    }

    #[test]
    fn test_position_clone() {
        let position = create_test_position("pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0);
        let cloned = position.clone();

        assert_eq!(position.id, cloned.id);
        assert_eq!(position.symbol, cloned.symbol);
        assert_eq!(position.size, cloned.size);
    }

    #[test]
    fn test_position_manager_clone() {
        let manager = PositionManager::new();
        manager.add_position(create_test_position(
            "pos1", "BTCUSDT", "BUY", 0.1, 50000.0, 51000.0,
        ));

        let cloned_manager = manager.clone();
        assert_eq!(cloned_manager.get_position_count(), 1);
        assert!(cloned_manager.has_position("BTCUSDT"));
    }

    #[test]
    fn test_extreme_values() {
        let manager = PositionManager::new();

        // Test with very large values
        let large_position =
            create_test_position("pos1", "BTCUSDT", "BUY", 1000.0, 100000.0, 100000.0);
        manager.add_position(large_position);

        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert_eq!(exposure, 100_000_000.0);
    }

    #[test]
    fn test_very_small_values() {
        let manager = PositionManager::new();

        // Test with very small values
        let small_position =
            create_test_position("pos1", "BTCUSDT", "BUY", 0.00001, 50000.0, 50001.0);
        manager.add_position(small_position);

        let exposure = manager.get_exposure_for_symbol("BTCUSDT");
        assert!((exposure - 0.50001).abs() < 0.000001); // Float comparison with tolerance
    }
}
