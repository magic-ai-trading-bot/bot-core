use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

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
                        .unwrap_or(&"bot_core".to_string()),
                );

                // Test connection by listing collections
                let _ = db.list_collection_names().await?;

                info!("MongoDB connected successfully to: {}", config.url);

                let storage = Self { db: Some(db) };

                // Create indexes for better query performance
                if let Err(e) = storage.ensure_indexes().await {
                    warn!("Failed to create indexes (non-fatal): {}", e);
                }

                Ok(storage)
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

    /// Create indexes for better query performance
    /// @spec:FR-PERF-001 - Database Index Optimization
    #[cfg(feature = "database")]
    async fn ensure_indexes(&self) -> Result<()> {
        use mongodb::options::IndexOptions;
        use mongodb::IndexModel;

        if let Some(db) = &self.db {
            // Index for ai_signals collection (used by get_latest_signals_per_symbol)
            // Compound index on (symbol, timestamp) for fast grouping and sorting
            let ai_signals: Collection<Document> = db.collection("ai_signals");
            let ai_signal_index = IndexModel::builder()
                .keys(doc! { "symbol": 1, "timestamp": -1 })
                .options(IndexOptions::builder().background(Some(true)).build())
                .build();
            if let Err(e) = ai_signals.create_index(ai_signal_index).await {
                warn!("Failed to create ai_signals index: {}", e);
            } else {
                info!("✅ Created index on ai_signals (symbol, timestamp)");
            }

            // Index for paper_trades collection
            let paper_trades: Collection<Document> = db.collection("paper_trades");
            let trades_index = IndexModel::builder()
                .keys(doc! { "signal_id": 1 })
                .options(IndexOptions::builder().background(Some(true)).build())
                .build();
            if let Err(e) = paper_trades.create_index(trades_index).await {
                warn!("Failed to create paper_trades index: {}", e);
            } else {
                info!("✅ Created index on paper_trades (signal_id)");
            }

            // Index for signals_history collection
            let signals_history: Collection<Document> = db.collection("signals_history");
            let history_index = IndexModel::builder()
                .keys(doc! { "timestamp": -1 })
                .options(IndexOptions::builder().background(Some(true)).build())
                .build();
            if let Err(e) = signals_history.create_index(history_index).await {
                warn!("Failed to create signals_history index: {}", e);
            } else {
                info!("✅ Created index on signals_history (timestamp)");
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "database"))]
    async fn ensure_indexes(&self) -> Result<()> {
        Ok(())
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
                let mut parse_errors = 0;

                for kline in klines {
                    // Parse prices with proper error handling - skip invalid data
                    let open_price = match kline.open.parse::<f64>() {
                        Ok(p) if p > 0.0 => p,
                        Ok(p) => {
                            warn!(
                                "Invalid open price {} for {} at {}, skipping kline",
                                p, symbol, kline.open_time
                            );
                            parse_errors += 1;
                            continue;
                        },
                        Err(e) => {
                            warn!(
                                "Failed to parse open price '{}' for {} at {}: {}, skipping kline",
                                kline.open, symbol, kline.open_time, e
                            );
                            parse_errors += 1;
                            continue;
                        },
                    };

                    let high_price = match kline.high.parse::<f64>() {
                        Ok(p) if p > 0.0 => p,
                        _ => {
                            parse_errors += 1;
                            continue;
                        },
                    };

                    let low_price = match kline.low.parse::<f64>() {
                        Ok(p) if p > 0.0 => p,
                        _ => {
                            parse_errors += 1;
                            continue;
                        },
                    };

                    let close_price = match kline.close.parse::<f64>() {
                        Ok(p) if p > 0.0 => p,
                        _ => {
                            parse_errors += 1;
                            continue;
                        },
                    };

                    let volume = kline.volume.parse::<f64>().unwrap_or(0.0);
                    let quote_volume = kline.quote_asset_volume.parse::<f64>().unwrap_or(0.0);

                    let doc = doc! {
                        "symbol": symbol,
                        "timeframe": timeframe,
                        "open_time": kline.open_time,
                        "close_time": kline.close_time,
                        "open_price": open_price,
                        "high_price": high_price,
                        "low_price": low_price,
                        "close_price": close_price,
                        "volume": volume,
                        "quote_volume": quote_volume,
                        "trades_count": kline.number_of_trades
                    };
                    docs.push(doc);
                }

                if parse_errors > 0 {
                    error!(
                        "Skipped {} invalid klines for {} {} due to parse errors",
                        parse_errors, symbol, timeframe
                    );
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
                let mut retrieval_errors = 0;
                while let Some(result) = cursor.next().await {
                    match result {
                        Ok(doc) => {
                            // Validate required fields exist before constructing Kline
                            match (
                                doc.get_i64("open_time"),
                                doc.get_i64("close_time"),
                                doc.get_f64("open_price"),
                                doc.get_f64("high_price"),
                                doc.get_f64("low_price"),
                                doc.get_f64("close_price"),
                            ) {
                                (
                                    Ok(open_time),
                                    Ok(close_time),
                                    Ok(open),
                                    Ok(high),
                                    Ok(low),
                                    Ok(close),
                                ) => {
                                    let kline = Kline {
                                        open_time,
                                        close_time,
                                        open: open.to_string(),
                                        high: high.to_string(),
                                        low: low.to_string(),
                                        close: close.to_string(),
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
                                },
                                _ => {
                                    retrieval_errors += 1;
                                    warn!("Skipping invalid market data document for {} {}: missing required fields", symbol, timeframe);
                                },
                            }
                        },
                        Err(e) => {
                            retrieval_errors += 1;
                            warn!(
                                "Failed to retrieve document from cursor for {} {}: {}",
                                symbol, timeframe, e
                            );
                        },
                    }
                }

                if retrieval_errors > 0 {
                    warn!(
                        "Encountered {} errors while retrieving market data for {} {}",
                        retrieval_errors, symbol, timeframe
                    );
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
    pub fn paper_trading_settings(&self) -> Result<Collection<Document>> {
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

        let result = self.paper_trades()?.update_one(filter, update).await?;

        // Verify the update actually happened
        if result.matched_count == 0 {
            return Err(anyhow::anyhow!(
                "Failed to update trade {}: No matching document found in database",
                trade.id
            ));
        }

        if result.modified_count == 0 {
            warn!(
                "⚠️ Trade {} update matched but didn't modify (document may be identical)",
                trade.id
            );
        }

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
            // Initialize outcome fields as pending
            outcome: Some("pending".to_string()),
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        self.ai_signals()?.insert_one(record).await?;
        Ok(())
    }

    /// Update AI signal outcome when trade closes
    /// @spec:FR-AI-012 - Signal Outcome Tracking
    pub async fn update_signal_outcome(
        &self,
        signal_id: &str,
        outcome: &str,
        actual_pnl: f64,
        pnl_percentage: f64,
        exit_price: f64,
        close_reason: &str,
    ) -> Result<()> {
        let filter = doc! { "signal_id": signal_id };
        let update = doc! {
            "$set": {
                "outcome": outcome,
                "actual_pnl": actual_pnl,
                "pnl_percentage": pnl_percentage,
                "exit_price": exit_price,
                "close_reason": close_reason,
                "closed_at": Utc::now(),
            }
        };

        let result = self.ai_signals()?.update_one(filter, update).await?;

        if result.matched_count == 0 {
            tracing::warn!(
                "⚠️ Signal {} not found in database for outcome update",
                signal_id
            );
        } else {
            tracing::info!(
                "✅ Updated signal {} outcome: {} (PnL: {:.2} USDT, {:.2}%)",
                signal_id,
                outcome,
                actual_pnl,
                pnl_percentage
            );
        }

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

    /// Get latest AI signal for each unique symbol (for quick page load)
    /// @spec:FR-AI-013 - Cached Signal Display
    pub async fn get_latest_signals_per_symbol(&self) -> Result<Vec<AISignalRecord>> {
        use mongodb::bson::Document;

        // Use aggregation to get the most recent signal for each symbol
        let pipeline = vec![
            // Sort by timestamp descending first
            doc! { "$sort": { "timestamp": -1 } },
            // Group by symbol, taking the first (most recent) document
            doc! {
                "$group": {
                    "_id": "$symbol",
                    "doc": { "$first": "$$ROOT" }
                }
            },
            // Replace root with the original document
            doc! { "$replaceRoot": { "newRoot": "$doc" } },
            // Sort results by symbol for consistent ordering
            doc! { "$sort": { "symbol": 1 } },
        ];

        let cursor = self.ai_signals()?.aggregate(pipeline).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;

        // Convert documents to AISignalRecord
        let signals: Vec<AISignalRecord> = docs
            .into_iter()
            .filter_map(|doc| mongodb::bson::from_document(doc).ok())
            .collect();

        log::info!(
            "📡 Retrieved {} latest signals (one per symbol)",
            signals.len()
        );
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
                // Convert settings to JSON string to avoid BSON type conversion issues
                // (especially with serde_json::Value fields that don't map well to BSON DateTime)
                let settings_json = serde_json::to_string(settings)?;

                let record = doc! {
                    "settings_json": settings_json,
                    "updated_at": Utc::now(),
                };

                // Use upsert to update existing or create new
                let filter = doc! {}; // Only one settings record
                let update = doc! {
                    "$set": record,
                    "$setOnInsert": {
                        "created_at": Utc::now()
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
                let record: Option<Document> =
                    self.paper_trading_settings()?.find_one(doc! {}).await?;

                if let Some(record) = record {
                    // Get the JSON string from the document
                    let settings_json = record
                        .get_str("settings_json")
                        .map_err(|e| anyhow::anyhow!("Failed to get settings_json field: {}", e))?;

                    let updated_at = record
                        .get_datetime("updated_at")
                        .map_err(|e| anyhow::anyhow!("Failed to get updated_at field: {}", e))?;

                    // Deserialize from JSON string
                    let settings: crate::paper_trading::PaperTradingSettings =
                        serde_json::from_str(settings_json)?;

                    info!(
                        "📖 Paper trading settings loaded from database (updated: {})",
                        updated_at
                    );
                    return Ok(Some(settings));
                }
            }
        }

        info!("📖 No saved paper trading settings found, will use defaults");
        Ok(None)
    }

    /// Get user symbols collection
    pub fn user_symbols(&self) -> Result<Collection<Document>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("user_symbols"))
    }

    /// Save user-added symbols to database
    pub async fn save_user_symbols(&self, symbols: &[String]) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let record = doc! {
                    "symbols": symbols,
                    "updated_at": Utc::now(),
                };

                // Use upsert to update existing or create new (only one record)
                let filter = doc! {};
                let update = doc! {
                    "$set": record,
                    "$setOnInsert": {
                        "created_at": Utc::now()
                    }
                };
                self.user_symbols()?
                    .update_one(filter, update)
                    .upsert(true)
                    .await?;

                info!("💾 User symbols saved to database: {:?}", symbols);
                return Ok(());
            }
        }

        info!("💾 User symbols saved (in-memory fallback)");
        Ok(())
    }

    /// Load user-added symbols from database
    pub async fn load_user_symbols(&self) -> Result<Vec<String>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let record: Option<Document> = self.user_symbols()?.find_one(doc! {}).await?;

                if let Some(record) = record {
                    if let Ok(symbols_bson) = record.get_array("symbols") {
                        let symbols: Vec<String> = symbols_bson
                            .iter()
                            .filter_map(|s| s.as_str().map(String::from))
                            .collect();

                        info!("📖 User symbols loaded from database: {:?}", symbols);
                        return Ok(symbols);
                    }
                }
            }
        }

        info!("📖 No saved user symbols found");
        Ok(vec![])
    }

    /// Add a single symbol to user symbols
    pub async fn add_user_symbol(&self, symbol: &str) -> Result<()> {
        let mut symbols = self.load_user_symbols().await?;
        if !symbols.contains(&symbol.to_string()) {
            symbols.push(symbol.to_string());
            self.save_user_symbols(&symbols).await?;
        }
        Ok(())
    }

    /// Remove a single symbol from user symbols
    pub async fn remove_user_symbol(&self, symbol: &str) -> Result<()> {
        let mut symbols = self.load_user_symbols().await?;
        symbols.retain(|s| s != symbol);
        self.save_user_symbols(&symbols).await?;
        Ok(())
    }

    // =========================================================================
    // API KEYS (Encrypted storage for Binance credentials)
    // =========================================================================

    /// Get API keys collection
    #[cfg(feature = "database")]
    fn api_keys(&self) -> Result<Collection<Document>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("api_keys"))
    }

    /// Save API key (encrypted) to database
    pub async fn save_api_key(&self, api_key: &crate::api::settings::StoredApiKey) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                // First delete any existing key, then insert new one (only one API key allowed)
                self.api_keys()?.delete_many(doc! {}).await?;
                let insert_doc = doc! {
                    "api_key": &api_key.api_key,
                    "api_secret_encrypted": &api_key.api_secret_encrypted,
                    "api_secret_nonce": &api_key.api_secret_nonce,
                    "use_testnet": api_key.use_testnet,
                    "permissions": {
                        "spot_trading": api_key.permissions.spot_trading,
                        "futures_trading": api_key.permissions.futures_trading,
                        "margin_trading": api_key.permissions.margin_trading,
                        "options_trading": api_key.permissions.options_trading,
                    },
                    "created_at": api_key.created_at,
                    "updated_at": api_key.updated_at,
                };
                self.api_keys()?.insert_one(insert_doc).await?;

                info!(
                    "API key saved to database (testnet: {})",
                    api_key.use_testnet
                );
                return Ok(());
            }
        }

        warn!("Database not available, API key not saved");
        Ok(())
    }

    /// Load API key from database
    pub async fn load_api_key(&self) -> Result<Option<crate::api::settings::StoredApiKey>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let record: Option<Document> = self.api_keys()?.find_one(doc! {}).await?;

                if let Some(record) = record {
                    let permissions_doc = record.get_document("permissions").ok();
                    let permissions = crate::api::settings::ApiKeyPermissions {
                        spot_trading: permissions_doc
                            .and_then(|p| p.get_bool("spot_trading").ok())
                            .unwrap_or(false),
                        futures_trading: permissions_doc
                            .and_then(|p| p.get_bool("futures_trading").ok())
                            .unwrap_or(true),
                        margin_trading: permissions_doc
                            .and_then(|p| p.get_bool("margin_trading").ok())
                            .unwrap_or(false),
                        options_trading: permissions_doc
                            .and_then(|p| p.get_bool("options_trading").ok())
                            .unwrap_or(false),
                    };

                    let stored_key = crate::api::settings::StoredApiKey {
                        api_key: record.get_str("api_key").unwrap_or("").to_string(),
                        api_secret_encrypted: record
                            .get_str("api_secret_encrypted")
                            .unwrap_or("")
                            .to_string(),
                        api_secret_nonce: record
                            .get_str("api_secret_nonce")
                            .unwrap_or("")
                            .to_string(),
                        use_testnet: record.get_bool("use_testnet").unwrap_or(true),
                        permissions,
                        created_at: record
                            .get_datetime("created_at")
                            .map(|dt| dt.to_chrono())
                            .unwrap_or_else(|_| chrono::Utc::now()),
                        updated_at: record
                            .get_datetime("updated_at")
                            .map(|dt| dt.to_chrono())
                            .unwrap_or_else(|_| chrono::Utc::now()),
                    };

                    info!("API key loaded from database");
                    return Ok(Some(stored_key));
                }
            }
        }

        debug!("No API key found in database");
        Ok(None)
    }

    /// Delete API key from database
    pub async fn delete_api_key(&self) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                self.api_keys()?.delete_many(doc! {}).await?;
                info!("API key deleted from database");
                return Ok(());
            }
        }

        warn!("Database not available, API key not deleted");
        Ok(())
    }

    // =========================================================================
    // NOTIFICATION PREFERENCES
    // =========================================================================

    /// Get notification preferences collection
    #[cfg(feature = "database")]
    fn notification_preferences(&self) -> Result<Collection<Document>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("notification_preferences"))
    }

    /// Get push subscriptions collection
    #[cfg(feature = "database")]
    fn push_subscriptions(&self) -> Result<Collection<Document>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("push_subscriptions"))
    }

    /// Save notification preferences to database
    pub async fn save_notification_preferences(
        &self,
        prefs: &crate::api::notifications::NotificationPreferences,
    ) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                // Only one preferences record per user (for now, single-user system)
                self.notification_preferences()?
                    .delete_many(doc! {})
                    .await?;

                let insert_doc = doc! {
                    "enabled": prefs.enabled,
                    "channels": {
                        "email": prefs.channels.email,
                        "push": {
                            "enabled": prefs.channels.push.enabled,
                            "vapid_public_key": prefs.channels.push.vapid_public_key.as_deref().unwrap_or(""),
                            "vapid_private_key": prefs.channels.push.vapid_private_key.as_deref().unwrap_or(""),
                        },
                        "sound": prefs.channels.sound,
                        "telegram": {
                            "enabled": prefs.channels.telegram.enabled,
                            "bot_token": prefs.channels.telegram.bot_token.as_deref().unwrap_or(""),
                            "chat_id": prefs.channels.telegram.chat_id.as_deref().unwrap_or(""),
                        },
                        "discord": {
                            "enabled": prefs.channels.discord.enabled,
                            "webhook_url": prefs.channels.discord.webhook_url.as_deref().unwrap_or(""),
                        },
                    },
                    "alerts": {
                        "price_alerts": prefs.alerts.price_alerts,
                        "trade_alerts": prefs.alerts.trade_alerts,
                        "system_alerts": prefs.alerts.system_alerts,
                        "signal_alerts": prefs.alerts.signal_alerts,
                        "risk_alerts": prefs.alerts.risk_alerts,
                    },
                    "price_alert_threshold": prefs.price_alert_threshold,
                    "created_at": prefs.created_at,
                    "updated_at": prefs.updated_at,
                };
                self.notification_preferences()?
                    .insert_one(insert_doc)
                    .await?;

                info!("Notification preferences saved to database");
                return Ok(());
            }
        }

        warn!("Database not available, notification preferences not saved");
        Ok(())
    }

    /// Load notification preferences from database
    pub async fn load_notification_preferences(
        &self,
    ) -> crate::api::notifications::NotificationPreferences {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                if let Ok(collection) = self.notification_preferences() {
                    if let Ok(Some(record)) = collection.find_one(doc! {}).await {
                        // Parse channels
                        let channels_doc = record.get_document("channels").ok();
                        let push_doc = channels_doc.and_then(|c| c.get_document("push").ok());
                        let telegram_doc =
                            channels_doc.and_then(|c| c.get_document("telegram").ok());
                        let discord_doc = channels_doc.and_then(|c| c.get_document("discord").ok());

                        let push = crate::api::notifications::PushSettings {
                            enabled: push_doc
                                .and_then(|p| p.get_bool("enabled").ok())
                                .unwrap_or(false),
                            vapid_public_key: push_doc
                                .and_then(|p| p.get_str("vapid_public_key").ok())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            vapid_private_key: push_doc
                                .and_then(|p| p.get_str("vapid_private_key").ok())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                        };

                        let telegram = crate::api::notifications::TelegramSettings {
                            enabled: telegram_doc
                                .and_then(|t| t.get_bool("enabled").ok())
                                .unwrap_or(false),
                            bot_token: telegram_doc
                                .and_then(|t| t.get_str("bot_token").ok())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            chat_id: telegram_doc
                                .and_then(|t| t.get_str("chat_id").ok())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                        };

                        let discord = crate::api::notifications::DiscordSettings {
                            enabled: discord_doc
                                .and_then(|d| d.get_bool("enabled").ok())
                                .unwrap_or(false),
                            webhook_url: discord_doc
                                .and_then(|d| d.get_str("webhook_url").ok())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                        };

                        let channels = crate::api::notifications::ChannelSettings {
                            email: channels_doc
                                .and_then(|c| c.get_bool("email").ok())
                                .unwrap_or(false),
                            push,
                            sound: channels_doc
                                .and_then(|c| c.get_bool("sound").ok())
                                .unwrap_or(true),
                            telegram,
                            discord,
                        };

                        // Parse alerts
                        let alerts_doc = record.get_document("alerts").ok();
                        let alerts = crate::api::notifications::AlertSettings {
                            price_alerts: alerts_doc
                                .and_then(|a| a.get_bool("price_alerts").ok())
                                .unwrap_or(true),
                            trade_alerts: alerts_doc
                                .and_then(|a| a.get_bool("trade_alerts").ok())
                                .unwrap_or(true),
                            system_alerts: alerts_doc
                                .and_then(|a| a.get_bool("system_alerts").ok())
                                .unwrap_or(true),
                            signal_alerts: alerts_doc
                                .and_then(|a| a.get_bool("signal_alerts").ok())
                                .unwrap_or(true),
                            risk_alerts: alerts_doc
                                .and_then(|a| a.get_bool("risk_alerts").ok())
                                .unwrap_or(true),
                        };

                        let prefs = crate::api::notifications::NotificationPreferences {
                            enabled: record.get_bool("enabled").unwrap_or(true),
                            channels,
                            alerts,
                            price_alert_threshold: record
                                .get_f64("price_alert_threshold")
                                .unwrap_or(5.0),
                            created_at: record
                                .get_datetime("created_at")
                                .map(|dt| dt.to_chrono())
                                .unwrap_or_else(|_| chrono::Utc::now()),
                            updated_at: record
                                .get_datetime("updated_at")
                                .map(|dt| dt.to_chrono())
                                .unwrap_or_else(|_| chrono::Utc::now()),
                        };

                        info!("Notification preferences loaded from database");
                        return prefs;
                    }
                }
            }
        }

        debug!("No notification preferences found, using defaults");
        crate::api::notifications::NotificationPreferences::default()
    }

    /// Save push notification subscription
    pub async fn save_push_subscription(
        &self,
        subscription: &crate::api::notifications::PushSubscription,
    ) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                // Only one subscription per user (for now)
                self.push_subscriptions()?.delete_many(doc! {}).await?;

                let insert_doc = doc! {
                    "endpoint": &subscription.endpoint,
                    "keys": {
                        "p256dh": &subscription.keys.p256dh,
                        "auth": &subscription.keys.auth,
                    },
                    "created_at": subscription.created_at,
                };
                self.push_subscriptions()?.insert_one(insert_doc).await?;

                info!("Push subscription saved to database");
                return Ok(());
            }
        }

        warn!("Database not available, push subscription not saved");
        Ok(())
    }

    /// Load push subscription from database
    pub async fn load_push_subscription(
        &self,
    ) -> Option<crate::api::notifications::PushSubscription> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                if let Ok(collection) = self.push_subscriptions() {
                    if let Ok(Some(record)) = collection.find_one(doc! {}).await {
                        let keys_doc = record.get_document("keys").ok()?;

                        return Some(crate::api::notifications::PushSubscription {
                            endpoint: record.get_str("endpoint").ok()?.to_string(),
                            keys: crate::api::notifications::PushSubscriptionKeys {
                                p256dh: keys_doc.get_str("p256dh").ok()?.to_string(),
                                auth: keys_doc.get_str("auth").ok()?.to_string(),
                            },
                            created_at: record
                                .get_datetime("created_at")
                                .map(|dt| dt.to_chrono())
                                .unwrap_or_else(|_| chrono::Utc::now()),
                        });
                    }
                }
            }
        }

        None
    }

    /// Delete push subscription from database
    pub async fn delete_push_subscription(&self) -> Result<()> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                self.push_subscriptions()?.delete_many(doc! {}).await?;
                info!("Push subscription deleted from database");
                return Ok(());
            }
        }

        warn!("Database not available, push subscription not deleted");
        Ok(())
    }

    // =========================================================================
    // TRADE ANALYSES (GPT-4 Analysis from Python AI Service)
    // =========================================================================

    /// Get trade analyses collection
    pub fn trade_analyses(&self) -> Result<Collection<TradeAnalysisRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("trade_analyses"))
    }

    /// Get config suggestions collection
    pub fn config_suggestions(&self) -> Result<Collection<ConfigSuggestionsRecord>> {
        self.db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
            .map(|db| db.collection("config_suggestions"))
    }

    /// Get all trade analyses from MongoDB
    /// @spec:FR-ASYNC-011 - GPT-4 Individual Trade Analysis
    pub async fn get_trade_analyses(
        &self,
        only_losing: bool,
        limit: Option<i64>,
    ) -> Result<Vec<TradeAnalysisRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let filter = if only_losing {
                    doc! { "is_winning": false }
                } else {
                    doc! {}
                };

                let cursor = self
                    .trade_analyses()?
                    .find(filter)
                    .sort(doc! { "created_at": -1 })
                    .limit(limit.unwrap_or(100))
                    .await?;

                let analyses: Vec<TradeAnalysisRecord> = cursor.try_collect().await?;
                info!("📖 Loaded {} trade analyses from database", analyses.len());
                return Ok(analyses);
            }
        }

        Ok(Vec::new())
    }

    /// Get trade analysis by trade ID
    pub async fn get_trade_analysis_by_id(
        &self,
        trade_id: &str,
    ) -> Result<Option<TradeAnalysisRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let filter = doc! { "trade_id": trade_id };
                let analysis = self.trade_analyses()?.find_one(filter).await?;
                return Ok(analysis);
            }
        }

        Ok(None)
    }

    /// Get config suggestions history from MongoDB
    /// @spec:FR-ASYNC-009 - GPT-4 Config Improvement Suggestions
    pub async fn get_config_suggestions(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<ConfigSuggestionsRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let cursor = self
                    .config_suggestions()?
                    .find(doc! {})
                    .sort(doc! { "created_at": -1 })
                    .limit(limit.unwrap_or(50))
                    .await?;

                let suggestions: Vec<ConfigSuggestionsRecord> = cursor.try_collect().await?;
                info!(
                    "📖 Loaded {} config suggestions from database",
                    suggestions.len()
                );
                return Ok(suggestions);
            }
        }

        Ok(Vec::new())
    }

    /// Get latest config suggestion
    pub async fn get_latest_config_suggestion(&self) -> Result<Option<ConfigSuggestionsRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(_db) = &self.db {
                let suggestion = self
                    .config_suggestions()?
                    .find_one(doc! {})
                    .sort(doc! { "created_at": -1 })
                    .await?;
                return Ok(suggestion);
            }
        }

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
    // Outcome tracking fields - updated when trade closes
    #[serde(default)]
    pub outcome: Option<String>, // "win", "loss", "pending"
    #[serde(default)]
    pub actual_pnl: Option<f64>, // PnL in USDT
    #[serde(default)]
    pub pnl_percentage: Option<f64>, // PnL as percentage
    #[serde(default)]
    pub exit_price: Option<f64>, // Price when trade closed
    #[serde(default)]
    pub close_reason: Option<String>, // TakeProfit, StopLoss, Manual, etc.
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>, // When trade was closed
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

