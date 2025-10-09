#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub active_positions: usize,
    pub total_trades: u64,
    pub cache_size: usize,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub last_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingMetrics {
    pub total_pnl: f64,
    pub win_rate: f64,
    pub avg_trade_duration_minutes: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: Option<f64>,
    pub total_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub websocket_connected: bool,
    pub api_responsive: bool,
    pub last_data_update: i64,
    pub reconnect_count: u32,
}

pub struct MonitoringService {
    start_time: std::time::Instant,
    metrics: SystemMetrics,
    trading_metrics: TradingMetrics,
    connection_status: ConnectionStatus,
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            metrics: SystemMetrics {
                uptime_seconds: 0,
                active_positions: 0,
                total_trades: 0,
                cache_size: 0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                last_update: chrono::Utc::now().timestamp(),
            },
            trading_metrics: TradingMetrics {
                total_pnl: 0.0,
                win_rate: 0.0,
                avg_trade_duration_minutes: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: None,
                total_volume: 0.0,
            },
            connection_status: ConnectionStatus {
                websocket_connected: false,
                api_responsive: false,
                last_data_update: 0,
                reconnect_count: 0,
            },
        }
    }

    pub fn update_system_metrics(&mut self, active_positions: usize, cache_size: usize) {
        self.metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        self.metrics.active_positions = active_positions;
        self.metrics.cache_size = cache_size;
        self.metrics.last_update = chrono::Utc::now().timestamp();

        // In a real implementation, you would get actual memory and CPU usage
        self.metrics.memory_usage_mb = 50.0; // Placeholder
        self.metrics.cpu_usage_percent = 10.0; // Placeholder
    }

    pub fn update_trading_metrics(&mut self, stats: &crate::storage::PerformanceStats) {
        self.trading_metrics.total_pnl = stats.total_pnl;
        self.trading_metrics.win_rate = stats.win_rate;
        // Other metrics would be calculated here
    }

    pub fn update_connection_status(&mut self, websocket_connected: bool, api_responsive: bool) {
        self.connection_status.websocket_connected = websocket_connected;
        self.connection_status.api_responsive = api_responsive;
        self.connection_status.last_data_update = chrono::Utc::now().timestamp();
    }

    pub fn record_reconnect(&mut self) {
        self.connection_status.reconnect_count =
            self.connection_status.reconnect_count.saturating_add(1);
        warn!(
            "Connection reconnect #{}",
            self.connection_status.reconnect_count
        );
    }

    pub fn get_system_metrics(&self) -> &SystemMetrics {
        &self.metrics
    }

    pub fn get_trading_metrics(&self) -> &TradingMetrics {
        &self.trading_metrics
    }

    pub fn get_connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn log_health_check(&self) {
        info!("System Health Check:");
        info!("  Uptime: {} seconds", self.metrics.uptime_seconds);
        info!("  Active Positions: {}", self.metrics.active_positions);
        info!("  Cache Size: {}", self.metrics.cache_size);
        info!(
            "  WebSocket Connected: {}",
            self.connection_status.websocket_connected
        );
        info!(
            "  API Responsive: {}",
            self.connection_status.api_responsive
        );
        info!("  Total PnL: {:.2}", self.trading_metrics.total_pnl);
        info!("  Win Rate: {:.2}%", self.trading_metrics.win_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::PerformanceStats;

    // Helper function to create test PerformanceStats
    fn create_test_performance_stats() -> PerformanceStats {
        PerformanceStats {
            total_trades: 100,
            winning_trades: 60,
            losing_trades: 40,
            win_rate: 60.0,
            total_pnl: 1500.50,
            avg_pnl: 15.00,
            max_win: 200.0,
            max_loss: -100.0,
        }
    }

    // Test SystemMetrics structure
    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics {
            uptime_seconds: 3600,
            active_positions: 5,
            total_trades: 100,
            cache_size: 1024,
            memory_usage_mb: 128.5,
            cpu_usage_percent: 45.2,
            last_update: 1234567890,
        };

        assert_eq!(metrics.uptime_seconds, 3600);
        assert_eq!(metrics.active_positions, 5);
        assert_eq!(metrics.total_trades, 100);
        assert_eq!(metrics.cache_size, 1024);
        assert_eq!(metrics.memory_usage_mb, 128.5);
        assert_eq!(metrics.cpu_usage_percent, 45.2);
        assert_eq!(metrics.last_update, 1234567890);
    }

    #[test]
    fn test_system_metrics_serialization() {
        let metrics = SystemMetrics {
            uptime_seconds: 100,
            active_positions: 2,
            total_trades: 50,
            cache_size: 512,
            memory_usage_mb: 64.0,
            cpu_usage_percent: 25.0,
            last_update: 1000000,
        };

        // Test serialization
        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        assert!(json.contains("\"uptime_seconds\":100"));
        assert!(json.contains("\"active_positions\":2"));

        // Test deserialization
        let deserialized: SystemMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.uptime_seconds, metrics.uptime_seconds);
        assert_eq!(deserialized.active_positions, metrics.active_positions);
    }

    #[test]
    fn test_system_metrics_clone() {
        let metrics = SystemMetrics {
            uptime_seconds: 200,
            active_positions: 3,
            total_trades: 75,
            cache_size: 256,
            memory_usage_mb: 32.0,
            cpu_usage_percent: 15.0,
            last_update: 2000000,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.uptime_seconds, metrics.uptime_seconds);
        assert_eq!(cloned.active_positions, metrics.active_positions);
        assert_eq!(cloned.total_trades, metrics.total_trades);
    }

    // Test TradingMetrics structure
    #[test]
    fn test_trading_metrics_creation() {
        let metrics = TradingMetrics {
            total_pnl: 2500.75,
            win_rate: 65.5,
            avg_trade_duration_minutes: 120.5,
            max_drawdown: -500.25,
            sharpe_ratio: Some(1.75),
            total_volume: 100000.0,
        };

        assert_eq!(metrics.total_pnl, 2500.75);
        assert_eq!(metrics.win_rate, 65.5);
        assert_eq!(metrics.avg_trade_duration_minutes, 120.5);
        assert_eq!(metrics.max_drawdown, -500.25);
        assert_eq!(metrics.sharpe_ratio, Some(1.75));
        assert_eq!(metrics.total_volume, 100000.0);
    }

    #[test]
    fn test_trading_metrics_with_none_sharpe() {
        let metrics = TradingMetrics {
            total_pnl: 1000.0,
            win_rate: 50.0,
            avg_trade_duration_minutes: 60.0,
            max_drawdown: -200.0,
            sharpe_ratio: None,
            total_volume: 50000.0,
        };

        assert!(metrics.sharpe_ratio.is_none());
    }

    #[test]
    fn test_trading_metrics_serialization() {
        let metrics = TradingMetrics {
            total_pnl: 1500.0,
            win_rate: 55.0,
            avg_trade_duration_minutes: 90.0,
            max_drawdown: -300.0,
            sharpe_ratio: Some(1.5),
            total_volume: 75000.0,
        };

        // Test serialization
        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        assert!(json.contains("\"total_pnl\":1500"));
        assert!(json.contains("\"win_rate\":55"));

        // Test deserialization
        let deserialized: TradingMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.total_pnl, metrics.total_pnl);
        assert_eq!(deserialized.sharpe_ratio, metrics.sharpe_ratio);
    }

    // Test ConnectionStatus structure
    #[test]
    fn test_connection_status_creation() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: true,
            last_data_update: 1234567890,
            reconnect_count: 5,
        };

        assert!(status.websocket_connected);
        assert!(status.api_responsive);
        assert_eq!(status.last_data_update, 1234567890);
        assert_eq!(status.reconnect_count, 5);
    }

    #[test]
    fn test_connection_status_disconnected() {
        let status = ConnectionStatus {
            websocket_connected: false,
            api_responsive: false,
            last_data_update: 0,
            reconnect_count: 0,
        };

        assert!(!status.websocket_connected);
        assert!(!status.api_responsive);
        assert_eq!(status.reconnect_count, 0);
    }

    #[test]
    fn test_connection_status_serialization() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: false,
            last_data_update: 999999,
            reconnect_count: 3,
        };

        // Test serialization
        let json = serde_json::to_string(&status).expect("Failed to serialize");
        assert!(json.contains("\"websocket_connected\":true"));
        assert!(json.contains("\"reconnect_count\":3"));

        // Test deserialization
        let deserialized: ConnectionStatus =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.websocket_connected, status.websocket_connected);
        assert_eq!(deserialized.reconnect_count, status.reconnect_count);
    }

    // Test MonitoringService::new()
    #[test]
    fn test_monitoring_service_new() {
        let service = MonitoringService::new();

        // Verify initial system metrics
        assert_eq!(service.metrics.uptime_seconds, 0);
        assert_eq!(service.metrics.active_positions, 0);
        assert_eq!(service.metrics.total_trades, 0);
        assert_eq!(service.metrics.cache_size, 0);
        assert_eq!(service.metrics.memory_usage_mb, 0.0);
        assert_eq!(service.metrics.cpu_usage_percent, 0.0);
        assert!(service.metrics.last_update > 0);

        // Verify initial trading metrics
        assert_eq!(service.trading_metrics.total_pnl, 0.0);
        assert_eq!(service.trading_metrics.win_rate, 0.0);
        assert_eq!(service.trading_metrics.avg_trade_duration_minutes, 0.0);
        assert_eq!(service.trading_metrics.max_drawdown, 0.0);
        assert!(service.trading_metrics.sharpe_ratio.is_none());
        assert_eq!(service.trading_metrics.total_volume, 0.0);

        // Verify initial connection status
        assert!(!service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);
        assert_eq!(service.connection_status.last_data_update, 0);
        assert_eq!(service.connection_status.reconnect_count, 0);
    }

    #[test]
    fn test_monitoring_service_default() {
        let service = MonitoringService::default();

        // Should behave the same as new()
        assert_eq!(service.metrics.active_positions, 0);
        assert_eq!(service.trading_metrics.total_pnl, 0.0);
        assert!(!service.connection_status.websocket_connected);
    }

    // Test update_system_metrics
    #[test]
    fn test_update_system_metrics() {
        let mut service = MonitoringService::new();
        let initial_timestamp = service.metrics.last_update;

        // Wait at least 1 second to ensure uptime is measurable
        std::thread::sleep(std::time::Duration::from_secs(1));

        service.update_system_metrics(3, 512);

        // Verify updates
        assert!(service.metrics.uptime_seconds >= 1);
        assert_eq!(service.metrics.active_positions, 3);
        assert_eq!(service.metrics.cache_size, 512);
        assert!(service.metrics.last_update >= initial_timestamp);
        assert_eq!(service.metrics.memory_usage_mb, 50.0); // Placeholder value
        assert_eq!(service.metrics.cpu_usage_percent, 10.0); // Placeholder value
    }

    #[test]
    fn test_update_system_metrics_multiple_times() {
        let mut service = MonitoringService::new();

        service.update_system_metrics(1, 100);
        assert_eq!(service.metrics.active_positions, 1);
        assert_eq!(service.metrics.cache_size, 100);

        service.update_system_metrics(5, 500);
        assert_eq!(service.metrics.active_positions, 5);
        assert_eq!(service.metrics.cache_size, 500);

        service.update_system_metrics(0, 0);
        assert_eq!(service.metrics.active_positions, 0);
        assert_eq!(service.metrics.cache_size, 0);
    }

    #[test]
    fn test_update_system_metrics_uptime_increases() {
        let mut service = MonitoringService::new();

        service.update_system_metrics(1, 100);
        let uptime1 = service.metrics.uptime_seconds;

        std::thread::sleep(std::time::Duration::from_millis(100));

        service.update_system_metrics(2, 200);
        let uptime2 = service.metrics.uptime_seconds;

        assert!(uptime2 >= uptime1);
    }

    // Test update_trading_metrics
    #[test]
    fn test_update_trading_metrics() {
        let mut service = MonitoringService::new();
        let stats = create_test_performance_stats();

        service.update_trading_metrics(&stats);

        assert_eq!(service.trading_metrics.total_pnl, 1500.50);
        assert_eq!(service.trading_metrics.win_rate, 60.0);
    }

    #[test]
    fn test_update_trading_metrics_zero_values() {
        let mut service = MonitoringService::new();
        let stats = PerformanceStats {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            total_pnl: 0.0,
            avg_pnl: 0.0,
            max_win: 0.0,
            max_loss: 0.0,
        };

        service.update_trading_metrics(&stats);

        assert_eq!(service.trading_metrics.total_pnl, 0.0);
        assert_eq!(service.trading_metrics.win_rate, 0.0);
    }

    #[test]
    fn test_update_trading_metrics_negative_pnl() {
        let mut service = MonitoringService::new();
        let stats = PerformanceStats {
            total_trades: 50,
            winning_trades: 20,
            losing_trades: 30,
            win_rate: 40.0,
            total_pnl: -500.75,
            avg_pnl: -10.01,
            max_win: 100.0,
            max_loss: -200.0,
        };

        service.update_trading_metrics(&stats);

        assert_eq!(service.trading_metrics.total_pnl, -500.75);
        assert_eq!(service.trading_metrics.win_rate, 40.0);
    }

    // Test update_connection_status
    #[test]
    fn test_update_connection_status_connected() {
        let mut service = MonitoringService::new();
        let initial_timestamp = service.connection_status.last_data_update;

        service.update_connection_status(true, true);

        assert!(service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);
        assert!(service.connection_status.last_data_update > initial_timestamp);
    }

    #[test]
    fn test_update_connection_status_disconnected() {
        let mut service = MonitoringService::new();

        service.update_connection_status(false, false);

        assert!(!service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);
    }

    #[test]
    fn test_update_connection_status_partial() {
        let mut service = MonitoringService::new();

        service.update_connection_status(true, false);
        assert!(service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);

        service.update_connection_status(false, true);
        assert!(!service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);
    }

    #[test]
    fn test_update_connection_status_timestamp_updates() {
        let mut service = MonitoringService::new();

        service.update_connection_status(true, true);
        let timestamp1 = service.connection_status.last_data_update;

        std::thread::sleep(std::time::Duration::from_millis(10));

        service.update_connection_status(true, false);
        let timestamp2 = service.connection_status.last_data_update;

        assert!(timestamp2 >= timestamp1);
    }

    // Test record_reconnect
    #[test]
    fn test_record_reconnect() {
        let mut service = MonitoringService::new();

        assert_eq!(service.connection_status.reconnect_count, 0);

        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, 1);

        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, 2);

        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, 3);
    }

    #[test]
    fn test_record_reconnect_multiple_times() {
        let mut service = MonitoringService::new();

        for i in 1..=10 {
            service.record_reconnect();
            assert_eq!(service.connection_status.reconnect_count, i);
        }
    }

    // Test getter methods
    #[test]
    fn test_get_system_metrics() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(5, 1024);

        let metrics = service.get_system_metrics();
        assert_eq!(metrics.active_positions, 5);
        assert_eq!(metrics.cache_size, 1024);
    }

    #[test]
    fn test_get_trading_metrics() {
        let mut service = MonitoringService::new();
        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);

        let metrics = service.get_trading_metrics();
        assert_eq!(metrics.total_pnl, 1500.50);
        assert_eq!(metrics.win_rate, 60.0);
    }

    #[test]
    fn test_get_connection_status() {
        let mut service = MonitoringService::new();
        service.update_connection_status(true, false);

        let status = service.get_connection_status();
        assert!(status.websocket_connected);
        assert!(!status.api_responsive);
    }

    #[test]
    fn test_getters_return_references() {
        let service = MonitoringService::new();

        // Test that we can get multiple references without moving
        let _metrics1 = service.get_system_metrics();
        let _metrics2 = service.get_system_metrics();
        let _trading1 = service.get_trading_metrics();
        let _trading2 = service.get_trading_metrics();
        let _status1 = service.get_connection_status();
        let _status2 = service.get_connection_status();
    }

    // Test log_health_check (should not panic)
    #[test]
    fn test_log_health_check() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(3, 512);
        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);
        service.update_connection_status(true, true);

        // Should not panic
        service.log_health_check();
    }

    #[test]
    fn test_log_health_check_with_defaults() {
        let service = MonitoringService::new();

        // Should not panic with default values
        service.log_health_check();
    }

    // Integration tests
    #[test]
    fn test_full_monitoring_workflow() {
        let mut service = MonitoringService::new();

        // Simulate system startup
        assert_eq!(service.metrics.active_positions, 0);
        assert_eq!(service.connection_status.reconnect_count, 0);

        // Update system metrics
        service.update_system_metrics(2, 256);
        assert_eq!(service.metrics.active_positions, 2);

        // Update connection status
        service.update_connection_status(true, true);
        assert!(service.connection_status.websocket_connected);

        // Record some reconnects
        service.record_reconnect();
        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, 2);

        // Update trading metrics
        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);
        assert_eq!(service.trading_metrics.total_pnl, 1500.50);

        // Check health
        service.log_health_check();

        // Verify all data is consistent
        let sys_metrics = service.get_system_metrics();
        let trading_metrics = service.get_trading_metrics();
        let conn_status = service.get_connection_status();

        assert_eq!(sys_metrics.active_positions, 2);
        assert_eq!(trading_metrics.total_pnl, 1500.50);
        assert_eq!(conn_status.reconnect_count, 2);
    }

    #[test]
    fn test_monitoring_service_state_isolation() {
        let mut service1 = MonitoringService::new();
        let mut service2 = MonitoringService::new();

        service1.update_system_metrics(5, 500);
        service2.update_system_metrics(10, 1000);

        assert_eq!(service1.metrics.active_positions, 5);
        assert_eq!(service2.metrics.active_positions, 10);
        assert_eq!(service1.metrics.cache_size, 500);
        assert_eq!(service2.metrics.cache_size, 1000);
    }

    #[test]
    fn test_metrics_persistence_across_updates() {
        let mut service = MonitoringService::new();

        // Set initial trading metrics
        let stats1 = PerformanceStats {
            total_trades: 50,
            winning_trades: 30,
            losing_trades: 20,
            win_rate: 60.0,
            total_pnl: 1000.0,
            avg_pnl: 20.0,
            max_win: 150.0,
            max_loss: -50.0,
        };
        service.update_trading_metrics(&stats1);

        // Update system metrics - should not affect trading metrics
        service.update_system_metrics(5, 512);

        assert_eq!(service.trading_metrics.total_pnl, 1000.0);
        assert_eq!(service.trading_metrics.win_rate, 60.0);
        assert_eq!(service.metrics.active_positions, 5);
    }

    #[test]
    fn test_connection_status_independent_of_metrics() {
        let mut service = MonitoringService::new();

        service.update_connection_status(true, true);
        service.record_reconnect();

        // Update other metrics
        service.update_system_metrics(10, 2048);
        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);

        // Connection status should remain unchanged
        assert_eq!(service.connection_status.reconnect_count, 1);
        assert!(service.connection_status.websocket_connected);
    }

    // Edge cases
    #[test]
    fn test_system_metrics_with_large_values() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(usize::MAX, usize::MAX);

        assert_eq!(service.metrics.active_positions, usize::MAX);
        assert_eq!(service.metrics.cache_size, usize::MAX);
    }

    #[test]
    fn test_trading_metrics_with_extreme_values() {
        let mut service = MonitoringService::new();
        let stats = PerformanceStats {
            total_trades: u64::MAX,
            winning_trades: u64::MAX,
            losing_trades: 0,
            win_rate: 100.0,
            total_pnl: f64::MAX,
            avg_pnl: f64::MAX,
            max_win: f64::MAX,
            max_loss: f64::MIN,
        };

        service.update_trading_metrics(&stats);

        assert_eq!(service.trading_metrics.total_pnl, f64::MAX);
        assert_eq!(service.trading_metrics.win_rate, 100.0);
    }

    #[test]
    fn test_reconnect_count_overflow() {
        let mut service = MonitoringService::new();

        // Set reconnect count near max
        service.connection_status.reconnect_count = u32::MAX - 2;

        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, u32::MAX - 1);

        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, u32::MAX);

        // This will saturate at max value
        service.record_reconnect();
        assert_eq!(service.connection_status.reconnect_count, u32::MAX);
    }

    // Additional tests for improved coverage

    // Test Debug trait implementations
    #[test]
    fn test_system_metrics_debug() {
        let metrics = SystemMetrics {
            uptime_seconds: 100,
            active_positions: 5,
            total_trades: 50,
            cache_size: 1024,
            memory_usage_mb: 64.0,
            cpu_usage_percent: 25.0,
            last_update: 1234567890,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("SystemMetrics"));
        assert!(debug_str.contains("uptime_seconds"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_trading_metrics_debug() {
        let metrics = TradingMetrics {
            total_pnl: 1500.0,
            win_rate: 55.0,
            avg_trade_duration_minutes: 90.0,
            max_drawdown: -300.0,
            sharpe_ratio: Some(1.5),
            total_volume: 75000.0,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("TradingMetrics"));
        assert!(debug_str.contains("total_pnl"));
        assert!(debug_str.contains("1500"));
    }

    #[test]
    fn test_connection_status_debug() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: false,
            last_data_update: 999999,
            reconnect_count: 3,
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("ConnectionStatus"));
        assert!(debug_str.contains("websocket_connected"));
        assert!(debug_str.contains("true"));
    }

    // Test floating point edge cases
    #[test]
    fn test_system_metrics_with_zero_floats() {
        let metrics = SystemMetrics {
            uptime_seconds: 0,
            active_positions: 0,
            total_trades: 0,
            cache_size: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            last_update: 0,
        };

        assert_eq!(metrics.memory_usage_mb, 0.0);
        assert_eq!(metrics.cpu_usage_percent, 0.0);
    }

    #[test]
    fn test_system_metrics_with_negative_floats() {
        let metrics = SystemMetrics {
            uptime_seconds: 100,
            active_positions: 1,
            total_trades: 10,
            cache_size: 100,
            memory_usage_mb: -1.0, // Edge case: negative memory (shouldn't happen in practice)
            cpu_usage_percent: -5.0, // Edge case: negative CPU (shouldn't happen in practice)
            last_update: 1000,
        };

        assert_eq!(metrics.memory_usage_mb, -1.0);
        assert_eq!(metrics.cpu_usage_percent, -5.0);
    }

    #[test]
    fn test_trading_metrics_with_zero_volume() {
        let metrics = TradingMetrics {
            total_pnl: 100.0,
            win_rate: 50.0,
            avg_trade_duration_minutes: 30.0,
            max_drawdown: -50.0,
            sharpe_ratio: Some(0.0),
            total_volume: 0.0,
        };

        assert_eq!(metrics.total_volume, 0.0);
        assert_eq!(metrics.sharpe_ratio, Some(0.0));
    }

    #[test]
    fn test_trading_metrics_with_negative_sharpe_ratio() {
        let metrics = TradingMetrics {
            total_pnl: -500.0,
            win_rate: 30.0,
            avg_trade_duration_minutes: 45.0,
            max_drawdown: -1000.0,
            sharpe_ratio: Some(-1.5),
            total_volume: 50000.0,
        };

        assert_eq!(metrics.sharpe_ratio, Some(-1.5));
        assert!(metrics.sharpe_ratio.unwrap() < 0.0);
    }

    #[test]
    fn test_trading_metrics_with_infinity_values() {
        let metrics = TradingMetrics {
            total_pnl: f64::INFINITY,
            win_rate: 100.0,
            avg_trade_duration_minutes: f64::INFINITY,
            max_drawdown: f64::NEG_INFINITY,
            sharpe_ratio: Some(f64::INFINITY),
            total_volume: f64::INFINITY,
        };

        assert!(metrics.total_pnl.is_infinite());
        assert!(metrics.avg_trade_duration_minutes.is_infinite());
        assert!(metrics.max_drawdown.is_infinite() && metrics.max_drawdown < 0.0);
    }

    // Test serialization edge cases
    #[test]
    fn test_system_metrics_serialization_with_special_chars() {
        let metrics = SystemMetrics {
            uptime_seconds: 12345,
            active_positions: 7,
            total_trades: 99,
            cache_size: 2048,
            memory_usage_mb: 123.456789,
            cpu_usage_percent: 78.9,
            last_update: 9876543210,
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: SystemMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.memory_usage_mb, metrics.memory_usage_mb);
        assert_eq!(deserialized.cpu_usage_percent, metrics.cpu_usage_percent);
    }

    #[test]
    fn test_trading_metrics_serialization_with_none() {
        let metrics = TradingMetrics {
            total_pnl: 2500.0,
            win_rate: 75.0,
            avg_trade_duration_minutes: 120.0,
            max_drawdown: -800.0,
            sharpe_ratio: None,
            total_volume: 100000.0,
        };

        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        assert!(json.contains("\"sharpe_ratio\":null"));

        let deserialized: TradingMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert!(deserialized.sharpe_ratio.is_none());
    }

    #[test]
    fn test_connection_status_clone() {
        let status = ConnectionStatus {
            websocket_connected: true,
            api_responsive: true,
            last_data_update: 123456789,
            reconnect_count: 10,
        };

        let cloned = status.clone();
        assert_eq!(cloned.websocket_connected, status.websocket_connected);
        assert_eq!(cloned.api_responsive, status.api_responsive);
        assert_eq!(cloned.last_data_update, status.last_data_update);
        assert_eq!(cloned.reconnect_count, status.reconnect_count);
    }

    #[test]
    fn test_trading_metrics_clone() {
        let metrics = TradingMetrics {
            total_pnl: 1000.0,
            win_rate: 65.0,
            avg_trade_duration_minutes: 90.0,
            max_drawdown: -200.0,
            sharpe_ratio: Some(1.8),
            total_volume: 80000.0,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.total_pnl, metrics.total_pnl);
        assert_eq!(cloned.win_rate, metrics.win_rate);
        assert_eq!(cloned.sharpe_ratio, metrics.sharpe_ratio);
    }

    // Test update methods with edge cases
    #[test]
    fn test_update_system_metrics_with_zero_values() {
        let mut service = MonitoringService::new();
        service.update_system_metrics(0, 0);

        assert_eq!(service.metrics.active_positions, 0);
        assert_eq!(service.metrics.cache_size, 0);
        assert_eq!(service.metrics.memory_usage_mb, 50.0); // Placeholder
        assert_eq!(service.metrics.cpu_usage_percent, 10.0); // Placeholder
    }

    #[test]
    fn test_update_system_metrics_timestamp_consistency() {
        let mut service = MonitoringService::new();
        let before = chrono::Utc::now().timestamp();

        service.update_system_metrics(5, 1024);

        let after = chrono::Utc::now().timestamp();
        assert!(service.metrics.last_update >= before);
        assert!(service.metrics.last_update <= after);
    }

    #[test]
    fn test_update_trading_metrics_multiple_updates() {
        let mut service = MonitoringService::new();

        let stats1 = PerformanceStats {
            total_trades: 50,
            winning_trades: 30,
            losing_trades: 20,
            win_rate: 60.0,
            total_pnl: 1000.0,
            avg_pnl: 20.0,
            max_win: 150.0,
            max_loss: -50.0,
        };
        service.update_trading_metrics(&stats1);
        assert_eq!(service.trading_metrics.total_pnl, 1000.0);

        let stats2 = PerformanceStats {
            total_trades: 100,
            winning_trades: 70,
            losing_trades: 30,
            win_rate: 70.0,
            total_pnl: 2500.0,
            avg_pnl: 25.0,
            max_win: 200.0,
            max_loss: -75.0,
        };
        service.update_trading_metrics(&stats2);
        assert_eq!(service.trading_metrics.total_pnl, 2500.0);
        assert_eq!(service.trading_metrics.win_rate, 70.0);
    }

    #[test]
    fn test_update_trading_metrics_with_high_win_rate() {
        let mut service = MonitoringService::new();
        let stats = PerformanceStats {
            total_trades: 1000,
            winning_trades: 950,
            losing_trades: 50,
            win_rate: 95.0,
            total_pnl: 10000.0,
            avg_pnl: 10.0,
            max_win: 500.0,
            max_loss: -50.0,
        };

        service.update_trading_metrics(&stats);
        assert_eq!(service.trading_metrics.win_rate, 95.0);
        assert_eq!(service.trading_metrics.total_pnl, 10000.0);
    }

    #[test]
    fn test_update_trading_metrics_with_low_win_rate() {
        let mut service = MonitoringService::new();
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 10,
            losing_trades: 90,
            win_rate: 10.0,
            total_pnl: -5000.0,
            avg_pnl: -50.0,
            max_win: 100.0,
            max_loss: -500.0,
        };

        service.update_trading_metrics(&stats);
        assert_eq!(service.trading_metrics.win_rate, 10.0);
        assert_eq!(service.trading_metrics.total_pnl, -5000.0);
    }

    #[test]
    fn test_connection_status_toggle() {
        let mut service = MonitoringService::new();

        // Initially disconnected
        assert!(!service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);

        // Connect
        service.update_connection_status(true, true);
        assert!(service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);

        // Disconnect
        service.update_connection_status(false, false);
        assert!(!service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);

        // Reconnect
        service.update_connection_status(true, true);
        assert!(service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);
    }

    #[test]
    fn test_connection_status_websocket_only() {
        let mut service = MonitoringService::new();
        service.update_connection_status(true, false);

        assert!(service.connection_status.websocket_connected);
        assert!(!service.connection_status.api_responsive);
    }

    #[test]
    fn test_connection_status_api_only() {
        let mut service = MonitoringService::new();
        service.update_connection_status(false, true);

        assert!(!service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);
    }

    #[test]
    fn test_record_reconnect_does_not_affect_other_fields() {
        let mut service = MonitoringService::new();
        service.update_connection_status(true, true);

        let timestamp_before = service.connection_status.last_data_update;
        let ws_status_before = service.connection_status.websocket_connected;
        let api_status_before = service.connection_status.api_responsive;

        service.record_reconnect();

        // Only reconnect count should change
        assert_eq!(service.connection_status.reconnect_count, 1);
        assert_eq!(service.connection_status.last_data_update, timestamp_before);
        assert_eq!(
            service.connection_status.websocket_connected,
            ws_status_before
        );
        assert_eq!(service.connection_status.api_responsive, api_status_before);
    }

    #[test]
    fn test_record_reconnect_sequential() {
        let mut service = MonitoringService::new();

        for expected in 1..=100 {
            service.record_reconnect();
            assert_eq!(service.connection_status.reconnect_count, expected);
        }
    }

    // Test concurrent-like scenarios
    #[test]
    fn test_rapid_metric_updates() {
        let mut service = MonitoringService::new();

        for i in 0..100 {
            service.update_system_metrics(i, i * 10);
            assert_eq!(service.metrics.active_positions, i);
            assert_eq!(service.metrics.cache_size, i * 10);
        }
    }

    #[test]
    fn test_interleaved_updates() {
        let mut service = MonitoringService::new();

        service.update_system_metrics(1, 100);
        service.update_connection_status(true, false);
        service.record_reconnect();

        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);

        service.update_system_metrics(2, 200);
        service.update_connection_status(false, true);
        service.record_reconnect();

        assert_eq!(service.metrics.active_positions, 2);
        assert_eq!(service.metrics.cache_size, 200);
        assert!(!service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);
        assert_eq!(service.connection_status.reconnect_count, 2);
        assert_eq!(service.trading_metrics.total_pnl, 1500.50);
    }

    // Test getter immutability
    #[test]
    fn test_getters_do_not_mutate() {
        let service = MonitoringService::new();

        let metrics_ref1 = service.get_system_metrics();
        let uptime1 = metrics_ref1.uptime_seconds;

        let metrics_ref2 = service.get_system_metrics();
        let uptime2 = metrics_ref2.uptime_seconds;

        // Values should be identical since we didn't mutate
        assert_eq!(uptime1, uptime2);
    }

    // Test complete lifecycle
    #[test]
    fn test_monitoring_service_lifecycle() {
        let mut service = MonitoringService::new();

        // Phase 1: Startup
        assert_eq!(service.metrics.uptime_seconds, 0);
        assert!(!service.connection_status.websocket_connected);

        // Phase 2: Connection establishment
        service.update_connection_status(true, true);
        assert!(service.connection_status.websocket_connected);
        assert!(service.connection_status.api_responsive);

        // Phase 3: Trading activity
        service.update_system_metrics(5, 1024);
        let stats = create_test_performance_stats();
        service.update_trading_metrics(&stats);

        assert_eq!(service.metrics.active_positions, 5);
        assert_eq!(service.trading_metrics.total_pnl, 1500.50);

        // Phase 4: Connection issues
        service.update_connection_status(false, false);
        service.record_reconnect();
        assert!(!service.connection_status.websocket_connected);
        assert_eq!(service.connection_status.reconnect_count, 1);

        // Phase 5: Recovery
        service.update_connection_status(true, true);
        assert!(service.connection_status.websocket_connected);

        // Phase 6: Continued trading
        service.update_system_metrics(10, 2048);
        assert_eq!(service.metrics.active_positions, 10);

        // Verify complete state
        service.log_health_check();
    }

    // Test timestamp ordering
    #[test]
    fn test_timestamps_are_monotonic() {
        let mut service = MonitoringService::new();

        service.update_system_metrics(1, 100);
        let timestamp1 = service.metrics.last_update;

        std::thread::sleep(std::time::Duration::from_millis(10));

        service.update_system_metrics(2, 200);
        let timestamp2 = service.metrics.last_update;

        assert!(timestamp2 >= timestamp1);
    }

    #[test]
    fn test_connection_timestamps_are_monotonic() {
        let mut service = MonitoringService::new();

        service.update_connection_status(true, true);
        let timestamp1 = service.connection_status.last_data_update;

        std::thread::sleep(std::time::Duration::from_millis(10));

        service.update_connection_status(false, false);
        let timestamp2 = service.connection_status.last_data_update;

        assert!(timestamp2 >= timestamp1);
    }
}
