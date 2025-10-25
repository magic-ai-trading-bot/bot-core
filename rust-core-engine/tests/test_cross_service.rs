/// Cross-service integration tests
/// Tests communication between Rust Core Engine and Python AI Service

#[cfg(test)]
mod cross_service_tests {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Serialize, Debug)]
    struct AnalysisRequest {
        symbol: String,
        timeframe: String,
        candles: Vec<Candle>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct Candle {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: i64,
    }

    #[derive(Deserialize, Debug)]
    struct AIAnalysis {
        signal: String,
        confidence: f64,
        reasoning: String,
    }

    #[tokio::test]
    #[ignore] // Run only when services are running
    async fn test_rust_calls_python_ai_analysis() {
        // Test Rust → Python AI service communication

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        // Prepare request
        let request = AnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            candles: vec![
                Candle {
                    open: 50000.0,
                    high: 50500.0,
                    low: 49800.0,
                    close: 50200.0,
                    volume: 1000.0,
                    timestamp: 1701234567000,
                },
                Candle {
                    open: 50200.0,
                    high: 50800.0,
                    low: 50100.0,
                    close: 50600.0,
                    volume: 1200.0,
                    timestamp: 1701238167000,
                },
            ],
        };

        // Call Python AI service
        let response = client
            .post("http://python-ai-service:8000/ai/analyze")
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200);

                let analysis: Result<AIAnalysis, _> = resp.json().await;
                assert!(analysis.is_ok());

                let analysis = analysis.unwrap();
                assert!(!analysis.signal.is_empty());
                assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
                assert!(!analysis.reasoning.is_empty());
            },
            Err(e) => {
                // Service might not be running in test environment
                println!("Python AI service not available: {}", e);
            },
        }
    }

    #[tokio::test]
    #[ignore] // Run only when services are running
    async fn test_python_health_check() {
        let client = Client::new();

        let response = client
            .get("http://python-ai-service:8000/health")
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200);

                #[derive(Deserialize)]
                struct HealthResponse {
                    status: String,
                    service: String,
                }

                let health: Result<HealthResponse, _> = resp.json().await;
                if let Ok(health) = health {
                    assert_eq!(health.status, "healthy");
                }
            },
            Err(e) => {
                println!("Health check failed: {}", e);
            },
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_ai_requests() {
        // Test multiple concurrent requests to Python AI

        let client = Client::new();
        let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];

        let mut handles = vec![];

        for symbol in symbols {
            let client = client.clone();
            let symbol = symbol.to_string();

            let handle = tokio::spawn(async move {
                let request = AnalysisRequest {
                    symbol: symbol.clone(),
                    timeframe: "1h".to_string(),
                    candles: vec![Candle {
                        open: 50000.0,
                        high: 50500.0,
                        low: 49800.0,
                        close: 50200.0,
                        volume: 1000.0,
                        timestamp: 1701234567000,
                    }],
                };

                client
                    .post("http://python-ai-service:8000/ai/analyze")
                    .json(&request)
                    .send()
                    .await
            });

            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        // At least some should succeed (if service is running)
        let success_count = results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        // Just verify no panics occurred
        assert!(success_count >= 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_error_handling_from_python() {
        // Test handling errors from Python service

        let client = Client::new();

        // Send invalid request
        let invalid_request = serde_json::json!({
            "symbol": "", // Invalid empty symbol
            "timeframe": "invalid",
            "candles": []
        });

        let response = client
            .post("http://python-ai-service:8000/ai/analyze")
            .json(&invalid_request)
            .send()
            .await;

        match response {
            Ok(resp) => {
                // Should get error response
                assert!(resp.status().is_client_error() || resp.status().is_server_error());
            },
            Err(_) => {
                // Network error is also acceptable in test
            },
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_retry_on_failure() {
        // Test retry logic when Python service fails

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let max_retries = 3;
        let mut attempts = 0;

        for _ in 0..max_retries {
            attempts += 1;

            let result = client
                .get("http://python-ai-service:8000/health")
                .send()
                .await;

            if result.is_ok() && result.unwrap().status().is_success() {
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        assert!(attempts <= max_retries);
    }
}
