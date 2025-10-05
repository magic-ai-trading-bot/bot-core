use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[cfg(feature = "database")]
use bson::{doc, Bson, Document};
#[cfg(feature = "database")]
use futures::stream::StreamExt;
#[cfg(feature = "database")]
use mongodb::{Client, Collection, Database};

use crate::binance::types::Kline;
use crate::market_data::analyzer::MultiTimeframeAnalysis;
use crate::paper_trading::{AITradingSignal, PaperPortfolio, PaperTrade};

#[derive(Clone)]
pub struct Storage {
    #[cfg(feature = "database")]
    db: Option<Database>,

    // In-memory fallback storage
    #[cfg(not(feature = "database"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Storage {
    pub async fn new(config: &crate::config::DatabaseConfig) -> Result<Self> {
        #[cfg(feature = "database")]
        {
            if config.url.starts_with("mongodb://") || config.url.starts_with("mongodb+srv://") {
                let client = Client::with_uri_str(&config.url).await?;
                let db = client.database(
                    config
                        .database_name
                        .as_ref()
                        .unwrap_or(&"trading_bot".to_string()),
                );

                // Test connection by listing collections
                let _ = db.list_collection_names().await?;

                info!("MongoDB connected successfully to: {}", config.url);

                Ok(Self { db: Some(db) })
            } else {
                info!(
                    "Database URL not recognized as MongoDB connection string: {}",
                    config.url
                );
                Ok(Self { db: None })
            }
        }

        #[cfg(not(feature = "database"))]
        {
            info!("Database feature disabled, using in-memory storage");
            Ok(Self {
                _phantom: std::marker::PhantomData,
            })
        }
    }

    pub async fn store_analysis(&self, analysis: &MultiTimeframeAnalysis) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("analysis_results");

                let doc = doc! {
                    "symbol": &analysis.symbol,
                    "timestamp": analysis.timestamp,
                    "overall_signal": format!("{:?}", analysis.overall_signal),
                    "overall_confidence": analysis.overall_confidence,
                    "analysis_data": bson::to_bson(analysis)?
                };

                // Use upsert pattern
                let filter = doc! { "symbol": &analysis.symbol };
                let update = doc! { "$set": doc };

                collection.update_one(filter, update).upsert(true).await?;