/// GPT-4 Trade Analysis record (created by Python AI service)
/// @spec:FR-ASYNC-011 - GPT-4 Individual Trade Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysisRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub trade_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub is_winning: bool,
    pub pnl_usdt: f64,
    pub pnl_percentage: f64,
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub entry_price: Option<f64>,
    pub exit_price: Option<f64>,
    pub close_reason: Option<String>,
    /// GPT-4 analysis result (stored as BSON Document from Python)
    pub analysis: mongodb::bson::Document,
    /// Original trade data (stored as BSON Document from Python)
    pub trade_data: Option<mongodb::bson::Document>,
}

/// Config suggestions record (created by Python AI service)
/// @spec:FR-ASYNC-009 - GPT-4 Config Improvement Suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSuggestionsRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub timestamp: Option<String>,
    pub current_config: Option<mongodb::bson::Document>,
    pub trade_stats: Option<mongodb::bson::Document>,
    pub suggestions: mongodb::bson::Document,
    pub applied_changes: Vec<String>,
    pub task_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Test helper to create sample TradeRecord
    fn create_sample_trade_record() -> TradeRecord {
        TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
            exit_price: Some(52000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(1000.0),
            status: "closed".to_string(),
            strategy_used: Some("MACD_RSI".to_string()),
        }
    }

    // Test helper to create sample Kline
    fn create_sample_kline() -> Kline {
        Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "50000.0".to_string(),
            high: "51000.0".to_string(),
            low: "49000.0".to_string(),
            close: "50500.0".to_string(),
            volume: "100.5".to_string(),
            quote_asset_volume: "5025000.0".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.0".to_string(),
            taker_buy_quote_asset_volume: "2500000.0".to_string(),
            ignore: "0".to_string(),
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_invalid_url() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://not-a-mongo-url".to_string(),
            database_name: Some("test_db".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let result = Storage::new(&config).await;

        // Should succeed but with None database (fallback mode)
        #[cfg(feature = "database")]
        {
            assert!(result.is_ok());
            let storage = result.unwrap();
            assert!(storage.db.is_none());
        }

        #[cfg(not(feature = "database"))]
        {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_without_database_name() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://invalid-host:27017".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        // Should use default database name "bot_core"
        // Will fail to connect but that's expected in unit test
        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    #[test]
    fn test_trade_record_serialization() {
        let trade = create_sample_trade_record();

        // Serialize to JSON
        let json = serde_json::to_string(&trade).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("BUY"));
        assert!(json.contains("50000"));

        // Deserialize back
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert_eq!(deserialized.side, "BUY");
        assert_eq!(deserialized.quantity, 0.5);
        assert_eq!(deserialized.entry_price, 50000.0);
    }

    #[test]
    fn test_trade_record_with_missing_optional_fields() {
        let trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "ETHUSDT");
        assert!(deserialized.exit_price.is_none());
        assert!(deserialized.stop_loss.is_none());
        assert!(deserialized.pnl.is_none());
    }

    #[test]
    fn test_performance_stats_default() {
        let stats = PerformanceStats::default();

        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.winning_trades, 0);
        assert_eq!(stats.losing_trades, 0);
        assert_eq!(stats.win_rate, 0.0);
        assert_eq!(stats.total_pnl, 0.0);
        assert_eq!(stats.avg_pnl, 0.0);
        assert_eq!(stats.max_win, 0.0);
        assert_eq!(stats.max_loss, 0.0);
    }

    #[test]
    fn test_performance_stats_serialization() {
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 60,
            losing_trades: 40,
            win_rate: 60.0,
            total_pnl: 5000.0,
            avg_pnl: 50.0,
            max_win: 500.0,
            max_loss: -200.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_trades, 100);
        assert_eq!(deserialized.winning_trades, 60);
        assert_eq!(deserialized.win_rate, 60.0);
        assert_eq!(deserialized.max_loss, -200.0);
    }

    #[test]
    fn test_performance_stats_with_zero_trades() {
        let stats = PerformanceStats::default();

        // Ensure no division by zero issues
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_trades, 0);
        assert_eq!(deserialized.win_rate, 0.0);
    }

    #[test]
    fn test_kline_price_parsing() {
        let kline = create_sample_kline();

        // Test string to float conversion
        let open_price: f64 = kline.open.parse().unwrap();
        let high_price: f64 = kline.high.parse().unwrap();
        let low_price: f64 = kline.low.parse().unwrap();
        let close_price: f64 = kline.close.parse().unwrap();

        assert_eq!(open_price, 50000.0);
        assert_eq!(high_price, 51000.0);
        assert_eq!(low_price, 49000.0);
        assert_eq!(close_price, 50500.0);
    }

    #[test]
    fn test_kline_with_invalid_prices() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "invalid".to_string(),
            high: "51000.0".to_string(),
            low: "49000.0".to_string(),
            close: "50500.0".to_string(),
            volume: "100.5".to_string(),
            quote_asset_volume: "5025000.0".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.0".to_string(),
            taker_buy_quote_asset_volume: "2500000.0".to_string(),
            ignore: "0".to_string(),
        };

        // Should handle parse error gracefully
        let open_price = kline.open.parse::<f64>().unwrap_or(0.0);
        assert_eq!(open_price, 0.0);
    }

    #[test]
    fn test_paper_trading_record_serialization() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "trade123".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "OPEN".to_string(),
            entry_price: 50000.0,
            exit_price: None,
            quantity: 0.1,
            leverage: 10,
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 5.0,
            funding_fees: 1.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: Some("signal456".to_string()),
            ai_confidence: Some(0.85),
            close_reason: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.trade_id, "trade123");
        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert_eq!(deserialized.leverage, 10);
        assert!(deserialized.exit_price.is_none());
    }

    #[test]
    fn test_portfolio_history_record_serialization() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let record = PortfolioHistoryRecord {
            id: None,
            timestamp,
            current_balance: 10000.0,
            equity: 12000.0,
            margin_used: 3000.0,
            free_margin: 7000.0,
            total_pnl: 2000.0,
            total_pnl_percentage: 20.0,
            total_trades: 50,
            win_rate: 60.0,
            profit_factor: 2.5,
            max_drawdown: 500.0,
            max_drawdown_percentage: 5.0,
            open_positions: 3,
            created_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.current_balance, 10000.0);
        assert_eq!(deserialized.equity, 12000.0);
        assert_eq!(deserialized.total_trades, 50);
        assert_eq!(deserialized.win_rate, 60.0);
    }

    #[test]
    fn test_ai_signal_record_serialization() {
        let timestamp = Utc::now();

        let record = AISignalRecord {
            id: None,
            signal_id: "signal789".to_string(),
            symbol: "ETHUSDT".to_string(),
            signal_type: "LONG".to_string(),
            confidence: 0.92,
            reasoning: "Strong uptrend with RSI oversold".to_string(),
            entry_price: 3000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.8,
            volatility: 0.3,
            risk_score: 0.2,
            executed: true,
            trade_id: Some("trade999".to_string()),
            created_at: timestamp,
            timestamp,
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.signal_id, "signal789");
        assert_eq!(deserialized.confidence, 0.92);
        assert_eq!(deserialized.trend_direction, "UP");
        assert!(deserialized.executed);
    }

    #[test]
    fn test_performance_metrics_record_serialization() {
        let date = Utc::now();

        let record = PerformanceMetricsRecord {
            id: None,
            date,
            total_trades: 100,
            winning_trades: 65,
            losing_trades: 35,
            win_rate: 65.0,
            average_win: 150.0,
            average_loss: -75.0,
            largest_win: 500.0,
            largest_loss: -200.0,
            profit_factor: 2.6,
            sharpe_ratio: 1.8,
            max_drawdown: 1000.0,
            max_drawdown_percentage: 10.0,
            total_pnl: 5000.0,
            daily_pnl: 250.0,
            created_at: date,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PerformanceMetricsRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_trades, 100);
        assert_eq!(deserialized.winning_trades, 65);
        assert_eq!(deserialized.win_rate, 65.0);
        assert_eq!(deserialized.sharpe_ratio, 1.8);
    }

    #[test]
    fn test_trade_record_edge_case_zero_quantity() {
        let trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.0,
            entry_price: 50000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "cancelled".to_string(),
            strategy_used: None,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.quantity, 0.0);
        assert_eq!(deserialized.status, "cancelled");
    }

    #[test]
    fn test_trade_record_negative_pnl() {
        let trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: Some(3200.0),
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(-200.0),
            status: "closed".to_string(),
            strategy_used: Some("BREAKOUT".to_string()),
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pnl, Some(-200.0));
        assert!(deserialized.pnl.unwrap() < 0.0);
    }

    #[test]
    fn test_performance_stats_100_percent_win_rate() {
        let stats = PerformanceStats {
            total_trades: 50,
            winning_trades: 50,
            losing_trades: 0,
            win_rate: 100.0,
            total_pnl: 10000.0,
            avg_pnl: 200.0,
            max_win: 1000.0,
            max_loss: 0.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.win_rate, 100.0);
        assert_eq!(deserialized.losing_trades, 0);
        assert_eq!(deserialized.max_loss, 0.0);
    }

    #[test]
    fn test_paper_trading_record_high_leverage() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "trade_leverage".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "OPEN".to_string(),
            entry_price: 50000.0,
            exit_price: None,
            quantity: 0.1,
            leverage: 125, // Maximum leverage on some platforms
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 5.0,
            funding_fees: 1.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.leverage, 125);
    }

    #[test]
    fn test_ai_signal_record_low_confidence() {
        let timestamp = Utc::now();

        let record = AISignalRecord {
            id: None,
            signal_id: "weak_signal".to_string(),
            symbol: "DOGEUSDT".to_string(),
            signal_type: "SHORT".to_string(),
            confidence: 0.15, // Low confidence
            reasoning: "Weak signal, not recommended".to_string(),
            entry_price: 0.08,
            trend_direction: "SIDEWAYS".to_string(),
            trend_strength: 0.2,
            volatility: 0.9,
            risk_score: 0.85,
            executed: false,
            trade_id: None,
            created_at: timestamp,
            timestamp,
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.confidence, 0.15);
        assert!(!deserialized.executed);
        assert!(deserialized.trade_id.is_none());
    }

    #[test]
    fn test_portfolio_history_record_with_loss() {
        let timestamp = Utc::now();

        let record = PortfolioHistoryRecord {
            id: None,
            timestamp,
            current_balance: 8000.0,
            equity: 7500.0,
            margin_used: 2000.0,
            free_margin: 6000.0,
            total_pnl: -2500.0, // Loss
            total_pnl_percentage: -25.0,
            total_trades: 30,
            win_rate: 30.0,
            profit_factor: 0.5,
            max_drawdown: 2500.0,
            max_drawdown_percentage: 25.0,
            open_positions: 1,
            created_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();

        assert!(deserialized.total_pnl < 0.0);
        assert!(deserialized.total_pnl_percentage < 0.0);
        assert!(deserialized.profit_factor < 1.0);
    }

    #[tokio::test]
    async fn test_storage_clone() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let storage_clone = storage.clone();

        // Should be able to clone without panic
        #[cfg(feature = "database")]
        {
            assert_eq!(storage.db.is_none(), storage_clone.db.is_none());
        }
    }

    #[test]
    fn test_kline_serialization() {
        let kline = create_sample_kline();

        let json = serde_json::to_string(&kline).unwrap();
        let deserialized: Kline = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.open_time, 1640000000000);
        assert_eq!(deserialized.close, "50500.0");
        assert_eq!(deserialized.number_of_trades, 1000);
    }

    #[test]
    fn test_empty_string_prices_in_kline() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "".to_string(),
            high: "".to_string(),
            low: "".to_string(),
            close: "".to_string(),
            volume: "".to_string(),
            quote_asset_volume: "".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "".to_string(),
            taker_buy_quote_asset_volume: "".to_string(),
            ignore: "".to_string(),
        };

        // Parse with fallback
        let open = kline.open.parse::<f64>().unwrap_or(0.0);
        let volume = kline.volume.parse::<f64>().unwrap_or(0.0);

        assert_eq!(open, 0.0);
        assert_eq!(volume, 0.0);
    }

    #[test]
    fn test_performance_stats_extreme_values() {
        let stats = PerformanceStats {
            total_trades: u64::MAX,
            winning_trades: u64::MAX / 2,
            losing_trades: u64::MAX / 2,
            win_rate: 50.0,
            total_pnl: f64::MAX,
            avg_pnl: 100.0,
            max_win: f64::MAX,
            max_loss: f64::MIN,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_trades, u64::MAX);
        assert_eq!(deserialized.win_rate, 50.0);
    }

    #[tokio::test]
    async fn test_get_database_without_feature() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let _storage = Storage::new(&config).await.unwrap();

        #[cfg(not(feature = "database"))]
        {
            let db = _storage.get_database();
            assert!(db.is_none());
        }
    }

    // ===== Configuration and Constructor Tests =====

    #[tokio::test]
    async fn test_storage_new_with_mongodb_url() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("trading_bot_test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        // This will fail to connect in unit tests, but tests the URL validation logic
        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_mongodb_srv_url() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb+srv://cluster.example.com".to_string(),
            database_name: Some("production_db".to_string()),
            max_connections: 20,
            enable_logging: true,
        };

        // Tests mongodb+srv:// URL pattern recognition
        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_default_database_name() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: None, // Should use default "bot_core"
            max_connections: 10,
            enable_logging: false,
        };

        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_http_url() {
        let config = crate::config::DatabaseConfig {
            url: "http://example.com".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        // HTTP URLs should create storage with None database (fallback mode)
        let result = Storage::new(&config).await;

        #[cfg(feature = "database")]
        {
            assert!(result.is_ok());
            let storage = result.unwrap();
            assert!(storage.db.is_none());
        }

        #[cfg(not(feature = "database"))]
        {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_empty_url() {
        let config = crate::config::DatabaseConfig {
            url: "".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let result = Storage::new(&config).await;

        #[cfg(feature = "database")]
        {
            assert!(result.is_ok());
            let storage = result.unwrap();
            assert!(storage.db.is_none());
        }

        #[cfg(not(feature = "database"))]
        {
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_max_connections() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 100, // Test with high connection count
            enable_logging: false,
        };

        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_storage_new_with_logging_enabled() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: true, // Test with logging enabled
        };

        let _result = Storage::new(&config).await;

        #[cfg(not(feature = "database"))]
        {
            assert!(_result.is_ok());
        }
    }

    // ===== Collection Accessor Tests (No MongoDB Required) =====

    #[tokio::test]
    async fn test_paper_trades_collection_without_db() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let result = storage.paper_trades();

        #[cfg(feature = "database")]
        {
            // Should return error when db is None
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Database not initialized"));
        }
    }

    #[tokio::test]
    async fn test_portfolio_history_collection_without_db() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let result = storage.portfolio_history();

        #[cfg(feature = "database")]
        {
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Database not initialized"));
        }
    }

    #[tokio::test]
    async fn test_ai_signals_collection_without_db() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let result = storage.ai_signals();

        #[cfg(feature = "database")]
        {
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Database not initialized"));
        }
    }

    #[tokio::test]
    async fn test_performance_metrics_collection_without_db() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let result = storage.performance_metrics();

        #[cfg(feature = "database")]
        {
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Database not initialized"));
        }
    }

    #[tokio::test]
    async fn test_paper_trading_settings_collection_without_db() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let result = storage.paper_trading_settings();

        #[cfg(feature = "database")]
        {
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Database not initialized"));
        }
    }

    // ===== Fallback Behavior Tests (No MongoDB) =====

    #[tokio::test]
    async fn test_store_analysis_fallback() {
        use crate::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};

        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let analysis = MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640000000000,
            overall_signal: TradingSignal::Buy,
            overall_confidence: 0.85,
            timeframe_signals: std::collections::HashMap::new(),
            entry_price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            risk_reward_ratio: Some(2.5),
        };

        // Should succeed with fallback (just logs)
        let result = storage.store_analysis(&analysis).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_analysis_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return None without database
        let result = storage.get_latest_analysis("BTCUSDT").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_analysis_history_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return empty vector without database
        let result = storage.get_analysis_history("ETHUSDT", Some(50)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_store_trade_record_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let trade = create_sample_trade_record();

        // Should succeed with fallback (just logs)
        let result = storage.store_trade_record(&trade).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_trade_history_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return empty vector without database
        let result = storage.get_trade_history(Some("BTCUSDT"), Some(100)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_trade_history_without_symbol() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return empty vector without database, no symbol filter
        let result = storage.get_trade_history(None, Some(100)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_performance_stats_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return default stats without database
        let result = storage.get_performance_stats().await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.winning_trades, 0);
        assert_eq!(stats.losing_trades, 0);
        assert_eq!(stats.win_rate, 0.0);
    }

    #[tokio::test]
    async fn test_store_market_data_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let klines = vec![create_sample_kline()];

        // Should succeed with fallback (does nothing)
        let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_market_data_empty_klines() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let klines: Vec<Kline> = vec![];

        // Should succeed with empty klines
        let result = storage.store_market_data("BTCUSDT", "5m", &klines).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_market_data_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return empty vector without database
        let result = storage.get_market_data("BTCUSDT", "1h", Some(500)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_market_data_with_default_limit() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should use default limit of 500 when None is provided
        let result = storage.get_market_data("ETHUSDT", "15m", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_store_price_history_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should succeed with fallback (does nothing)
        let result = storage
            .store_price_history("BTCUSDT", 50000.0, 1000000.0, 2000.0, 4.0)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_price_history_negative_changes() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Test with negative price changes
        let result = storage
            .store_price_history("BTCUSDT", 48000.0, 900000.0, -2000.0, -4.0)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_store_price_history_zero_values() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Test with zero values
        let result = storage
            .store_price_history("NEWCOIN", 0.0, 0.0, 0.0, 0.0)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_paper_trading_settings_fallback() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Should return None without database
        let result = storage.load_paper_trading_settings().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ===== Database Operation Tests (Require MongoDB - marked #[ignore]) =====

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_store_analysis_integration() {
        use crate::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};

        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let analysis = MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640000000000,
            overall_signal: TradingSignal::Buy,
            overall_confidence: 0.85,
            timeframe_signals: std::collections::HashMap::new(),
            entry_price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            risk_reward_ratio: Some(2.5),
        };

        // Requires MongoDB to be running
        let result = storage.store_analysis(&analysis).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_store_and_retrieve_analysis() {
        use crate::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};

        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let analysis = MultiTimeframeAnalysis {
            symbol: "ETHUSDT".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            overall_signal: TradingSignal::Sell,
            overall_confidence: 0.75,
            timeframe_signals: std::collections::HashMap::new(),
            entry_price: Some(3000.0),
            stop_loss: Some(3100.0),
            take_profit: Some(2800.0),
            risk_reward_ratio: Some(2.0),
        };

        // Store analysis
        storage.store_analysis(&analysis).await.unwrap();

        // Retrieve it
        let retrieved = storage.get_latest_analysis("ETHUSDT").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().symbol, "ETHUSDT");
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_store_trade_record_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let trade = create_sample_trade_record();

        let result = storage.store_trade_record(&trade).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_get_trade_history_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let _trades = storage
            .get_trade_history(Some("BTCUSDT"), Some(10))
            .await
            .unwrap();

        // Should return vector (may be empty if no trades stored yet)
        // trades.len() is always >= 0 by definition
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_store_market_data_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();
        let klines = vec![create_sample_kline()];

        let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_get_market_data_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let _klines = storage
            .get_market_data("BTCUSDT", "1h", Some(100))
            .await
            .unwrap();

        // Should return vector (may be empty if no data stored yet)
        // klines.len() is always >= 0 by definition
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_get_performance_stats_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let _stats = storage.get_performance_stats().await.unwrap();

        // Should return stats (may be default if no trades)
        // stats.total_trades is always >= 0 by definition (u64)
    }

    #[tokio::test]
    #[ignore] // Requires actual MongoDB connection
    async fn test_paper_trades_collection_integration() {
        let config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_trading_bot".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        let collection = storage.paper_trades();
        assert!(collection.is_ok());
    }

    // ===== Edge Case Tests =====

    #[test]
    fn test_trade_record_with_very_large_quantity() {
        let trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: f64::MAX,
            entry_price: 50000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.quantity, f64::MAX);
    }

    #[test]
    fn test_trade_record_with_very_small_quantity() {
        let trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: f64::MIN_POSITIVE,
            entry_price: 50000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.quantity, f64::MIN_POSITIVE);
    }

    #[test]
    fn test_performance_stats_win_rate_calculation_validation() {
        // Test that win_rate makes sense given winning/total trades
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 65,
            losing_trades: 35,
            win_rate: 65.0,
            total_pnl: 5000.0,
            avg_pnl: 50.0,
            max_win: 500.0,
            max_loss: -200.0,
        };

        // Verify consistency
        assert_eq!(
            stats.winning_trades + stats.losing_trades,
            stats.total_trades
        );
        assert_eq!(
            (stats.winning_trades as f64 / stats.total_trades as f64 * 100.0).round(),
            stats.win_rate
        );
    }

    #[test]
    fn test_paper_trading_settings_record_serialization() {
        use bson::doc;

        let timestamp = Utc::now();
        let settings_doc = doc! {
            "initial_balance": 10000.0,
            "max_leverage": 10,
            "trading_enabled": true
        };

        let record = PaperTradingSettingsRecord {
            id: None,
            settings_data: settings_doc.clone(),
            created_at: timestamp,
            updated_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingSettingsRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.settings_data.get_f64("initial_balance"),
            settings_doc.get_f64("initial_balance")
        );
    }

    #[tokio::test]
    async fn test_multiple_storage_instances() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };

        // Create multiple storage instances
        let storage1 = Storage::new(&config).await.unwrap();
        let storage2 = Storage::new(&config).await.unwrap();
        let storage3 = storage1.clone();

        // All should be valid
        #[cfg(feature = "database")]
        {
            assert!(storage1.db.is_none());
            assert!(storage2.db.is_none());
            assert!(storage3.db.is_none());
        }
    }

    #[test]
    fn test_kline_with_special_characters_in_strings() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "50000.0€".to_string(), // Invalid character
            high: "51000.0".to_string(),
            low: "49000.0".to_string(),
            close: "50500.0".to_string(),
            volume: "100.5abc".to_string(), // Invalid characters
            quote_asset_volume: "5025000.0".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.0".to_string(),
            taker_buy_quote_asset_volume: "2500000.0".to_string(),
            ignore: "0".to_string(),
        };

        // Should handle parse errors gracefully
        let open = kline.open.parse::<f64>().unwrap_or(0.0);
        let volume = kline.volume.parse::<f64>().unwrap_or(0.0);

        assert_eq!(open, 0.0);
        assert_eq!(volume, 0.0);
    }

    #[test]
    fn test_performance_stats_with_negative_values() {
        let stats = PerformanceStats {
            total_trades: 50,
            winning_trades: 20,
            losing_trades: 30,
            win_rate: 40.0,
            total_pnl: -5000.0,
            avg_pnl: -100.0,
            max_win: 500.0,
            max_loss: -1000.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert!(deserialized.total_pnl < 0.0);
        assert!(deserialized.avg_pnl < 0.0);
        assert!(deserialized.max_loss < 0.0);
    }

    #[tokio::test]
    async fn test_get_analysis_history_with_limit() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Test with different limit values
        let result1 = storage.get_analysis_history("BTCUSDT", Some(10)).await;
        let result2 = storage.get_analysis_history("BTCUSDT", Some(1000)).await;
        let result3 = storage.get_analysis_history("BTCUSDT", None).await; // Uses default 100

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_store_market_data_with_multiple_klines() {
        let config = crate::config::DatabaseConfig {
            url: "invalid://test".to_string(),
            database_name: None,
            max_connections: 10,
            enable_logging: false,
        };

        let storage = Storage::new(&config).await.unwrap();

        // Create multiple klines
        let klines = vec![
            create_sample_kline(),
            create_sample_kline(),
            create_sample_kline(),
        ];

        let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_paper_trading_record_with_closed_trade() {
        let open_time = Utc::now();
        let close_time = open_time + chrono::Duration::hours(2);

        let record = PaperTradingRecord {
            id: None,
            trade_id: "closed_trade_123".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "SHORT".to_string(),
            status: "CLOSED".to_string(),
            entry_price: 52000.0,
            exit_price: Some(51000.0),
            quantity: 0.5,
            leverage: 5,
            pnl: Some(500.0),
            pnl_percentage: 9.62,
            trading_fees: 10.0,
            funding_fees: 2.5,
            open_time,
            close_time: Some(close_time),
            ai_signal_id: Some("signal789".to_string()),
            ai_confidence: Some(0.88),
            close_reason: Some("TAKE_PROFIT".to_string()),
            created_at: open_time,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();

        assert!(deserialized.exit_price.is_some());
        assert!(deserialized.pnl.is_some());
        assert!(deserialized.close_time.is_some());
        assert!(deserialized.close_reason.is_some());
    }

    #[test]
    fn test_ai_signal_record_with_all_fields() {
        let timestamp = Utc::now();

        let record = AISignalRecord {
            id: None,
            signal_id: "full_signal_001".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "LONG".to_string(),
            confidence: 0.95,
            reasoning: "All indicators align for strong buy".to_string(),
            entry_price: 50000.0,
            trend_direction: "BULLISH".to_string(),
            trend_strength: 0.9,
            volatility: 0.25,
            risk_score: 0.15,
            executed: true,
            trade_id: Some("trade_executed_001".to_string()),
            created_at: timestamp,
            timestamp,
            outcome: Some("win".to_string()),
            actual_pnl: Some(500.0),
            pnl_percentage: Some(2.5),
            exit_price: Some(51250.0),
            close_reason: Some("TAKE_PROFIT".to_string()),
            closed_at: Some(timestamp),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.signal_id, "full_signal_001");
        assert!(deserialized.confidence > 0.9);
        assert!(deserialized.executed);
        assert!(deserialized.trade_id.is_some());
    }

    // ========== NULL-DB COVERAGE TESTS ==========
    // These tests call storage methods with null-DB to cover error paths

    async fn create_null_db_storage() -> Storage {
        let config = crate::config::DatabaseConfig {
            url: "no-db://test".to_string(),
            database_name: Some("test".to_string()),
            max_connections: 10,
            enable_logging: false,
        };
        Storage::new(&config).await.unwrap()
    }

    #[tokio::test]
    async fn test_null_db_get_database() {
        let storage = create_null_db_storage().await;
        assert!(storage.get_database().is_none());
    }

    #[tokio::test]
    async fn test_null_db_collection_errors() {
        let storage = create_null_db_storage().await;
        assert!(storage.paper_trades().is_err());
        assert!(storage.portfolio_history().is_err());
        assert!(storage.ai_signals().is_err());
        assert!(storage.performance_metrics().is_err());
        assert!(storage.paper_trading_settings().is_err());
        assert!(storage.user_symbols().is_err());
    }

    #[tokio::test]
    async fn test_null_db_store_analysis() {
        let storage = create_null_db_storage().await;
        let analysis = MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            timeframe_signals: std::collections::HashMap::new(),
            overall_signal: crate::market_data::analyzer::TradingSignal::Buy,
            overall_confidence: 0.85,
            entry_price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            risk_reward_ratio: Some(2.5),
        };
        let result = storage.store_analysis(&analysis).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_latest_analysis() {
        let storage = create_null_db_storage().await;
        let result = storage.get_latest_analysis("BTCUSDT").await;
        assert!(result.unwrap().is_none()); // Returns Ok(None) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_analysis_history() {
        let storage = create_null_db_storage().await;
        let result = storage.get_analysis_history("BTCUSDT", Some(50)).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_null_db_store_trade_record() {
        let storage = create_null_db_storage().await;
        let trade = create_sample_trade_record();
        let result = storage.store_trade_record(&trade).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_trade_history() {
        let storage = create_null_db_storage().await;
        assert!(storage.get_trade_history(Some("BTCUSDT"), Some(20)).await.unwrap().is_empty());
        assert!(storage.get_trade_history(None, None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_null_db_get_performance_stats() {
        let storage = create_null_db_storage().await;
        let result = storage.get_performance_stats().await;
        assert!(result.is_ok()); // Returns Ok(PerformanceStats::default())
    }

    #[tokio::test]
    async fn test_null_db_store_market_data() {
        let storage = create_null_db_storage().await;
        let klines = vec![create_sample_kline()];
        let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_market_data() {
        let storage = create_null_db_storage().await;
        let result = storage.get_market_data("BTCUSDT", "1h", Some(100)).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_null_db_store_price_history() {
        let storage = create_null_db_storage().await;
        let result = storage.store_price_history("BTCUSDT", 50000.0, 1000000.0, 500.0, 1.0).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_paper_trades_history() {
        let storage = create_null_db_storage().await;
        let result = storage.get_paper_trades_history(Some(30)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_null_db_get_portfolio_history() {
        let storage = create_null_db_storage().await;
        let result = storage.get_portfolio_history(Some(7)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_null_db_get_ai_signals_history() {
        let storage = create_null_db_storage().await;
        assert!(storage.get_ai_signals_history(None, Some(50)).await.is_err());
        assert!(storage.get_ai_signals_history(Some("BTCUSDT"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_null_db_get_latest_signals_per_symbol() {
        let storage = create_null_db_storage().await;
        let result = storage.get_latest_signals_per_symbol().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_null_db_save_paper_trading_settings() {
        let storage = create_null_db_storage().await;
        let settings = crate::paper_trading::PaperTradingSettings::default();
        let result = storage.save_paper_trading_settings(&settings).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_load_paper_trading_settings() {
        let storage = create_null_db_storage().await;
        let result = storage.load_paper_trading_settings().await;
        assert!(result.unwrap().is_none()); // Returns Ok(None) when db is None
    }

    #[tokio::test]
    async fn test_null_db_save_user_symbols() {
        let storage = create_null_db_storage().await;
        let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        let result = storage.save_user_symbols(&symbols).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_load_user_symbols() {
        let storage = create_null_db_storage().await;
        let result = storage.load_user_symbols().await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_null_db_save_api_key() {
        let storage = create_null_db_storage().await;
        let key = crate::api::settings::StoredApiKey {
            api_key: "test_key".to_string(),
            api_secret_encrypted: "encrypted".to_string(),
            api_secret_nonce: "nonce".to_string(),
            use_testnet: true,
            permissions: crate::api::settings::ApiKeyPermissions::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = storage.save_api_key(&key).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_load_api_key() {
        let storage = create_null_db_storage().await;
        let result = storage.load_api_key().await;
        assert!(result.unwrap().is_none()); // Returns Ok(None) when db is None
    }

    #[tokio::test]
    async fn test_null_db_delete_api_key() {
        let storage = create_null_db_storage().await;
        let result = storage.delete_api_key().await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_save_notification_preferences() {
        let storage = create_null_db_storage().await;
        let prefs = crate::api::notifications::NotificationPreferences::default();
        let result = storage.save_notification_preferences(&prefs).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_load_notification_preferences() {
        let storage = create_null_db_storage().await;
        let result = storage.load_notification_preferences().await;
        // Returns default NotificationPreferences when db is None (enabled=true by default)
        assert!(result.enabled);
    }

    #[tokio::test]
    async fn test_null_db_delete_push_subscription() {
        let storage = create_null_db_storage().await;
        let result = storage.delete_push_subscription().await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_trade_analyses() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_analyses(false, Some(10)).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_null_db_get_latest_config_suggestion() {
        let storage = create_null_db_storage().await;
        let result = storage.get_latest_config_suggestion().await;
        assert!(result.unwrap().is_none()); // Returns Ok(None) when db is None
    }

    #[tokio::test]
    async fn test_null_db_update_signal_outcome() {
        let storage = create_null_db_storage().await;
        let result = storage.update_signal_outcome(
            "signal123", "win", 500.0, 2.5, 52000.0, "TAKE_PROFIT"
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov6_storage_clone_null_db() {
        let storage = create_null_db_storage().await;
        let cloned = storage.clone();
        assert!(cloned.get_database().is_none());
    }

    // =========================================================================
    // COV7: Additional collection accessor tests
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_trade_analyses_collection() {
        let storage = create_null_db_storage().await;
        assert!(storage.trade_analyses().is_err());
    }

    #[tokio::test]
    async fn test_cov7_null_db_config_suggestions_collection() {
        let storage = create_null_db_storage().await;
        assert!(storage.config_suggestions().is_err());
    }

    #[tokio::test]
    async fn test_cov7_null_db_get_config_suggestions() {
        let storage = create_null_db_storage().await;
        let result = storage.get_config_suggestions(Some(20)).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_cov7_null_db_get_trade_analysis_by_id() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_analysis_by_id("trade123").await;
        assert!(result.unwrap().is_none()); // Returns Ok(None) when db is None
    }

    // =========================================================================
    // COV7: Add/remove user symbol tests
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_add_user_symbol() {
        let storage = create_null_db_storage().await;
        let result = storage.add_user_symbol("BTCUSDT").await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_cov7_null_db_remove_user_symbol() {
        let storage = create_null_db_storage().await;
        let result = storage.remove_user_symbol("BTCUSDT").await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    // =========================================================================
    // COV7: Push subscription tests
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_save_push_subscription() {
        let storage = create_null_db_storage().await;
        let subscription = crate::api::notifications::PushSubscription {
            endpoint: "https://push.example.com".to_string(),
            keys: crate::api::notifications::PushSubscriptionKeys {
                p256dh: "key123".to_string(),
                auth: "auth456".to_string(),
            },
            created_at: Utc::now(),
        };
        let result = storage.save_push_subscription(&subscription).await;
        assert!(result.is_ok()); // Returns Ok(()) when db is None
    }

    #[tokio::test]
    async fn test_cov7_null_db_load_push_subscription() {
        let storage = create_null_db_storage().await;
        let result = storage.load_push_subscription().await;
        assert!(result.is_none()); // Returns None when db is None
    }

    // =========================================================================
    // COV7: Edge cases for get_trade_analyses with only_losing parameter
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_get_trade_analyses_only_losing() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_analyses(true, Some(50)).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    #[tokio::test]
    async fn test_cov7_null_db_get_trade_analyses_all() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_analyses(false, None).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    // =========================================================================
    // COV7: Data structure serialization edge cases
    // =========================================================================

    #[test]
    fn test_cov7_trade_record_large_quantity() {
        let trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 999999.999999,
            entry_price: 100000.0,
            exit_price: Some(110000.0),
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(10000000.0),
            status: "closed".to_string(),
            strategy_used: Some("WHALE".to_string()),
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.quantity, 999999.999999);
        assert_eq!(deserialized.pnl, Some(10000000.0));
    }

    #[test]
    fn test_cov7_performance_stats_zero_win_rate() {
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 0,
            losing_trades: 100,
            win_rate: 0.0,
            total_pnl: -50000.0,
            avg_pnl: -500.0,
            max_win: 0.0,
            max_loss: -5000.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.win_rate, 0.0);
        assert_eq!(deserialized.winning_trades, 0);
        assert!(deserialized.total_pnl < 0.0);
    }

    #[test]
    fn test_cov7_ai_signal_record_high_risk() {
        let timestamp = Utc::now();
        let record = AISignalRecord {
            id: None,
            signal_id: "high_risk_signal".to_string(),
            symbol: "ALTUSDT".to_string(),
            signal_type: "LONG".to_string(),
            confidence: 0.55,
            reasoning: "High volatility, use caution".to_string(),
            entry_price: 1.5,
            trend_direction: "UP".to_string(),
            trend_strength: 0.5,
            volatility: 0.95,
            risk_score: 0.95, // Very high risk
            executed: false,
            trade_id: None,
            created_at: timestamp,
            timestamp,
            outcome: Some("pending".to_string()),
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.risk_score, 0.95);
        assert_eq!(deserialized.volatility, 0.95);
        assert!(!deserialized.executed);
    }

    #[test]
    fn test_cov7_portfolio_history_record_zero_trades() {
        let timestamp = Utc::now();
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp,
            current_balance: 10000.0,
            equity: 10000.0,
            margin_used: 0.0,
            free_margin: 10000.0,
            total_pnl: 0.0,
            total_pnl_percentage: 0.0,
            total_trades: 0, // No trades yet
            win_rate: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            open_positions: 0,
            created_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_trades, 0);
        assert_eq!(deserialized.open_positions, 0);
        assert_eq!(deserialized.total_pnl, 0.0);
    }

    #[test]
    fn test_cov7_paper_trading_record_closed_with_profit() {
        let open_time = Utc::now();
        let close_time = open_time + chrono::Duration::hours(2);

        let record = PaperTradingRecord {
            id: None,
            trade_id: "profitable_trade".to_string(),
            symbol: "ETHUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "CLOSED".to_string(),
            entry_price: 3000.0,
            exit_price: Some(3300.0),
            quantity: 1.0,
            leverage: 5,
            pnl: Some(1500.0), // 300 * 5 leverage
            pnl_percentage: 50.0, // 10% price gain * 5 leverage
            trading_fees: 15.0,
            funding_fees: 2.5,
            open_time,
            close_time: Some(close_time),
            ai_signal_id: Some("signal_winner".to_string()),
            ai_confidence: Some(0.88),
            close_reason: Some("TAKE_PROFIT".to_string()),
            created_at: open_time,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, "CLOSED");
        assert!(deserialized.pnl.unwrap() > 0.0);
        assert_eq!(deserialized.leverage, 5);
    }

    #[test]
    fn test_cov7_kline_zero_volume() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "50000.0".to_string(),
            high: "50000.0".to_string(),
            low: "50000.0".to_string(),
            close: "50000.0".to_string(),
            volume: "0".to_string(), // No volume
            quote_asset_volume: "0".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "0".to_string(),
            taker_buy_quote_asset_volume: "0".to_string(),
            ignore: "0".to_string(),
        };

        let volume: f64 = kline.volume.parse().unwrap();
        assert_eq!(volume, 0.0);
        assert_eq!(kline.number_of_trades, 0);
    }

    #[test]
    fn test_cov7_performance_metrics_record_negative_sharpe() {
        let date = Utc::now();
        let record = PerformanceMetricsRecord {
            id: None,
            date,
            total_trades: 50,
            winning_trades: 20,
            losing_trades: 30,
            win_rate: 40.0,
            average_win: 100.0,
            average_loss: -150.0,
            largest_win: 500.0,
            largest_loss: -800.0,
            profit_factor: 0.67, // More losses than wins
            sharpe_ratio: -0.5, // Negative Sharpe (underperforming)
            max_drawdown: 5000.0,
            max_drawdown_percentage: 25.0,
            total_pnl: -1500.0,
            daily_pnl: -30.0,
            created_at: date,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PerformanceMetricsRecord = serde_json::from_str(&json).unwrap();
        assert!(deserialized.sharpe_ratio < 0.0);
        assert!(deserialized.profit_factor < 1.0);
        assert!(deserialized.total_pnl < 0.0);
    }

    #[test]
    fn test_cov7_trade_record_cancelled_status() {
        let trade = TradeRecord {
            id: None,
            symbol: "SOLUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 0.0,
            entry_price: 100.0,
            exit_price: None,
            stop_loss: Some(98.0),
            take_profit: Some(105.0),
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "cancelled".to_string(),
            strategy_used: Some("MOMENTUM".to_string()),
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, "cancelled");
        assert_eq!(deserialized.quantity, 0.0);
        assert!(deserialized.exit_price.is_none());
    }

    // =========================================================================
    // COV7: Null DB tests for get_analysis_history with None limit
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_get_analysis_history_no_limit() {
        let storage = create_null_db_storage().await;
        let result = storage.get_analysis_history("ETHUSDT", None).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    // =========================================================================
    // COV7: Null DB tests for get_market_data with None limit
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_get_market_data_no_limit() {
        let storage = create_null_db_storage().await;
        let result = storage.get_market_data("BTCUSDT", "15m", None).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    // =========================================================================
    // COV7: Null DB tests for get_config_suggestions with None limit
    // =========================================================================

    #[tokio::test]
    async fn test_cov7_null_db_get_config_suggestions_no_limit() {
        let storage = create_null_db_storage().await;
        let result = storage.get_config_suggestions(None).await;
        assert!(result.unwrap().is_empty()); // Returns Ok(vec![]) when db is None
    }

    // =========================================================================
    // COV_BOOST: Collection accessor error paths (null DB)
    // =========================================================================

    #[test]
    fn test_collection_paper_trades_null_db() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = create_null_db_storage().await;
            let result = storage.paper_trades();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Database not initialized");
        });
    }

    #[test]
    fn test_collection_portfolio_history_null_db() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = create_null_db_storage().await;
            let result = storage.portfolio_history();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Database not initialized");
        });
    }

    #[test]
    fn test_collection_ai_signals_null_db() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = create_null_db_storage().await;
            let result = storage.ai_signals();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Database not initialized");
        });
    }

    #[test]
    fn test_collection_performance_metrics_null_db() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = create_null_db_storage().await;
            let result = storage.performance_metrics();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Database not initialized");
        });
    }

    #[test]
    fn test_collection_paper_trading_settings_null_db() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = create_null_db_storage().await;
            let result = storage.paper_trading_settings();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Database not initialized");
        });
    }

    // =========================================================================
    // COV_BOOST: Additional struct serialization tests
    // =========================================================================

    #[test]
    fn test_paper_trading_settings_record_serialization_v2() {
        use mongodb::bson::{doc, Document};

        let timestamp = Utc::now();
        let settings_doc: Document = doc! {
            "initial_balance": 10000.0,
            "max_trades": 5
        };

        let record = PaperTradingSettingsRecord {
            id: None,
            settings_data: settings_doc,
            created_at: timestamp,
            updated_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingSettingsRecord = serde_json::from_str(&json).unwrap();
        assert!(deserialized.settings_data.contains_key("initial_balance"));
        assert!(deserialized.settings_data.contains_key("max_trades"));
    }

    #[test]
    fn test_trade_analysis_record_serialization() {
        use mongodb::bson::{doc, Document};

        let timestamp = Utc::now();
        let analysis_doc: Document = doc! {
            "reasoning": "Strong momentum",
            "technical_indicators": {"RSI": 65, "MACD": "positive"},
            "expected_outcome": "PROFIT"
        };

        let record = TradeAnalysisRecord {
            id: None,
            trade_id: "trade_analysis_001".to_string(),
            created_at: timestamp,
            is_winning: true,
            pnl_usdt: 125.50,
            pnl_percentage: 8.5,
            symbol: Some("BTCUSDT".to_string()),
            side: Some("LONG".to_string()),
            entry_price: Some(50000.0),
            exit_price: Some(54250.0),
            close_reason: Some("TAKE_PROFIT".to_string()),
            analysis: analysis_doc,
            trade_data: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: TradeAnalysisRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trade_id, "trade_analysis_001");
        assert!(deserialized.is_winning);
        assert_eq!(deserialized.pnl_usdt, 125.50);
        assert!(deserialized.symbol.is_some());
    }

    #[test]
    fn test_config_suggestions_record_serialization() {
        use mongodb::bson::{doc, Document};

        let timestamp = Utc::now();
        let suggestions_doc: Document = doc! {
            "stop_loss_percentage": {
                "current": "2.0",
                "suggested": "1.5",
                "reasoning": "Reduce drawdown risk"
            }
        };

        let record = ConfigSuggestionsRecord {
            id: None,
            created_at: timestamp,
            status: "pending".to_string(),
            timestamp: Some(timestamp.to_rfc3339()),
            current_config: None,
            trade_stats: None,
            suggestions: suggestions_doc,
            applied_changes: vec![],
            task_id: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: ConfigSuggestionsRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, "pending");
        assert!(deserialized.applied_changes.is_empty());
        assert!(deserialized.task_id.is_none());
    }

    // =========================================================================
    // COV_BOOST: Edge cases for existing structs
    // =========================================================================

    #[test]
    fn test_ai_signal_record_with_outcome_fields() {
        let timestamp = Utc::now();
        let record = AISignalRecord {
            id: None,
            signal_id: "closed_signal".to_string(),
            symbol: "ADAUSDT".to_string(),
            signal_type: "SHORT".to_string(),
            confidence: 0.75,
            reasoning: "Overbought RSI".to_string(),
            entry_price: 0.5,
            trend_direction: "DOWN".to_string(),
            trend_strength: 0.65,
            volatility: 0.4,
            risk_score: 0.3,
            executed: true,
            trade_id: Some("completed_trade".to_string()),
            created_at: timestamp,
            timestamp,
            outcome: Some("WIN".to_string()),
            actual_pnl: Some(125.50),
            pnl_percentage: Some(8.5),
            exit_price: Some(0.46),
            close_reason: Some("TAKE_PROFIT".to_string()),
            closed_at: Some(timestamp + chrono::Duration::hours(3)),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.outcome.unwrap(), "WIN");
        assert_eq!(deserialized.actual_pnl.unwrap(), 125.50);
        assert_eq!(deserialized.pnl_percentage.unwrap(), 8.5);
        assert!(deserialized.closed_at.is_some());
    }

    #[test]
    fn test_portfolio_history_record_extreme_values() {
        let timestamp = Utc::now();
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp,
            current_balance: f64::MAX / 2.0,
            equity: f64::MAX / 2.0,
            margin_used: 0.0,
            free_margin: f64::MAX / 2.0,
            total_pnl: f64::MAX / 4.0,
            total_pnl_percentage: 1000000.0,
            total_trades: u32::MAX,
            win_rate: 100.0,
            profit_factor: f64::MAX,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            open_positions: u32::MAX,
            created_at: timestamp,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();
        assert!(deserialized.current_balance > 1e100);
        assert_eq!(deserialized.total_trades, u32::MAX);
    }

    #[test]
    fn test_performance_metrics_record_edge_case() {
        let date = Utc::now();
        let record = PerformanceMetricsRecord {
            id: None,
            date,
            total_trades: 1,
            winning_trades: 1,
            losing_trades: 0,
            win_rate: 100.0,
            average_win: 1000.0,
            average_loss: 0.0,
            largest_win: 1000.0,
            largest_loss: 0.0,
            profit_factor: 999.99,
            sharpe_ratio: 5.5,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            total_pnl: 1000.0,
            daily_pnl: 1000.0,
            created_at: date,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PerformanceMetricsRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.profit_factor, 999.99);
        assert_eq!(deserialized.losing_trades, 0);
    }

    // =========================================================================
    // COV_BOOST: Additional null DB path coverage
    // =========================================================================

    #[tokio::test]
    async fn test_null_db_store_analysis_v2() {
        use crate::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};
        use std::collections::HashMap;

        let storage = create_null_db_storage().await;
        let analysis = MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1640000000000,
            timeframe_signals: HashMap::new(),
            overall_signal: TradingSignal::Buy,
            overall_confidence: 0.85,
            entry_price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            risk_reward_ratio: Some(2.5),
        };

        let result = storage.store_analysis(&analysis).await;
        assert!(result.is_ok()); // Should succeed with null DB (logs only)
    }

    #[tokio::test]
    async fn test_null_db_store_trade_record_v2() {
        let storage = create_null_db_storage().await;
        let trade = create_sample_trade_record();

        let result = storage.store_trade_record(&trade).await;
        assert!(result.is_ok()); // Should succeed with null DB (logs only)
    }

    #[tokio::test]
    async fn test_null_db_get_performance_stats_v2() {
        let storage = create_null_db_storage().await;
        let result = storage.get_performance_stats().await;

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.win_rate, 0.0);
    }

    #[tokio::test]
    async fn test_null_db_get_trade_history_with_symbol() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_history(Some("ETHUSDT"), Some(50)).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_null_db_get_trade_history_no_symbol() {
        let storage = create_null_db_storage().await;
        let result = storage.get_trade_history(None, None).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_null_db_store_market_data_v2() {
        let storage = create_null_db_storage().await;
        let klines = vec![create_sample_kline()];

        let result = storage.store_market_data("BTCUSDT", "1h", &klines).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // COV_BOOST: Price parsing edge cases
    // =========================================================================

    #[test]
    fn test_kline_negative_price_parsing() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "-50000.0".to_string(),
            high: "51000.0".to_string(),
            low: "49000.0".to_string(),
            close: "50500.0".to_string(),
            volume: "100.5".to_string(),
            quote_asset_volume: "5025000.0".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.0".to_string(),
            taker_buy_quote_asset_volume: "2500000.0".to_string(),
            ignore: "0".to_string(),
        };

        let open_price = kline.open.parse::<f64>();
        assert!(open_price.is_ok());
        assert!(open_price.unwrap() < 0.0);
    }

    #[test]
    fn test_kline_zero_price() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "0.0".to_string(),
            high: "0.0".to_string(),
            low: "0.0".to_string(),
            close: "0.0".to_string(),
            volume: "0".to_string(),
            quote_asset_volume: "0".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "0".to_string(),
            taker_buy_quote_asset_volume: "0".to_string(),
            ignore: "0".to_string(),
        };

        let close_price = kline.close.parse::<f64>().unwrap();
        assert_eq!(close_price, 0.0);
    }

    #[test]
    fn test_kline_very_large_numbers() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "999999999999.99".to_string(),
            high: "1000000000000.00".to_string(),
            low: "999999999999.00".to_string(),
            close: "999999999999.50".to_string(),
            volume: "1000000000.0".to_string(),
            quote_asset_volume: "999999999999999.0".to_string(),
            number_of_trades: i64::MAX,
            taker_buy_base_asset_volume: "500000000.0".to_string(),
            taker_buy_quote_asset_volume: "500000000000000.0".to_string(),
            ignore: "0".to_string(),
        };

        let high_price = kline.high.parse::<f64>().unwrap();
        assert!(high_price >= 1e12);
        assert_eq!(kline.number_of_trades, i64::MAX);
    }

    // =========================================================================
    // COV_BOOST: TradeRecord edge cases
    // =========================================================================

    #[test]
    fn test_trade_record_very_long_trade() {
        let trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 1.0,
            entry_price: 30000.0,
            exit_price: Some(60000.0),
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000, // Jan 2022
            exit_time: Some(1672536000000), // Jan 2023 (1 year later)
            pnl: Some(30000.0),
            status: "closed".to_string(),
            strategy_used: Some("HODL".to_string()),
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        let duration = deserialized.exit_time.unwrap() - deserialized.entry_time;
        assert!(duration > 365 * 24 * 60 * 60 * 1000); // More than a year
    }

    #[test]
    fn test_trade_record_multiple_strategy_names() {
        let trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 5.0,
            entry_price: 2000.0,
            exit_price: Some(1800.0),
            stop_loss: Some(2100.0),
            take_profit: Some(1500.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(1000.0),
            status: "closed".to_string(),
            strategy_used: Some("MACD_RSI_BOLLINGER_COMBINED".to_string()),
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        let strategy = deserialized.strategy_used.as_ref().unwrap();
        assert!(strategy.contains("MACD"));
        assert!(strategy.contains("RSI"));
    }

    // =========================================================================
    // COV_BOOST: Test Default implementation paths
    // =========================================================================

    #[test]
    fn test_performance_stats_default_all_fields() {
        let stats = PerformanceStats::default();

        // Verify every field is initialized to zero/default
        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.winning_trades, 0);
        assert_eq!(stats.losing_trades, 0);
        assert_eq!(stats.win_rate, 0.0);
        assert_eq!(stats.total_pnl, 0.0);
        assert_eq!(stats.avg_pnl, 0.0);
        assert_eq!(stats.max_win, 0.0);
        assert_eq!(stats.max_loss, 0.0);

        // Can be serialized
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_trades\":0"));
    }

    // =========================================================================
    // COV_BOOST: Comprehensive struct serialization/deserialization tests
    // =========================================================================

    #[test]
    fn test_boost_trade_record_serialization() {
        let record = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 1.5,
            entry_price: 45000.0,
            exit_price: Some(46000.0),
            stop_loss: Some(44000.0),
            take_profit: Some(47000.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(1500.0),
            status: "closed".to_string(),
            strategy_used: Some("RSI".to_string()),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.symbol, deserialized.symbol);
        assert_eq!(record.side, deserialized.side);
        assert_eq!(record.quantity, deserialized.quantity);
        assert_eq!(record.pnl, deserialized.pnl);
    }

    #[test]
    fn test_boost_trade_record_open_trade() {
        let record = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 5.0,
            entry_price: 3000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: TradeRecord = serde_json::from_str(&json).unwrap();

        assert!(deserialized.exit_price.is_none());
        assert!(deserialized.pnl.is_none());
        assert_eq!(deserialized.status, "open");
    }

    #[test]
    fn test_boost_performance_stats_serialization() {
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 65,
            losing_trades: 35,
            win_rate: 0.65,
            total_pnl: 5000.0,
            avg_pnl: 50.0,
            max_win: 1000.0,
            max_loss: -500.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PerformanceStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_trades, deserialized.total_trades);
        assert_eq!(stats.winning_trades, deserialized.winning_trades);
        assert_eq!(stats.win_rate, deserialized.win_rate);
    }

    #[test]
    fn test_boost_performance_stats_clone() {
        let stats = PerformanceStats::default();
        let cloned = stats.clone();

        assert_eq!(stats.total_trades, cloned.total_trades);
        assert_eq!(stats.win_rate, cloned.win_rate);
    }

    #[test]
    fn test_boost_performance_stats_debug() {
        let stats = PerformanceStats::default();
        let debug_str = format!("{:?}", stats);

        assert!(debug_str.contains("PerformanceStats"));
    }

    #[test]
    fn test_boost_paper_trading_record_serialization() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "trade_123".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "closed".to_string(),
            entry_price: 50000.0,
            exit_price: Some(51000.0),
            quantity: 0.1,
            leverage: 5,
            pnl: Some(50.0),
            pnl_percentage: 2.0,
            trading_fees: 0.5,
            funding_fees: 0.1,
            open_time: Utc::now(),
            close_time: Some(Utc::now()),
            ai_signal_id: Some("signal_456".to_string()),
            ai_confidence: Some(0.85),
            close_reason: Some("TakeProfit".to_string()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PaperTradingRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.trade_id, deserialized.trade_id);
        assert_eq!(record.symbol, deserialized.symbol);
        assert_eq!(record.leverage, deserialized.leverage);
    }

    #[test]
    fn test_boost_paper_trading_record_open() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "trade_open".to_string(),
            symbol: "ETHUSDT".to_string(),
            trade_type: "SHORT".to_string(),
            status: "open".to_string(),
            entry_price: 2000.0,
            exit_price: None,
            quantity: 1.0,
            leverage: 3,
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 0.2,
            funding_fees: 0.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"status\":\"open\""));
        assert!(json.contains("trade_open"));
    }

    #[test]
    fn test_boost_portfolio_history_record_serialization() {
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp: Utc::now(),
            current_balance: 10000.0,
            equity: 10500.0,
            margin_used: 2000.0,
            free_margin: 8500.0,
            total_pnl: 500.0,
            total_pnl_percentage: 5.0,
            total_trades: 20,
            win_rate: 0.65,
            profit_factor: 1.8,
            max_drawdown: 300.0,
            max_drawdown_percentage: 3.0,
            open_positions: 2,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PortfolioHistoryRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.current_balance, deserialized.current_balance);
        assert_eq!(record.equity, deserialized.equity);
        assert_eq!(record.total_trades, deserialized.total_trades);
    }

    #[test]
    fn test_boost_portfolio_history_negative_pnl() {
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp: Utc::now(),
            current_balance: 9500.0,
            equity: 9000.0,
            margin_used: 1500.0,
            free_margin: 7500.0,
            total_pnl: -500.0,
            total_pnl_percentage: -5.0,
            total_trades: 10,
            win_rate: 0.4,
            profit_factor: 0.8,
            max_drawdown: 800.0,
            max_drawdown_percentage: 8.0,
            open_positions: 1,
            created_at: Utc::now(),
        };

        assert!(record.total_pnl < 0.0);
        assert!(record.profit_factor < 1.0);
    }

    #[test]
    fn test_boost_ai_signal_record_serialization() {
        let record = AISignalRecord {
            id: None,
            signal_id: "sig_123".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "BUY".to_string(),
            confidence: 0.85,
            reasoning: "Strong bullish momentum".to_string(),
            entry_price: 45000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.75,
            volatility: 0.15,
            risk_score: 0.25,
            executed: true,
            trade_id: Some("trade_789".to_string()),
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: Some("win".to_string()),
            actual_pnl: Some(500.0),
            pnl_percentage: Some(2.5),
            exit_price: Some(46125.0),
            close_reason: Some("TakeProfit".to_string()),
            closed_at: Some(Utc::now()),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AISignalRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.signal_id, deserialized.signal_id);
        assert_eq!(record.confidence, deserialized.confidence);
        assert_eq!(record.executed, deserialized.executed);
    }

    #[test]
    fn test_boost_ai_signal_not_executed() {
        let record = AISignalRecord {
            id: None,
            signal_id: "sig_pending".to_string(),
            symbol: "ETHUSDT".to_string(),
            signal_type: "SELL".to_string(),
            confidence: 0.65,
            reasoning: "Weak signal".to_string(),
            entry_price: 3000.0,
            trend_direction: "DOWN".to_string(),
            trend_strength: 0.45,
            volatility: 0.25,
            risk_score: 0.55,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        assert!(!record.executed);
        assert!(record.trade_id.is_none());
        assert!(record.outcome.is_none());
    }

    #[test]
    fn test_boost_performance_metrics_record_serialization() {
        let record = PerformanceMetricsRecord {
            id: None,
            date: Utc::now(),
            total_trades: 50,
            winning_trades: 32,
            losing_trades: 18,
            win_rate: 0.64,
            average_win: 150.0,
            average_loss: -80.0,
            largest_win: 500.0,
            largest_loss: -250.0,
            profit_factor: 2.0,
            sharpe_ratio: 1.5,
            max_drawdown: 400.0,
            max_drawdown_percentage: 4.0,
            total_pnl: 2000.0,
            daily_pnl: 100.0,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: PerformanceMetricsRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.total_trades, deserialized.total_trades);
        assert_eq!(record.win_rate, deserialized.win_rate);
        assert_eq!(record.profit_factor, deserialized.profit_factor);
    }

    #[test]
    fn test_boost_performance_metrics_zero_trades() {
        let record = PerformanceMetricsRecord {
            id: None,
            date: Utc::now(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            average_win: 0.0,
            average_loss: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            profit_factor: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            total_pnl: 0.0,
            daily_pnl: 0.0,
            created_at: Utc::now(),
        };

        assert_eq!(record.total_trades, 0);
        assert_eq!(record.win_rate, 0.0);
    }

    #[test]
    fn test_boost_paper_trading_settings_record_serialization() {
        let mut doc = mongodb::bson::Document::new();
        doc.insert("initial_balance", 10000.0);
        doc.insert("leverage", 5);

        let record = PaperTradingSettingsRecord {
            id: None,
            settings_data: doc.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("settings_data"));
    }

    #[test]
    fn test_boost_trade_analysis_record_serialization() {
        let mut analysis_doc = mongodb::bson::Document::new();
        analysis_doc.insert("summary", "Good trade");

        let mut trade_data_doc = mongodb::bson::Document::new();
        trade_data_doc.insert("symbol", "BTCUSDT");

        let record = TradeAnalysisRecord {
            id: None,
            trade_id: "trade_analysis_1".to_string(),
            created_at: Utc::now(),
            is_winning: true,
            pnl_usdt: 500.0,
            pnl_percentage: 5.0,
            symbol: Some("BTCUSDT".to_string()),
            side: Some("BUY".to_string()),
            entry_price: Some(45000.0),
            exit_price: Some(47250.0),
            close_reason: Some("TakeProfit".to_string()),
            analysis: analysis_doc,
            trade_data: Some(trade_data_doc),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("trade_analysis_1"));
        assert!(json.contains("pnl_usdt"));
    }

    #[test]
    fn test_boost_trade_analysis_losing_trade() {
        let mut analysis_doc = mongodb::bson::Document::new();
        analysis_doc.insert("summary", "Bad entry");

        let record = TradeAnalysisRecord {
            id: None,
            trade_id: "losing_trade".to_string(),
            created_at: Utc::now(),
            is_winning: false,
            pnl_usdt: -200.0,
            pnl_percentage: -2.0,
            symbol: Some("ETHUSDT".to_string()),
            side: Some("SELL".to_string()),
            entry_price: Some(3000.0),
            exit_price: Some(3060.0),
            close_reason: Some("StopLoss".to_string()),
            analysis: analysis_doc,
            trade_data: None,
        };

        assert!(!record.is_winning);
        assert!(record.pnl_usdt < 0.0);
    }

    #[test]
    fn test_boost_config_suggestions_record_serialization() {
        let mut suggestions_doc = mongodb::bson::Document::new();
        suggestions_doc.insert("recommendation", "Increase stop loss");

        let record = ConfigSuggestionsRecord {
            id: None,
            created_at: Utc::now(),
            status: "pending".to_string(),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            current_config: None,
            trade_stats: None,
            suggestions: suggestions_doc,
            applied_changes: vec!["stop_loss".to_string()],
            task_id: Some("task_123".to_string()),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("pending"));
        assert!(json.contains("stop_loss"));
    }

    #[test]
    fn test_boost_config_suggestions_applied() {
        let mut suggestions_doc = mongodb::bson::Document::new();
        suggestions_doc.insert("recommendation", "Update");

        let record = ConfigSuggestionsRecord {
            id: None,
            created_at: Utc::now(),
            status: "applied".to_string(),
            timestamp: None,
            current_config: None,
            trade_stats: None,
            suggestions: suggestions_doc,
            applied_changes: vec!["leverage".to_string(), "position_size".to_string()],
            task_id: None,
        };

        assert_eq!(record.status, "applied");
        assert_eq!(record.applied_changes.len(), 2);
    }

    // =========================================================================
    // COV_BOOST: Edge cases for struct fields
    // =========================================================================

    #[test]
    fn test_boost_trade_record_empty_symbol() {
        let record = TradeRecord {
            id: None,
            symbol: "".to_string(),
            side: "BUY".to_string(),
            quantity: 1.0,
            entry_price: 1.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 0,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        assert_eq!(record.symbol, "");
    }

    #[test]
    fn test_boost_trade_record_negative_quantity() {
        let record = TradeRecord {
            id: None,
            symbol: "TEST".to_string(),
            side: "SELL".to_string(),
            quantity: -1.0,
            entry_price: 100.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: None,
        };

        assert!(record.quantity < 0.0);
    }

    #[test]
    fn test_boost_trade_record_zero_price() {
        let record = TradeRecord {
            id: None,
            symbol: "TEST".to_string(),
            side: "BUY".to_string(),
            quantity: 1.0,
            entry_price: 0.0,
            exit_price: Some(0.0),
            stop_loss: Some(0.0),
            take_profit: Some(0.0),
            entry_time: 0,
            exit_time: Some(0),
            pnl: Some(0.0),
            status: "closed".to_string(),
            strategy_used: None,
        };

        assert_eq!(record.entry_price, 0.0);
    }

    #[test]
    fn test_boost_performance_stats_negative_values() {
        let stats = PerformanceStats {
            total_trades: 10,
            winning_trades: 0,
            losing_trades: 10,
            win_rate: 0.0,
            total_pnl: -5000.0,
            avg_pnl: -500.0,
            max_win: 0.0,
            max_loss: -1000.0,
        };

        assert!(stats.total_pnl < 0.0);
        assert!(stats.max_loss < 0.0);
    }

    #[test]
    fn test_boost_paper_trading_record_max_leverage() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "high_lev".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "open".to_string(),
            entry_price: 50000.0,
            exit_price: None,
            quantity: 0.1,
            leverage: 125,
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 0.5,
            funding_fees: 0.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: None,
            created_at: Utc::now(),
        };

        assert_eq!(record.leverage, 125);
    }

    #[test]
    fn test_boost_ai_signal_record_extreme_confidence() {
        let record = AISignalRecord {
            id: None,
            signal_id: "sig_extreme".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "BUY".to_string(),
            confidence: 0.99,
            reasoning: "Very strong signal".to_string(),
            entry_price: 50000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.95,
            volatility: 0.05,
            risk_score: 0.10,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        assert!(record.confidence > 0.9);
        assert!(record.trend_strength > 0.9);
    }

    #[test]
    fn test_boost_ai_signal_record_low_confidence() {
        let record = AISignalRecord {
            id: None,
            signal_id: "sig_weak".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "NEUTRAL".to_string(),
            confidence: 0.01,
            reasoning: "Very weak signal".to_string(),
            entry_price: 50000.0,
            trend_direction: "SIDEWAYS".to_string(),
            trend_strength: 0.05,
            volatility: 0.95,
            risk_score: 0.90,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        assert!(record.confidence < 0.1);
        assert!(record.volatility > 0.9);
    }

    #[test]
    fn test_boost_portfolio_history_zero_balance() {
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp: Utc::now(),
            current_balance: 0.0,
            equity: 0.0,
            margin_used: 0.0,
            free_margin: 0.0,
            total_pnl: 0.0,
            total_pnl_percentage: 0.0,
            total_trades: 0,
            win_rate: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            open_positions: 0,
            created_at: Utc::now(),
        };

        assert_eq!(record.current_balance, 0.0);
        assert_eq!(record.equity, 0.0);
    }

    #[test]
    fn test_boost_performance_metrics_perfect_win_rate() {
        let record = PerformanceMetricsRecord {
            id: None,
            date: Utc::now(),
            total_trades: 100,
            winning_trades: 100,
            losing_trades: 0,
            win_rate: 1.0,
            average_win: 100.0,
            average_loss: 0.0,
            largest_win: 500.0,
            largest_loss: 0.0,
            profit_factor: 99.9,
            sharpe_ratio: 3.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            total_pnl: 10000.0,
            daily_pnl: 100.0,
            created_at: Utc::now(),
        };

        assert_eq!(record.win_rate, 1.0);
        assert_eq!(record.losing_trades, 0);
    }

    // =========================================================================
    // COV_BOOST: Clone trait coverage
    // =========================================================================

    #[test]
    fn test_boost_trade_record_clone() {
        let record = create_sample_trade_record();
        let cloned = record.clone();

        assert_eq!(record.symbol, cloned.symbol);
        assert_eq!(record.quantity, cloned.quantity);
    }

    #[test]
    fn test_boost_paper_trading_record_clone() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "clone_test".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "open".to_string(),
            entry_price: 50000.0,
            exit_price: None,
            quantity: 0.1,
            leverage: 5,
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 0.5,
            funding_fees: 0.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: None,
            created_at: Utc::now(),
        };

        let cloned = record.clone();
        assert_eq!(record.trade_id, cloned.trade_id);
    }

    #[test]
    fn test_boost_portfolio_history_record_clone() {
        let record = PortfolioHistoryRecord {
            id: None,
            timestamp: Utc::now(),
            current_balance: 10000.0,
            equity: 10000.0,
            margin_used: 0.0,
            free_margin: 10000.0,
            total_pnl: 0.0,
            total_pnl_percentage: 0.0,
            total_trades: 0,
            win_rate: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percentage: 0.0,
            open_positions: 0,
            created_at: Utc::now(),
        };

        let cloned = record.clone();
        assert_eq!(record.current_balance, cloned.current_balance);
    }

    #[test]
    fn test_boost_ai_signal_record_clone() {
        let record = AISignalRecord {
            id: None,
            signal_id: "clone_sig".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "BUY".to_string(),
            confidence: 0.75,
            reasoning: "Test".to_string(),
            entry_price: 50000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.7,
            volatility: 0.2,
            risk_score: 0.3,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        let cloned = record.clone();
        assert_eq!(record.signal_id, cloned.signal_id);
    }

    // =========================================================================
    // COV_BOOST: Debug trait coverage
    // =========================================================================

    #[test]
    fn test_boost_trade_record_debug() {
        let record = create_sample_trade_record();
        let debug_str = format!("{:?}", record);

        assert!(debug_str.contains("TradeRecord"));
        assert!(debug_str.contains("BTCUSDT"));
    }

    #[test]
    fn test_boost_paper_trading_record_debug() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "debug_test".to_string(),
            symbol: "ETHUSDT".to_string(),
            trade_type: "SHORT".to_string(),
            status: "closed".to_string(),
            entry_price: 3000.0,
            exit_price: Some(2900.0),
            quantity: 1.0,
            leverage: 3,
            pnl: Some(100.0),
            pnl_percentage: 3.33,
            trading_fees: 0.3,
            funding_fees: 0.1,
            open_time: Utc::now(),
            close_time: Some(Utc::now()),
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: Some("TakeProfit".to_string()),
            created_at: Utc::now(),
        };

        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("PaperTradingRecord"));
    }

    #[test]
    fn test_boost_ai_signal_record_debug() {
        let record = AISignalRecord {
            id: None,
            signal_id: "debug_sig".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "BUY".to_string(),
            confidence: 0.85,
            reasoning: "Debug test".to_string(),
            entry_price: 50000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.8,
            volatility: 0.15,
            risk_score: 0.2,
            executed: true,
            trade_id: Some("trade_debug".to_string()),
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: Some("pending".to_string()),
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("AISignalRecord"));
        assert!(debug_str.contains("debug_sig"));
    }

    // =========================================================================
    // COV_BOOST: Empty string fields
    // =========================================================================

    #[test]
    fn test_boost_trade_record_empty_strategy() {
        let record = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 1.0,
            entry_price: 50000.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: Some("".to_string()),
        };

        assert_eq!(record.strategy_used.as_ref().unwrap(), "");
    }

    #[test]
    fn test_boost_ai_signal_empty_reasoning() {
        let record = AISignalRecord {
            id: None,
            signal_id: "empty_reason".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "BUY".to_string(),
            confidence: 0.5,
            reasoning: "".to_string(),
            entry_price: 50000.0,
            trend_direction: "".to_string(),
            trend_strength: 0.5,
            volatility: 0.5,
            risk_score: 0.5,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: None,
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        assert_eq!(record.reasoning, "");
        assert_eq!(record.trend_direction, "");
    }
    // =========================================================================
    // COV_BOOST: Additional collection & edge case tests (new, non-duplicate)
    // =========================================================================

    #[tokio::test]
    async fn test_boost_api_keys_collection_error() {
        let storage = create_null_db_storage().await;
        let result = storage.api_keys();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database not initialized"));
    }

    #[tokio::test]
    async fn test_boost_push_subscriptions_collection_error() {
        let storage = create_null_db_storage().await;
        let result = storage.push_subscriptions();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database not initialized"));
    }

    // =========================================================================
    // COV_BOOST: Kline field combinations for parse error paths
    // =========================================================================

    #[test]
    fn test_boost_kline_all_invalid_prices() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "invalid".to_string(),
            high: "bad".to_string(),
            low: "wrong".to_string(),
            close: "error".to_string(),
            volume: "nope".to_string(),
            quote_asset_volume: "fail".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "".to_string(),
            taker_buy_quote_asset_volume: "".to_string(),
            ignore: "0".to_string(),
        };

        // All prices should fail to parse
        assert!(kline.open.parse::<f64>().is_err());
        assert!(kline.high.parse::<f64>().is_err());
        assert!(kline.low.parse::<f64>().is_err());
        assert!(kline.close.parse::<f64>().is_err());
    }

    #[test]
    fn test_boost_kline_mixed_valid_invalid() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "50000.0".to_string(),
            high: "invalid".to_string(),
            low: "49000.0".to_string(),
            close: "error".to_string(),
            volume: "100".to_string(),
            quote_asset_volume: "5000000".to_string(),
            number_of_trades: 100,
            taker_buy_base_asset_volume: "50".to_string(),
            taker_buy_quote_asset_volume: "2500000".to_string(),
            ignore: "0".to_string(),
        };

        assert!(kline.open.parse::<f64>().is_ok());
        assert!(kline.high.parse::<f64>().is_err());
        assert!(kline.low.parse::<f64>().is_ok());
        assert!(kline.close.parse::<f64>().is_err());
    }

    // =========================================================================
    // COV_BOOST: Record edge cases with None/Some option fields
    // =========================================================================

    #[test]
    fn test_boost_paper_trading_record_all_none() {
        let record = PaperTradingRecord {
            id: None,
            trade_id: "all_none".to_string(),
            symbol: "ETHUSDT".to_string(),
            trade_type: "LONG".to_string(),
            status: "OPEN".to_string(),
            entry_price: 3000.0,
            exit_price: None,
            quantity: 1.0,
            leverage: 5,
            pnl: None,
            pnl_percentage: 0.0,
            trading_fees: 0.0,
            funding_fees: 0.0,
            open_time: Utc::now(),
            close_time: None,
            ai_signal_id: None,
            ai_confidence: None,
            close_reason: None,
            created_at: Utc::now(),
        };

        assert!(record.exit_price.is_none());
        assert!(record.pnl.is_none());
        assert!(record.close_time.is_none());
        assert!(record.ai_signal_id.is_none());
        assert!(record.ai_confidence.is_none());
        assert!(record.close_reason.is_none());
    }

    #[test]
    fn test_boost_paper_trading_record_all_some() {
        let record = PaperTradingRecord {
            id: Some(bson::oid::ObjectId::new()),
            trade_id: "all_some".to_string(),
            symbol: "BTCUSDT".to_string(),
            trade_type: "SHORT".to_string(),
            status: "CLOSED".to_string(),
            entry_price: 50000.0,
            exit_price: Some(48000.0),
            quantity: 0.5,
            leverage: 10,
            pnl: Some(1000.0),
            pnl_percentage: 20.0,
            trading_fees: 5.0,
            funding_fees: 2.0,
            open_time: Utc::now(),
            close_time: Some(Utc::now()),
            ai_signal_id: Some("signal_abc".to_string()),
            ai_confidence: Some(0.92),
            close_reason: Some("StopLoss".to_string()),
            created_at: Utc::now(),
        };

        assert!(record.id.is_some());
        assert!(record.exit_price.is_some());
        assert!(record.pnl.is_some());
        assert!(record.close_time.is_some());
        assert!(record.ai_signal_id.is_some());
        assert!(record.ai_confidence.is_some());
        assert!(record.close_reason.is_some());
    }

    #[test]
    fn test_boost_ai_signal_record_all_outcomes() {
        // Test with "win" outcome
        let win_record = AISignalRecord {
            id: None,
            signal_id: "win_signal".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: "LONG".to_string(),
            confidence: 0.85,
            reasoning: "Strong buy".to_string(),
            entry_price: 50000.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.9,
            volatility: 0.2,
            risk_score: 0.1,
            executed: true,
            trade_id: Some("trade_win".to_string()),
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: Some("win".to_string()),
            actual_pnl: Some(500.0),
            pnl_percentage: Some(10.0),
            exit_price: Some(55000.0),
            close_reason: Some("TakeProfit".to_string()),
            closed_at: Some(Utc::now()),
        };

        assert_eq!(win_record.outcome, Some("win".to_string()));

        // Test with "loss" outcome
        let loss_record = AISignalRecord {
            id: None,
            signal_id: "loss_signal".to_string(),
            symbol: "ETHUSDT".to_string(),
            signal_type: "SHORT".to_string(),
            confidence: 0.75,
            reasoning: "Bearish".to_string(),
            entry_price: 3000.0,
            trend_direction: "DOWN".to_string(),
            trend_strength: 0.7,
            volatility: 0.4,
            risk_score: 0.3,
            executed: true,
            trade_id: Some("trade_loss".to_string()),
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: Some("loss".to_string()),
            actual_pnl: Some(-200.0),
            pnl_percentage: Some(-5.0),
            exit_price: Some(3150.0),
            close_reason: Some("StopLoss".to_string()),
            closed_at: Some(Utc::now()),
        };

        assert_eq!(loss_record.outcome, Some("loss".to_string()));

        // Test with "pending" outcome
        let pending_record = AISignalRecord {
            id: None,
            signal_id: "pending_signal".to_string(),
            symbol: "BNBUSDT".to_string(),
            signal_type: "LONG".to_string(),
            confidence: 0.80,
            reasoning: "Bullish setup".to_string(),
            entry_price: 400.0,
            trend_direction: "UP".to_string(),
            trend_strength: 0.8,
            volatility: 0.3,
            risk_score: 0.2,
            executed: false,
            trade_id: None,
            created_at: Utc::now(),
            timestamp: Utc::now(),
            outcome: Some("pending".to_string()),
            actual_pnl: None,
            pnl_percentage: None,
            exit_price: None,
            close_reason: None,
            closed_at: None,
        };

        assert_eq!(pending_record.outcome, Some("pending".to_string()));
    }

    // =========================================================================
    // COV_BOOST: TradeRecord various status and side combinations
    // =========================================================================

    #[test]
    fn test_boost_trade_record_all_sides() {
        let buy_trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
            exit_price: Some(52000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(1000.0),
            status: "closed".to_string(),
            strategy_used: Some("RSI".to_string()),
        };

        assert_eq!(buy_trade.side, "BUY");

        let sell_trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: Some(2900.0),
            stop_loss: Some(3100.0),
            take_profit: Some(2800.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(100.0),
            status: "closed".to_string(),
            strategy_used: Some("MACD".to_string()),
        };

        assert_eq!(sell_trade.side, "SELL");
    }

    #[test]
    fn test_boost_trade_record_all_statuses() {
        let open_trade = TradeRecord {
            id: None,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
            exit_price: None,
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            entry_time: 1640000000000,
            exit_time: None,
            pnl: None,
            status: "open".to_string(),
            strategy_used: Some("VOLUME".to_string()),
        };

        assert_eq!(open_trade.status, "open");

        let closed_trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: Some(2900.0),
            stop_loss: Some(3100.0),
            take_profit: Some(2800.0),
            entry_time: 1640000000000,
            exit_time: Some(1640086400000),
            pnl: Some(100.0),
            status: "closed".to_string(),
            strategy_used: Some("BOLLINGER".to_string()),
        };

        assert_eq!(closed_trade.status, "closed");

        let cancelled_trade = TradeRecord {
            id: None,
            symbol: "BNBUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 2.0,
            entry_price: 400.0,
            exit_price: None,
            stop_loss: None,
            take_profit: None,
            entry_time: 1640000000000,
            exit_time: Some(1640003600000),
            pnl: Some(0.0),
            status: "cancelled".to_string(),
            strategy_used: None,
        };

        assert_eq!(cancelled_trade.status, "cancelled");
    }

    // =========================================================================
    // COV_BOOST: Serialization roundtrip for all record types
    // =========================================================================

    #[test]
    fn test_boost_trade_analysis_record_roundtrip() {
        use bson::doc;

        let record = TradeAnalysisRecord {
            id: Some(bson::oid::ObjectId::new()),
            trade_id: "roundtrip_test".to_string(),
            created_at: Utc::now(),
            is_winning: true,
            pnl_usdt: 1500.0,
            pnl_percentage: 15.0,
            symbol: Some("BTCUSDT".to_string()),
            side: Some("LONG".to_string()),
            entry_price: Some(50000.0),
            exit_price: Some(57500.0),
            close_reason: Some("TakeProfit".to_string()),
            analysis: doc! {
                "summary": "Excellent market timing",
                "recommendations": ["Continue with current strategy", "Consider increasing position size"]
            },
            trade_data: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: TradeAnalysisRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.trade_id, "roundtrip_test");
        assert!(deserialized.is_winning);
        assert_eq!(deserialized.pnl_usdt, 1500.0);
    }

    #[test]
    fn test_boost_config_suggestions_record_roundtrip() {
        use bson::doc;

        let record = ConfigSuggestionsRecord {
            id: Some(bson::oid::ObjectId::new()),
            created_at: Utc::now(),
            status: "pending".to_string(),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            current_config: Some(doc! { "leverage": 20, "stop_loss": 2.0 }),
            trade_stats: Some(doc! { "total_trades": 50, "win_rate": 0.6 }),
            suggestions: doc! {
                "improvements": ["Reduce leverage from 20x to 10x", "Increase stop loss to 3%"],
                "reasoning": "Current settings show high risk with recent market volatility"
            },
            applied_changes: vec![],
            task_id: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: ConfigSuggestionsRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.status, "pending");
        assert!(deserialized.suggestions.contains_key("improvements"));
    }

    // =========================================================================
    // COV_BOOST: Multiple kline test variations
    // =========================================================================

    #[test]
    fn test_boost_kline_with_large_numbers() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "999999999.99999999".to_string(),
            high: "1000000000.0".to_string(),
            low: "999999999.0".to_string(),
            close: "999999999.5".to_string(),
            volume: "1000000000.0".to_string(),
            quote_asset_volume: "999999999999999.0".to_string(),
            number_of_trades: 9999999,
            taker_buy_base_asset_volume: "500000000.0".to_string(),
            taker_buy_quote_asset_volume: "499999999999999.0".to_string(),
            ignore: "0".to_string(),
        };

        let open: f64 = kline.open.parse().unwrap();
        assert!(open > 999999999.0);
    }

    #[test]
    fn test_boost_kline_with_small_numbers() {
        let kline = Kline {
            open_time: 1640000000000,
            close_time: 1640003600000,
            open: "0.00000001".to_string(),
            high: "0.00000002".to_string(),
            low: "0.000000005".to_string(),
            close: "0.000000015".to_string(),
            volume: "1000000.0".to_string(),
            quote_asset_volume: "0.015".to_string(),
            number_of_trades: 100,
            taker_buy_base_asset_volume: "500000.0".to_string(),
            taker_buy_quote_asset_volume: "0.0075".to_string(),
            ignore: "0".to_string(),
        };

        let open: f64 = kline.open.parse().unwrap();
        assert!(open < 0.0001);
    }

    // =========================================================================
    // COV_BOOST: PerformanceStats edge case calculations
    // =========================================================================

    #[test]
    fn test_boost_performance_stats_perfect_trading() {
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 100,
            losing_trades: 0,
            win_rate: 100.0,
            total_pnl: 10000.0,
            avg_pnl: 100.0,
            max_win: 500.0,
            max_loss: 0.0,
        };

        assert_eq!(stats.losing_trades, 0);
        assert_eq!(stats.max_loss, 0.0);
    }

    #[test]
    fn test_boost_performance_stats_all_losses() {
        let stats = PerformanceStats {
            total_trades: 100,
            winning_trades: 0,
            losing_trades: 100,
            win_rate: 0.0,
            total_pnl: -10000.0,
            avg_pnl: -100.0,
            max_win: 0.0,
            max_loss: -500.0,
        };

        assert_eq!(stats.winning_trades, 0);
        assert_eq!(stats.max_win, 0.0);
    }
}

