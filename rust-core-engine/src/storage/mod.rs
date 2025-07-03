use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

#[cfg(feature = "database")]
use sqlx::{Row, SqlitePool};

use crate::market_data::analyzer::MultiTimeframeAnalysis;

#[derive(Clone)]
pub struct Storage {
    #[cfg(feature = "database")]
    pool: Option<SqlitePool>,
    
    // In-memory fallback storage
    #[cfg(not(feature = "database"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Storage {
    pub async fn new(config: &crate::config::DatabaseConfig) -> Result<Self> {
        #[cfg(feature = "database")]
        {
            if config.url.starts_with("sqlite:") {
                let pool = SqlitePool::connect(&config.url).await?;
                
                // Run migrations
                sqlx::migrate!("./migrations").run(&pool).await?;
                
                info!("Database connected and migrated successfully");
                
                Ok(Self {
                    pool: Some(pool),
                })
            } else {
                Ok(Self { pool: None })
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
            if let Some(pool) = &self.pool {
                let analysis_json = serde_json::to_string(analysis)?;
                
                sqlx::query!(
                    r#"
                    INSERT OR REPLACE INTO analysis_results 
                    (symbol, timestamp, overall_signal, overall_confidence, analysis_data)
                    VALUES (?, ?, ?, ?, ?)
                    "#,
                    analysis.symbol,
                    analysis.timestamp,
                    format!("{:?}", analysis.overall_signal),
                    analysis.overall_confidence,
                    analysis_json
                )
                .execute(pool)
                .await?;
                
                debug!("Stored analysis result for {} at {}", analysis.symbol, analysis.timestamp);
                return Ok(());
            }
        }
        
        // Fallback: just log the analysis (in-memory storage could be implemented here)
        debug!("Analysis for {}: {:?} (confidence: {:.2})", 
               analysis.symbol, analysis.overall_signal, analysis.overall_confidence);
        Ok(())
    }

    pub async fn get_latest_analysis(&self, symbol: &str) -> Result<Option<MultiTimeframeAnalysis>> {
        #[cfg(feature = "database")]
        {
            if let Some(pool) = &self.pool {
                let row = sqlx::query!(
                    r#"
                    SELECT analysis_data 
                    FROM analysis_results 
                    WHERE symbol = ? 
                    ORDER BY timestamp DESC 
                    LIMIT 1
                    "#,
                    symbol
                )
                .fetch_optional(pool)
                .await?;

                if let Some(row) = row {
                    let analysis: MultiTimeframeAnalysis = serde_json::from_str(&row.analysis_data)?;
                    return Ok(Some(analysis));
                }
            }
        }
        
        Ok(None)
    }

    pub async fn get_analysis_history(
        &self, 
        symbol: &str, 
        limit: Option<i64>
    ) -> Result<Vec<MultiTimeframeAnalysis>> {
        #[cfg(feature = "database")]
        {
            if let Some(pool) = &self.pool {
                let limit = limit.unwrap_or(100);
                
                let rows = sqlx::query!(
                    r#"
                    SELECT analysis_data 
                    FROM analysis_results 
                    WHERE symbol = ? 
                    ORDER BY timestamp DESC 
                    LIMIT ?
                    "#,
                    symbol,
                    limit
                )
                .fetch_all(pool)
                .await?;

                let mut analyses = Vec::new();
                for row in rows {
                    if let Ok(analysis) = serde_json::from_str::<MultiTimeframeAnalysis>(&row.analysis_data) {
                        analyses.push(analysis);
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
            if let Some(pool) = &self.pool {
                sqlx::query!(
                    r#"
                    INSERT INTO trade_records 
                    (symbol, side, quantity, entry_price, exit_price, stop_loss, take_profit, 
                     entry_time, exit_time, pnl, status, strategy_used)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                    trade.symbol,
                    trade.side,
                    trade.quantity,
                    trade.entry_price,
                    trade.exit_price,
                    trade.stop_loss,
                    trade.take_profit,
                    trade.entry_time,
                    trade.exit_time,
                    trade.pnl,
                    trade.status,
                    trade.strategy_used
                )
                .execute(pool)
                .await?;
                
                info!("Stored trade record for {} {} at {}", trade.symbol, trade.side, trade.entry_price);
                return Ok(());
            }
        }
        
        info!("Trade record: {} {} {} @ {} (PnL: {:?})", 
              trade.symbol, trade.side, trade.quantity, trade.entry_price, trade.pnl);
        Ok(())
    }

    pub async fn get_trade_history(&self, symbol: Option<&str>, limit: Option<i64>) -> Result<Vec<TradeRecord>> {
        #[cfg(feature = "database")]
        {
            if let Some(pool) = &self.pool {
                let limit = limit.unwrap_or(100);
                
                let rows = if let Some(symbol) = symbol {
                    sqlx::query!(
                        r#"
                        SELECT * FROM trade_records 
                        WHERE symbol = ? 
                        ORDER BY entry_time DESC 
                        LIMIT ?
                        "#,
                        symbol,
                        limit
                    )
                    .fetch_all(pool)
                    .await?
                } else {
                    sqlx::query!(
                        r#"
                        SELECT * FROM trade_records 
                        ORDER BY entry_time DESC 
                        LIMIT ?
                        "#,
                        limit
                    )
                    .fetch_all(pool)
                    .await?
                };

                let mut trades = Vec::new();
                for row in rows {
                    trades.push(TradeRecord {
                        id: Some(row.id),
                        symbol: row.symbol,
                        side: row.side,
                        quantity: row.quantity,
                        entry_price: row.entry_price,
                        exit_price: row.exit_price,
                        stop_loss: row.stop_loss,
                        take_profit: row.take_profit,
                        entry_time: row.entry_time,
                        exit_time: row.exit_time,
                        pnl: row.pnl,
                        status: row.status,
                        strategy_used: row.strategy_used,
                    });
                }
                
                return Ok(trades);
            }
        }
        
        Ok(Vec::new())
    }

    pub async fn get_performance_stats(&self) -> Result<PerformanceStats> {
        #[cfg(feature = "database")]
        {
            if let Some(pool) = &self.pool {
                let stats_row = sqlx::query!(
                    r#"
                    SELECT 
                        COUNT(*) as total_trades,
                        COUNT(CASE WHEN pnl > 0 THEN 1 END) as winning_trades,
                        COUNT(CASE WHEN pnl < 0 THEN 1 END) as losing_trades,
                        SUM(pnl) as total_pnl,
                        AVG(pnl) as avg_pnl,
                        MAX(pnl) as max_win,
                        MIN(pnl) as max_loss
                    FROM trade_records 
                    WHERE status = 'closed'
                    "#
                )
                .fetch_one(pool)
                .await?;

                let total_trades = stats_row.total_trades as u64;
                let winning_trades = stats_row.winning_trades.unwrap_or(0) as u64;
                let losing_trades = stats_row.losing_trades.unwrap_or(0) as u64;
                
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
                    total_pnl: stats_row.total_pnl.unwrap_or(0.0),
                    avg_pnl: stats_row.avg_pnl.unwrap_or(0.0),
                    max_win: stats_row.max_win.unwrap_or(0.0),
                    max_loss: stats_row.max_loss.unwrap_or(0.0),
                });
            }
        }
        
        // Return default stats if database is not available
        Ok(PerformanceStats::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: Option<i64>,
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