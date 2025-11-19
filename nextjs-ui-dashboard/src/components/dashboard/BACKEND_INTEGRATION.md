# Backend Integration Guide for PerSymbolSettings

## Overview

This guide explains how to implement the required backend API endpoints in the Rust core engine to support the `PerSymbolSettings` component.

## Required Endpoints

### 1. GET `/api/paper-trading/symbol-settings`

Load all symbol configurations.

**Rust Implementation:**

```rust
// src/api/paper_trading.rs

use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolConfig {
    pub symbol: String,
    pub enabled: bool,
    pub leverage: u8,
    pub position_size_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_positions: u8,
}

#[derive(Serialize)]
struct SymbolSettingsResponse {
    success: bool,
    data: Option<Vec<SymbolConfig>>,
    error: Option<String>,
    timestamp: String,
}

#[get("/api/paper-trading/symbol-settings")]
pub async fn get_symbol_settings(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let paper_trading = data.paper_trading.lock().await;

    match paper_trading.get_symbol_settings() {
        Ok(configs) => Ok(HttpResponse::Ok().json(SymbolSettingsResponse {
            success: true,
            data: Some(configs),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(SymbolSettingsResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
    }
}
```

**MongoDB Schema:**

```rust
// Store in MongoDB collection: symbol_settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSettingsDocument {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String, // For multi-user support
    pub symbol: String,
    pub enabled: bool,
    pub leverage: u8,
    pub position_size_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_positions: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

### 2. PUT `/api/paper-trading/symbol-settings`

Save all symbol configurations.

**Request Body:**

```json
{
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 2
    }
  ]
}
```

**Rust Implementation:**

```rust
use actix_web::{put, web, HttpResponse};

#[derive(Deserialize)]
struct UpdateSymbolSettingsRequest {
    symbols: Vec<SymbolConfig>,
}

#[derive(Serialize)]
struct UpdateResponse {
    success: bool,
    message: Option<String>,
    error: Option<String>,
    timestamp: String,
}

