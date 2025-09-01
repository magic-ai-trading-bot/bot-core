mod common;

use binance_trading_bot::config::DatabaseConfig;
use binance_trading_bot::storage::Storage;

#[tokio::test]
async fn test_storage_creation_with_mock_config() {
    // Test with invalid MongoDB URL (will use in-memory)
    let config = DatabaseConfig {
        url: "invalid://localhost".to_string(),
        database_name: Some("test_db".to_string()),
        max_connections: 10,
        enable_logging: false,
    };
    
    let storage = Storage::new(&config).await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_storage_with_different_configs() {
    let config = DatabaseConfig {
        url: "mock://test".to_string(),
        database_name: Some("test_db".to_string()),
        max_connections: 5,
        enable_logging: true,
    };
    
    let storage = Storage::new(&config).await;
    assert!(storage.is_ok());
    
    // Test with different config
    let config2 = DatabaseConfig {
        url: "test://localhost".to_string(),
        database_name: None,
        max_connections: 20,
        enable_logging: false,
    };
    
    let storage2 = Storage::new(&config2).await;
    assert!(storage2.is_ok());
}

#[tokio::test]  
async fn test_multiple_storage_instances() {
    let config1 = DatabaseConfig {
        url: "mock://test1".to_string(),
        database_name: Some("test_db1".to_string()),
        max_connections: 10,
        enable_logging: false,
    };
    
    let config2 = DatabaseConfig {
        url: "mock://test2".to_string(),
        database_name: Some("test_db2".to_string()),
        max_connections: 15,
        enable_logging: true,
    };
    
    let storage1 = Storage::new(&config1).await;
    let storage2 = Storage::new(&config2).await;
    
    // Both should work independently
    assert!(storage1.is_ok());
    assert!(storage2.is_ok());
}