                debug!(
                    "Stored analysis result for {} at {}",
                    analysis.symbol, analysis.timestamp
                );
                return Ok(());
            }
        }

        // Fallback: just log the analysis
        debug!(
            "Analysis for {}: {:?} (confidence: {:.2})",
            analysis.symbol, analysis.overall_signal, analysis.overall_confidence
        );
        Ok(())
    }

    pub async fn get_latest_analysis(
        &self,
        symbol: &str,
    ) -> Result<Option<MultiTimeframeAnalysis>> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("analysis_results");

                let filter = doc! { "symbol": symbol };
                let doc = collection.find_one(filter).await?;

                if let Some(doc) = doc {
                    if let Some(analysis_data) = doc.get("analysis_data") {
                        let analysis: MultiTimeframeAnalysis =
                            bson::from_bson(analysis_data.clone())?;
                        return Ok(Some(analysis));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn get_analysis_history(
        &self,
        symbol: &str,
        limit: Option<i64>,
    ) -> Result<Vec<MultiTimeframeAnalysis>> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("analysis_results");
                let limit = limit.unwrap_or(100);

                let filter = doc! { "symbol": symbol };

                let mut cursor = collection
                    .find(filter)
                    .limit(limit)
                    .sort(doc! { "timestamp": -1 })
                    .await?;

                let mut analyses = Vec::new();
                while let Some(doc_result) = cursor.next().await {
                    if let Ok(document) = doc_result {
                        if let Some(analysis_data) = document.get("analysis_data") {
                            if let Ok(analysis) =
                                bson::from_bson::<MultiTimeframeAnalysis>(analysis_data.clone())
                            {
                                analyses.push(analysis);
                            }
                        }
                    }
                }

                return Ok(analyses);
            }
        }

        Ok(Vec::new())
    }

    pub async fn store_trade_record(&self, trade: &TradeRecord) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<TradeRecord> = db.collection("trade_records");

                collection.insert_one(trade).await?;

                info!(
                    "Stored trade record for {} {} at {}",
                    trade.symbol, trade.side, trade.entry_price
                );
                return Ok(());
            }
        }

        info!(
            "Trade record: {} {} {} @ {} (PnL: {:?})",
            trade.symbol, trade.side, trade.quantity, trade.entry_price, trade.pnl
        );
        Ok(())
    }

    pub async fn get_trade_history(
        &self,
        symbol: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<TradeRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<TradeRecord> = db.collection("trade_records");
                let limit = limit.unwrap_or(100);

                let filter = if let Some(symbol) = symbol {
                    doc! { "symbol": symbol }
                } else {
                    doc! {}
                };

                let mut cursor = collection
                    .find(filter)
                    .limit(limit)
                    .sort(doc! { "entry_time": -1 })
                    .await?;

                let mut trades = Vec::new();
                while let Some(result) = cursor.next().await {
                    if let Ok(trade) = result {
                        trades.push(trade);
                    }
                }

                return Ok(trades);
            }
        }

        Ok(Vec::new())
    }

    pub async fn get_performance_stats(&self) -> Result<PerformanceStats> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("trade_records");

                let pipeline = vec![
                    doc! {
                        "$match": { "status": "closed" }
                    },
                    doc! {
                        "$group": {
                            "_id": Bson::Null,
                            "total_trades": { "$sum": 1 },
                            "winning_trades": { "$sum": { "$cond": [{ "$gt": ["$pnl", 0] }, 1, 0] } },
                            "losing_trades": { "$sum": { "$cond": [{ "$lt": ["$pnl", 0] }, 1, 0] } },
                            "total_pnl": { "$sum": "$pnl" },
                            "avg_pnl": { "$avg": "$pnl" },
                            "max_win": { "$max": "$pnl" },
                            "max_loss": { "$min": "$pnl" }
                        }
                    },
                ];

                let mut cursor = collection.aggregate(pipeline).await?;

                if let Some(Ok(doc)) = cursor.next().await {
                    let total_trades = doc.get_i32("total_trades").unwrap_or(0) as u64;
                    let winning_trades = doc.get_i32("winning_trades").unwrap_or(0) as u64;
                    let losing_trades = doc.get_i32("losing_trades").unwrap_or(0) as u64;

                    let win_rate = if total_trades > 0 {
                        (winning_trades as f64 / total_trades as f64) * 100.0
                    } else {
                        0.0
                    };

                    return Ok(PerformanceStats {
                        total_trades,
                        winning_trades,
                        losing_trades,
                        win_rate,
                        total_pnl: doc.get_f64("total_pnl").unwrap_or(0.0),
                        avg_pnl: doc.get_f64("avg_pnl").unwrap_or(0.0),
                        max_win: doc.get_f64("max_win").unwrap_or(0.0),
                        max_loss: doc.get_f64("max_loss").unwrap_or(0.0),
                    });
                }
            }
        }

        // Return default stats if database is not available
        Ok(PerformanceStats::default())
    }

    pub async fn store_market_data(
        &self,
        symbol: &str,
        timeframe: &str,
        klines: &[Kline],
    ) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("market_data");

                let mut docs = Vec::new();
                for kline in klines {
                    let doc = doc! {
                        "symbol": symbol,
                        "timeframe": timeframe,
                        "open_time": kline.open_time,
                        "close_time": kline.close_time,
                        "open_price": kline.open.parse::<f64>().unwrap_or(0.0),
                        "high_price": kline.high.parse::<f64>().unwrap_or(0.0),
                        "low_price": kline.low.parse::<f64>().unwrap_or(0.0),
                        "close_price": kline.close.parse::<f64>().unwrap_or(0.0),
                        "volume": kline.volume.parse::<f64>().unwrap_or(0.0),
                        "quote_volume": kline.quote_asset_volume.parse::<f64>().unwrap_or(0.0),
                        "trades_count": kline.number_of_trades
                    };
                    docs.push(doc);
                }

                if !docs.is_empty() {
                    collection.insert_many(docs).await?;
                    debug!(
                        "Stored {} market data entries for {} {}",
                        klines.len(),
                        symbol,
                        timeframe
                    );
                }

                return Ok(());
            }
        }

        Ok(())
    }

    pub async fn get_market_data(
        &self,
        symbol: &str,
        timeframe: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Kline>> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("market_data");
                let limit = limit.unwrap_or(500);

                let filter = doc! {
                    "symbol": symbol,
                    "timeframe": timeframe
                };
                let mut cursor = collection
                    .find(filter)
                    .limit(limit)
                    .sort(doc! { "open_time": -1 })
                    .await?;

                let mut klines = Vec::new();
                while let Some(result) = cursor.next().await {
                    if let Ok(doc) = result {
                        let kline = Kline {
                            open_time: doc.get_i64("open_time").unwrap_or(0),
                            close_time: doc.get_i64("close_time").unwrap_or(0),
                            open: doc.get_f64("open_price").unwrap_or(0.0).to_string(),
                            high: doc.get_f64("high_price").unwrap_or(0.0).to_string(),
                            low: doc.get_f64("low_price").unwrap_or(0.0).to_string(),
                            close: doc.get_f64("close_price").unwrap_or(0.0).to_string(),
                            volume: doc.get_f64("volume").unwrap_or(0.0).to_string(),
                            quote_asset_volume: doc
                                .get_f64("quote_volume")
                                .unwrap_or(0.0)
                                .to_string(),
                            number_of_trades: doc.get_i64("trades_count").unwrap_or(0),
                            taker_buy_base_asset_volume: "0".to_string(),
                            taker_buy_quote_asset_volume: "0".to_string(),
                            ignore: "0".to_string(),
                        };
                        klines.push(kline);
                    }
                }

                // Reverse to get chronological order
                klines.reverse();
                return Ok(klines);
            }
        }

        Ok(Vec::new())
    }

    pub async fn store_price_history(
        &self,
        symbol: &str,
        price: f64,
        volume_24h: f64,
        price_change_24h: f64,
        price_change_percent_24h: f64,
    ) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(db) = &self.db {
                let collection: Collection<Document> = db.collection("price_history");
                let timestamp = chrono::Utc::now().timestamp_millis();

                let doc = doc! {
                    "symbol": symbol,
                    "price": price,
                    "volume_24h": volume_24h,
                    "price_change_24h": price_change_24h,
                    "price_change_percent_24h": price_change_percent_24h,
                    "timestamp": timestamp
                };

                // Use upsert pattern
                let filter = doc! { "symbol": symbol };
                let update = doc! { "$set": doc };

                collection.update_one(filter, update).upsert(true).await?;

                debug!("Stored price history for {} at {}", symbol, price);
                return Ok(());
            }
        }

        Ok(())
    }

    // Get MongoDB database handle for auth service
    #[cfg(feature = "database")]
    pub fn get_database(&self) -> Option<&Database> {
        self.db.as_ref()
    }

    #[cfg(not(feature = "database"))]
    pub fn get_database(&self) -> Option<&()> {
        None
    }

    /// Get paper trading collection
    pub fn paper_trades(&self) -> Result<Collection<PaperTradingRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("paper_trades"))
    }

    /// Get portfolio history collection
    pub fn portfolio_history(&self) -> Result<Collection<PortfolioHistoryRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("portfolio_history"))
    }

    /// Get AI signals collection
    pub fn ai_signals(&self) -> Result<Collection<AISignalRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("ai_signals"))
    }

    /// Get performance metrics collection
    pub fn performance_metrics(&self) -> Result<Collection<PerformanceMetricsRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("performance_metrics"))
    }

    /// Get paper trading settings collection
    pub fn paper_trading_settings(&self) -> Result<Collection<PaperTradingSettingsRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("paper_trading_settings"))
    }

    /// Save paper trade to database
    pub async fn save_paper_trade(&self, trade: &PaperTrade) -> Result<()> {
        let record = PaperTradingRecord {
            id: None,
            trade_id: trade.id.clone(),
            symbol: trade.symbol.clone(),
            trade_type: format!("{:?}", trade.trade_type),
            status: format!("{:?}", trade.status),
            entry_price: trade.entry_price,
            exit_price: trade.exit_price,
            quantity: trade.quantity,
            leverage: trade.leverage,
            pnl: trade.realized_pnl,
            pnl_percentage: trade.pnl_percentage,
            trading_fees: trade.trading_fees,
            funding_fees: trade.funding_fees,
            open_time: trade.open_time,
            close_time: trade.close_time,
            ai_signal_id: trade.ai_signal_id.clone(),
            ai_confidence: trade.ai_confidence,
            close_reason: trade.close_reason.as_ref().map(|r| format!("{r:?}")),
            created_at: Utc::now(),
        };

        self.paper_trades()?.insert_one(record).await?;
        Ok(())
    }

    /// Update paper trade in database
    pub async fn update_paper_trade(&self, trade: &PaperTrade) -> Result<()> {
        let filter = doc! { "trade_id": &trade.id };
        let update = doc! {
            "$set": {
                "status": format!("{:?}", trade.status),
                "exit_price": trade.exit_price,
                "pnl": trade.realized_pnl,
                "pnl_percentage": trade.pnl_percentage,
                "funding_fees": trade.funding_fees,
                "close_time": trade.close_time,
                "close_reason": trade.close_reason.as_ref().map(|r| format!("{r:?}")),
            }
        };

        self.paper_trades()?.update_one(filter, update).await?;
        Ok(())
    }

    /// Save portfolio snapshot to history
    pub async fn save_portfolio_snapshot(&self, portfolio: &PaperPortfolio) -> Result<()> {
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp: Utc::now(),
            current_balance: portfolio.cash_balance,
            equity: portfolio.equity,
            margin_used: portfolio.margin_used,
            free_margin: portfolio.free_margin,
            total_pnl: portfolio.metrics.total_pnl,
            total_pnl_percentage: portfolio.metrics.total_pnl_percentage,
            total_trades: portfolio.metrics.total_trades as u32,
            win_rate: portfolio.metrics.win_rate,
            profit_factor: portfolio.metrics.profit_factor,
            max_drawdown: portfolio.metrics.max_drawdown,
            max_drawdown_percentage: portfolio.metrics.max_drawdown_percentage,
            open_positions: portfolio.open_trade_ids.len() as u32,
            created_at: Utc::now(),
        };

        self.portfolio_history()?.insert_one(record).await?;
        Ok(())
    }

    /// Save AI signal to database
    pub async fn save_ai_signal(
        &self,
        signal: &AITradingSignal,
        executed: bool,
        trade_id: Option<String>,
    ) -> Result<()> {
        let record = AISignalRecord {
            id: None,
            signal_id: signal.id.clone(),
            symbol: signal.symbol.clone(),
            signal_type: format!("{:?}", signal.signal_type),
            confidence: signal.confidence,
            reasoning: signal.reasoning.clone(),
            entry_price: signal.entry_price,
            trend_direction: signal.market_analysis.trend_direction.clone(),
            trend_strength: signal.market_analysis.trend_strength,
            volatility: signal.market_analysis.volatility,
            risk_score: signal.market_analysis.risk_score,
            executed,
            trade_id,
            created_at: Utc::now(),
            timestamp: signal.timestamp,
        };

        self.ai_signals()?.insert_one(record).await?;
        Ok(())
    }

    /// Save daily performance metrics
    pub async fn save_daily_metrics(
        &self,
        portfolio: &PaperPortfolio,
        daily_pnl: f64,
    ) -> Result<()> {
        let record = PerformanceMetricsRecord {
            id: None,
            date: Utc::now(),
            total_trades: portfolio.metrics.total_trades as u32,
            winning_trades: (portfolio.metrics.win_rate * portfolio.metrics.total_trades as f64
                / 100.0) as u32,
            losing_trades: portfolio.metrics.total_trades as u32
                - (portfolio.metrics.win_rate * portfolio.metrics.total_trades as f64 / 100.0)
                    as u32,
            win_rate: portfolio.metrics.win_rate,
            average_win: portfolio.metrics.average_win,
            average_loss: portfolio.metrics.average_loss,
            largest_win: portfolio.metrics.largest_win,
            largest_loss: portfolio.metrics.largest_loss,
            profit_factor: portfolio.metrics.profit_factor,
            sharpe_ratio: portfolio.metrics.sharpe_ratio,
            max_drawdown: portfolio.metrics.max_drawdown,
            max_drawdown_percentage: portfolio.metrics.max_drawdown_percentage,
            total_pnl: portfolio.metrics.total_pnl,
            daily_pnl,
            created_at: Utc::now(),
        };

        self.performance_metrics()?.insert_one(record).await?;
        Ok(())
    }

    /// Get trade history from database
    pub async fn get_paper_trades_history(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<PaperTradingRecord>> {
        let cursor = self
            .paper_trades()?
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(limit.unwrap_or(1000))
            .await?;
        let trades: Vec<PaperTradingRecord> = cursor.try_collect().await?;
        Ok(trades)
    }

    /// Get portfolio history from database
    pub async fn get_portfolio_history(
        &self,
        days: Option<i64>,
    ) -> Result<Vec<PortfolioHistoryRecord>> {
        let filter = if let Some(days) = days {
            let start_date = Utc::now() - chrono::Duration::days(days);
            doc! { "timestamp": { "$gte": start_date } }
        } else {
            doc! {}
        };

        let cursor = self
            .portfolio_history()?
            .find(filter)
            .sort(doc! { "timestamp": 1 })
            .await?;
        let history: Vec<PortfolioHistoryRecord> = cursor.try_collect().await?;
        Ok(history)
    }

    /// Get AI signals history from database
    pub async fn get_ai_signals_history(
        &self,
        symbol: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<AISignalRecord>> {
        let filter = if let Some(symbol) = symbol {
            doc! { "symbol": symbol }
        } else {
            doc! {}
        };

        let cursor = self
            .ai_signals()?
            .find(filter)
            .sort(doc! { "timestamp": -1 })
            .limit(limit.unwrap_or(1000))
            .await?;
        let signals: Vec<AISignalRecord> = cursor.try_collect().await?;
        Ok(signals)
    }

    /// Save paper trading settings to database
    pub async fn save_paper_trading_settings(
        &self,
        settings: &crate::paper_trading::PaperTradingSettings,
    ) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                // Convert settings to BSON document
                let settings_bson = bson::to_bson(settings)?;
                let settings_doc = settings_bson
                    .as_document()
                    .ok_or_else(|| anyhow::anyhow!("Failed to convert settings to document"))?
                    .clone();

                let record = PaperTradingSettingsRecord {
                    id: None,
                    settings_data: settings_doc,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Use upsert to update existing or create new
                let filter = doc! {}; // Only one settings record
                let update = doc! {
                    "$set": {
                        "settings_data": &record.settings_data,
                        "updated_at": &record.updated_at
                    },
                    "$setOnInsert": {
                        "created_at": &record.created_at
                    }
                };
                self.paper_trading_settings()?
                    .update_one(filter, update)
                    .upsert(true)
                    .await?;

                info!("💾 Paper trading settings saved to database");
                return Ok(());
            }
        }

        info!("💾 Paper trading settings saved (in-memory fallback)");
        Ok(())
    }

    /// Load paper trading settings from database
    pub async fn load_paper_trading_settings(
        &self,
    ) -> Result<Option<crate::paper_trading::PaperTradingSettings>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let record = self.paper_trading_settings()?.find_one(doc! {}).await?;

                if let Some(record) = record {
                    // Convert BSON document back to settings
                    let settings_bson = bson::Bson::Document(record.settings_data);
                    let settings = bson::from_bson::<crate::paper_trading::PaperTradingSettings>(
                        settings_bson,
                    )?;

                    info!(
                        "📖 Paper trading settings loaded from database (updated: {})",
                        record.updated_at
                    );
                    return Ok(Some(settings));
                }
            }
        }

        info!("📖 No saved paper trading settings found, will use defaults");
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub symbol: String,
    pub side: String, // "BUY" or "SELL"
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub entry_time: i64,
    pub exit_time: Option<i64>,
    pub pnl: Option<f64>,
    pub status: String, // "open", "closed", "cancelled"
    pub strategy_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub max_win: f64,
    pub max_loss: f64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            total_pnl: 0.0,
            avg_pnl: 0.0,
            max_win: 0.0,
            max_loss: 0.0,
        }
    }
}

/// Paper trading data models for MongoDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub trade_id: String,
    pub symbol: String,
    pub trade_type: String,
    pub status: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub leverage: u8,
    pub pnl: Option<f64>,
    pub pnl_percentage: f64,
    pub trading_fees: f64,
    pub funding_fees: f64,
    pub open_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub ai_signal_id: Option<String>,
    pub ai_confidence: Option<f64>,
    pub close_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHistoryRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub timestamp: DateTime<Utc>,
    pub current_balance: f64,
    pub equity: f64,
    pub margin_used: f64,
    pub free_margin: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub total_trades: u32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub open_positions: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISignalRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub signal_id: String,
    pub symbol: String,
    pub signal_type: String,
    pub confidence: f64,
    pub reasoning: String,
    pub entry_price: f64,
    pub trend_direction: String,
    pub trend_strength: f64,
    pub volatility: f64,
    pub risk_score: f64,
    pub executed: bool,
    pub trade_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub date: DateTime<Utc>,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub created_at: DateTime<Utc>,
}

/// Paper trading settings record for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingSettingsRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub settings_data: mongodb::bson::Document,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
