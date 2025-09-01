mod common;

use binance_trading_bot::config::DatabaseConfig;
use binance_trading_bot::storage::Storage;

#[tokio::test]
async fn test_storage_creation_with_mock_config() {
    // Test with invalid MongoDB URL (will use in-memory)
    let config = DatabaseConfig {
        url: "invalid://localhost".to_string(),
        database_name: Some("test_db".to_string()),
    };
    
    let storage = Storage::new(&config).await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_storage_operations() {
    let config = DatabaseConfig {
        url: "mock://test".to_string(),
        database_name: Some("test_db".to_string()),
    };
    
    let storage = Storage::new(&config).await.unwrap();
    
    // Test cleanup operation (should not panic)
    let result = storage.cleanup_old_data(30).await;
    assert!(result.is_ok());
}

#[tokio::test]  
async fn test_multiple_storage_instances() {
    let config1 = DatabaseConfig {
        url: "mock://test1".to_string(),
        database_name: Some("test_db1".to_string()),
    };
    
    let config2 = DatabaseConfig {
        url: "mock://test2".to_string(),
        database_name: Some("test_db2".to_string()),
    };
    
    let storage1 = Storage::new(&config1).await.unwrap();
    let storage2 = Storage::new(&config2).await.unwrap();
    
    // Both should work independently
    let result1 = storage1.cleanup_old_data(30).await;
    let result2 = storage2.cleanup_old_data(60).await;
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}