#[put("/api/paper-trading/symbol-settings")]
pub async fn update_symbol_settings(
    data: web::Data<AppState>,
    req: web::Json<UpdateSymbolSettingsRequest>,
) -> Result<HttpResponse, Error> {
    let mut paper_trading = data.paper_trading.lock().await;

    // Validate inputs
    for config in &req.symbols {
        if config.leverage < 1 || config.leverage > 20 {
            return Ok(HttpResponse::BadRequest().json(UpdateResponse {
                success: false,
                message: None,
                error: Some(format!("Invalid leverage for {}: must be 1-20", config.symbol)),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }));
        }

        if config.position_size_pct < 1.0 || config.position_size_pct > 10.0 {
            return Ok(HttpResponse::BadRequest().json(UpdateResponse {
                success: false,
                message: None,
                error: Some(format!("Invalid position size for {}: must be 1-10%", config.symbol)),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }));
        }

        if config.stop_loss_pct < 0.5 || config.stop_loss_pct > 5.0 {
            return Ok(HttpResponse::BadRequest().json(UpdateResponse {
                success: false,
                message: None,
                error: Some(format!("Invalid stop loss for {}: must be 0.5-5%", config.symbol)),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }));
        }

        if config.take_profit_pct < 1.0 || config.take_profit_pct > 10.0 {
            return Ok(HttpResponse::BadRequest().json(UpdateResponse {
                success: false,
                message: None,
                error: Some(format!("Invalid take profit for {}: must be 1-10%", config.symbol)),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }));
        }

        if config.max_positions < 1 || config.max_positions > 5 {
            return Ok(HttpResponse::BadRequest().json(UpdateResponse {
                success: false,
                message: None,
                error: Some(format!("Invalid max positions for {}: must be 1-5", config.symbol)),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }));
        }
    }

    // Save to database
    match paper_trading.update_symbol_settings(req.symbols.clone()).await {
        Ok(_) => Ok(HttpResponse::Ok().json(UpdateResponse {
            success: true,
            message: Some("Symbol settings updated successfully".to_string()),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(UpdateResponse {
            success: false,
            message: None,
            error: Some(e.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
    }
}
```

---

### 3. PUT `/api/paper-trading/symbol-settings/{symbol}`

Save individual symbol configuration.

**Rust Implementation:**

```rust
use actix_web::{put, web, HttpResponse};

#[put("/api/paper-trading/symbol-settings/{symbol}")]
pub async fn update_individual_symbol_setting(
    data: web::Data<AppState>,
    symbol: web::Path<String>,
    config: web::Json<SymbolConfig>,
) -> Result<HttpResponse, Error> {
    let mut paper_trading = data.paper_trading.lock().await;

    // Validate that symbol in path matches config
    if symbol.as_str() != config.symbol {
        return Ok(HttpResponse::BadRequest().json(UpdateResponse {
            success: false,
            message: None,
            error: Some("Symbol in path does not match config".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }));
    }

    // Validate config (same validations as batch update)
    // ... validation code ...

    match paper_trading.update_symbol_setting(config.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok().json(UpdateResponse {
            success: true,
            message: Some(format!("{} settings updated successfully", symbol)),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(UpdateResponse {
            success: false,
            message: None,
            error: Some(e.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })),
    }
}
```

---

## PaperTrading Service Implementation

Add these methods to your `PaperTradingService`:

```rust
// src/paper_trading/mod.rs

impl PaperTradingService {
    /// Get all symbol settings
    pub fn get_symbol_settings(&self) -> Result<Vec<SymbolConfig>, PaperTradingError> {
        // Load from MongoDB or in-memory cache
        let settings = self.symbol_settings.clone();
        Ok(settings)
    }

    /// Update all symbol settings
    pub async fn update_symbol_settings(
        &mut self,
        configs: Vec<SymbolConfig>,
    ) -> Result<(), PaperTradingError> {
        // Validate configs
        for config in &configs {
            self.validate_symbol_config(config)?;
        }

        // Save to MongoDB
        let collection = self.db.collection::<SymbolSettingsDocument>("symbol_settings");

        for config in configs {
            let filter = doc! { "symbol": &config.symbol };
            let update = doc! {
                "$set": {
                    "enabled": config.enabled,
                    "leverage": config.leverage as i32,
                    "position_size_pct": config.position_size_pct,
                    "stop_loss_pct": config.stop_loss_pct,
                    "take_profit_pct": config.take_profit_pct,
                    "max_positions": config.max_positions as i32,
                    "updated_at": chrono::Utc::now(),
                }
            };

            collection
                .update_one(filter, update, UpdateOptions::builder().upsert(true).build())
                .await?;

            // Update in-memory cache
            self.symbol_settings.retain(|s| s.symbol != config.symbol);
            self.symbol_settings.push(config);
        }

        Ok(())
    }

    /// Update single symbol setting
    pub async fn update_symbol_setting(
        &mut self,
        config: SymbolConfig,
    ) -> Result<(), PaperTradingError> {
        self.update_symbol_settings(vec![config]).await
    }

    /// Validate symbol configuration
    fn validate_symbol_config(&self, config: &SymbolConfig) -> Result<(), PaperTradingError> {
        if config.leverage < 1 || config.leverage > 20 {
            return Err(PaperTradingError::InvalidConfig(
                "Leverage must be between 1 and 20".to_string(),
            ));
        }

        if config.position_size_pct < 1.0 || config.position_size_pct > 10.0 {
            return Err(PaperTradingError::InvalidConfig(
                "Position size must be between 1% and 10%".to_string(),
            ));
        }

        if config.stop_loss_pct < 0.5 || config.stop_loss_pct > 5.0 {
            return Err(PaperTradingError::InvalidConfig(
                "Stop loss must be between 0.5% and 5%".to_string(),
            ));
        }

        if config.take_profit_pct < 1.0 || config.take_profit_pct > 10.0 {
            return Err(PaperTradingError::InvalidConfig(
                "Take profit must be between 1% and 10%".to_string(),
            ));
        }

        if config.max_positions < 1 || config.max_positions > 5 {
            return Err(PaperTradingError::InvalidConfig(
                "Max positions must be between 1 and 5".to_string(),
            ));
        }

        Ok(())
    }

    /// Get configuration for a specific symbol
    pub fn get_symbol_config(&self, symbol: &str) -> Option<&SymbolConfig> {
        self.symbol_settings.iter().find(|s| s.symbol == symbol)
    }

    /// Check if trading is enabled for a symbol
    pub fn is_symbol_enabled(&self, symbol: &str) -> bool {
        self.get_symbol_config(symbol)
            .map(|c| c.enabled)
            .unwrap_or(false)
    }
}
```

---

## Route Registration

Add the routes to your Actix-web configuration:

```rust
// src/api/mod.rs

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_symbol_settings)
        .service(update_symbol_settings)
        .service(update_individual_symbol_setting);
}
```

---

## CORS Configuration

Ensure CORS is properly configured to allow requests from the frontend:

```rust
// src/main.rs

use actix_cors::Cors;

HttpServer::new(move || {
    let cors = Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::ACCEPT,
            actix_web::http::header::CONTENT_TYPE,
        ])
        .max_age(3600);

    App::new()
        .wrap(cors)
        .configure(configure_routes)
})
```

---

## Testing the Integration

### 1. Test GET endpoint:

```bash
curl -X GET http://localhost:8080/api/paper-trading/symbol-settings
```

Expected response:
```json
{
  "success": true,
  "data": [
    {
      "symbol": "BTCUSDT",
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 2
    }
  ],
  "timestamp": "2025-11-19T10:30:00Z"
}
```

### 2. Test PUT endpoint (all symbols):

```bash
curl -X PUT http://localhost:8080/api/paper-trading/symbol-settings \
  -H "Content-Type: application/json" \
  -d '{
    "symbols": [
      {
        "symbol": "BTCUSDT",
        "enabled": true,
        "leverage": 10,
        "position_size_pct": 5.0,
        "stop_loss_pct": 2.0,
        "take_profit_pct": 4.0,
        "max_positions": 2
      }
    ]
  }'
```

### 3. Test PUT endpoint (individual symbol):

```bash
curl -X PUT http://localhost:8080/api/paper-trading/symbol-settings/BTCUSDT \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "enabled": true,
    "leverage": 10,
    "position_size_pct": 5.0,
    "stop_loss_pct": 2.0,
    "take_profit_pct": 4.0,
    "max_positions": 2
  }'
```

---

## Database Migrations

Create initial symbol settings in MongoDB:

```rust
// migrations/init_symbol_settings.rs

use mongodb::{Client, bson::doc};

async fn init_symbol_settings(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = client.database("bot_core");
    let collection = db.collection("symbol_settings");

    let default_symbols = vec![
        doc! {
            "symbol": "BTCUSDT",
            "enabled": true,
            "leverage": 10,
            "position_size_pct": 5.0,
            "stop_loss_pct": 2.0,
            "take_profit_pct": 4.0,
            "max_positions": 2,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        },
        doc! {
            "symbol": "ETHUSDT",
            "enabled": true,
            "leverage": 7,
            "position_size_pct": 4.0,
            "stop_loss_pct": 2.5,
            "take_profit_pct": 5.0,
            "max_positions": 2,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        },
        // ... other symbols
    ];

    collection.insert_many(default_symbols, None).await?;
    Ok(())
}
```

---

## Integration with Trading Logic

Use symbol settings when opening trades:

```rust
impl PaperTradingService {
    pub async fn open_trade(
        &mut self,
        symbol: &str,
        trade_type: TradeType,
        current_price: f64,
    ) -> Result<PaperTrade, PaperTradingError> {
        // Check if symbol is enabled
        if !self.is_symbol_enabled(symbol) {
            return Err(PaperTradingError::SymbolDisabled(symbol.to_string()));
        }

        // Get symbol-specific configuration
        let config = self.get_symbol_config(symbol)
            .ok_or(PaperTradingError::SymbolNotFound(symbol.to_string()))?;

        // Check max positions for this symbol
        let open_positions = self.get_open_positions_for_symbol(symbol).len();
        if open_positions >= config.max_positions as usize {
            return Err(PaperTradingError::MaxPositionsReached(symbol.to_string()));
        }

        // Calculate position size using symbol-specific settings
        let position_size = self.calculate_position_size(
            config.position_size_pct,
            config.leverage,
        )?;

        // Calculate stop loss and take profit using symbol-specific settings
        let stop_loss = self.calculate_stop_loss(
            current_price,
            config.stop_loss_pct,
            &trade_type,
        );

        let take_profit = self.calculate_take_profit(
            current_price,
            config.take_profit_pct,
            &trade_type,
        );

        // Create trade with symbol-specific parameters
        let trade = PaperTrade {
            symbol: symbol.to_string(),
            trade_type,
            entry_price: current_price,
            quantity: position_size,
            leverage: config.leverage,
            stop_loss: Some(stop_loss),
            take_profit: Some(take_profit),
            // ... other fields
        };

        // Save and return
        self.save_trade(&trade).await?;
        Ok(trade)
    }
}
```

---

## Summary

This integration enables:

1. ✅ Per-symbol configuration storage in MongoDB
2. ✅ RESTful API endpoints for CRUD operations
3. ✅ Input validation and error handling
4. ✅ Integration with trading logic
5. ✅ CORS support for frontend access

The frontend `PerSymbolSettings` component will seamlessly integrate with these endpoints to provide a complete per-symbol configuration experience.
