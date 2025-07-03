use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::cache::{CandleData, MarketDataCache};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub symbol: String,
    pub timeframe: String,
    pub candles: Vec<CandleDataForAnalysis>,
    pub analysis_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleDataForAnalysis {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<&CandleData> for CandleDataForAnalysis {
    fn from(candle: &CandleData) -> Self {
        Self {
            timestamp: candle.open_time,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub symbol: String,
    pub timeframe: String,
    pub timestamp: i64,
    pub signal: TradingSignal,
    pub confidence: f64,
    pub indicators: HashMap<String, f64>,
    pub analysis_details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingSignal {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
    #[serde(rename = "HOLD")]
    Hold,
    #[serde(rename = "STRONG_BUY")]
    StrongBuy,
    #[serde(rename = "STRONG_SELL")]
    StrongSell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTimeframeAnalysis {
    pub symbol: String,
    pub timestamp: i64,
    pub timeframe_signals: HashMap<String, AnalysisResponse>,
    pub overall_signal: TradingSignal,
    pub overall_confidence: f64,
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub risk_reward_ratio: Option<f64>,
}

pub struct MarketDataAnalyzer {
    client: Client,
    ai_service_url: String,
    cache: MarketDataCache,
}

impl MarketDataAnalyzer {
    pub fn new(ai_service_url: String, cache: MarketDataCache) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client for analyzer");

        Self {
            client,
            ai_service_url,
            cache,
        }
    }

    pub async fn analyze_single_timeframe(
        &self,
        symbol: &str,
        timeframe: &str,
        analysis_type: &str,
        limit: Option<usize>,
    ) -> Result<AnalysisResponse> {
        let candles = self.cache.get_candles(symbol, timeframe, limit);
        
        if candles.is_empty() {
            return Err(anyhow::anyhow!("No candle data available for {} {}", symbol, timeframe));
        }

        let analysis_candles: Vec<CandleDataForAnalysis> = candles
            .iter()
            .map(CandleDataForAnalysis::from)
            .collect();

        let request = AnalysisRequest {
            symbol: symbol.to_uppercase(),
            timeframe: timeframe.to_string(),
            candles: analysis_candles,
            analysis_type: analysis_type.to_string(),
            parameters: HashMap::new(),
        };

        let url = format!("{}/api/analyze", self.ai_service_url);
        
        debug!("Sending analysis request to {} for {} {}", url, symbol, timeframe);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!("Analysis request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!("AI service request failed: {} - {}", status, error_text));
        }

        let analysis_response: AnalysisResponse = response.json().await?;
        
        info!("Received analysis for {} {}: {:?} (confidence: {:.2})", 
              symbol, timeframe, analysis_response.signal, analysis_response.confidence);
        
        Ok(analysis_response)
    }

    pub async fn analyze_multi_timeframe(
        &self,
        symbol: &str,
        timeframes: &[String],
        analysis_type: &str,
        limit: Option<usize>,
    ) -> Result<MultiTimeframeAnalysis> {
        let mut timeframe_signals = HashMap::new();
        let mut failed_analyses = Vec::new();

        // Analyze each timeframe
        for timeframe in timeframes {
            match self.analyze_single_timeframe(symbol, timeframe, analysis_type, limit).await {
                Ok(analysis) => {
                    timeframe_signals.insert(timeframe.clone(), analysis);
                }
                Err(e) => {
                    warn!("Failed to analyze {} {}: {}", symbol, timeframe, e);
                    failed_analyses.push(timeframe.clone());
                }
            }
        }

        if timeframe_signals.is_empty() {
            return Err(anyhow::anyhow!("All timeframe analyses failed for {}", symbol));
        }

        // Combine signals to determine overall signal
        let (overall_signal, overall_confidence) = self.combine_signals(&timeframe_signals);
        
        // Calculate trade parameters based on multi-timeframe analysis
        let (entry_price, stop_loss, take_profit, risk_reward_ratio) = 
            self.calculate_trade_parameters(symbol, &timeframe_signals).await?;

        let multi_timeframe_analysis = MultiTimeframeAnalysis {
            symbol: symbol.to_uppercase(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            timeframe_signals,
            overall_signal,
            overall_confidence,
            entry_price,
            stop_loss,
            take_profit,
            risk_reward_ratio,
        };

        info!("Multi-timeframe analysis for {}: {:?} (confidence: {:.2})", 
              symbol, multi_timeframe_analysis.overall_signal, overall_confidence);

        Ok(multi_timeframe_analysis)
    }

    fn combine_signals(&self, timeframe_signals: &HashMap<String, AnalysisResponse>) -> (TradingSignal, f64) {
        if timeframe_signals.is_empty() {
            return (TradingSignal::Hold, 0.0);
        }

        // Weight different timeframes (longer timeframes have more weight)
        let timeframe_weights = HashMap::from([
            ("1m".to_string(), 1.0),
            ("5m".to_string(), 2.0),
            ("15m".to_string(), 3.0),
            ("1h".to_string(), 4.0),
            ("4h".to_string(), 5.0),
            ("1d".to_string(), 6.0),
        ]);

        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut total_confidence = 0.0;

        for (timeframe, analysis) in timeframe_signals {
            let weight = timeframe_weights.get(timeframe).unwrap_or(&1.0);
            
            let signal_score = match analysis.signal {
                TradingSignal::StrongBuy => 2.0,
                TradingSignal::Buy => 1.0,
                TradingSignal::Hold => 0.0,
                TradingSignal::Sell => -1.0,
                TradingSignal::StrongSell => -2.0,
            };

            weighted_score += signal_score * weight * analysis.confidence;
            total_weight += weight;
            total_confidence += analysis.confidence;
        }

        let average_score = if total_weight > 0.0 { weighted_score / total_weight } else { 0.0 };
        let average_confidence = total_confidence / timeframe_signals.len() as f64;

        let overall_signal = if average_score >= 1.5 {
            TradingSignal::StrongBuy
        } else if average_score >= 0.5 {
            TradingSignal::Buy
        } else if average_score <= -1.5 {
            TradingSignal::StrongSell
        } else if average_score <= -0.5 {
            TradingSignal::Sell
        } else {
            TradingSignal::Hold
        };

        (overall_signal, average_confidence)
    }

    async fn calculate_trade_parameters(
        &self,
        symbol: &str,
        timeframe_signals: &HashMap<String, AnalysisResponse>,
    ) -> Result<(Option<f64>, Option<f64>, Option<f64>, Option<f64>)> {
        let current_price = self.cache.get_latest_price(symbol);
        
        if current_price.is_none() {
            return Ok((None, None, None, None));
        }
        
        let current_price = current_price.unwrap();
        
        // Use the longest timeframe for main signal direction
        let main_analysis = timeframe_signals
            .get("1d")
            .or_else(|| timeframe_signals.get("4h"))
            .or_else(|| timeframe_signals.get("1h"))
            .or_else(|| timeframe_signals.values().next());

        if let Some(analysis) = main_analysis {
            let entry_price = Some(current_price);
            
            // Calculate stop loss and take profit based on signal
            let (stop_loss, take_profit) = match analysis.signal {
                TradingSignal::Buy | TradingSignal::StrongBuy => {
                    let stop_loss = current_price * 0.98; // 2% stop loss
                    let take_profit = current_price * 1.04; // 4% take profit
                    (Some(stop_loss), Some(take_profit))
                }
                TradingSignal::Sell | TradingSignal::StrongSell => {
                    let stop_loss = current_price * 1.02; // 2% stop loss (price goes up)
                    let take_profit = current_price * 0.96; // 4% take profit (price goes down)
                    (Some(stop_loss), Some(take_profit))
                }
                TradingSignal::Hold => (None, None),
            };

            let risk_reward_ratio = if let (Some(sl), Some(tp)) = (stop_loss, take_profit) {
                let risk = (current_price - sl).abs();
                let reward = (tp - current_price).abs();
                if risk > 0.0 {
                    Some(reward / risk)
                } else {
                    None
                }
            } else {
                None
            };

            Ok((entry_price, stop_loss, take_profit, risk_reward_ratio))
        } else {
            Ok((None, None, None, None))
        }
    }

    pub async fn get_market_overview(&self, symbols: &[String]) -> Result<Vec<MarketOverview>> {
        let mut overviews = Vec::new();

        for symbol in symbols {
            if let Some(latest_price) = self.cache.get_latest_price(symbol) {
                let timeframes = self.cache.get_timeframes_for_symbol(symbol);
                
                let mut latest_analyses = HashMap::new();
                for timeframe in &timeframes {
                    // Get the most recent analysis (in a real implementation, 
                    // you'd cache these analyses)
                    if let Ok(analysis) = self.analyze_single_timeframe(
                        symbol, 
                        timeframe, 
                        "trend_analysis", 
                        Some(50)
                    ).await {
                        latest_analyses.insert(timeframe.clone(), analysis);
                    }
                }

                let overview = MarketOverview {
                    symbol: symbol.clone(),
                    current_price: latest_price,
                    timeframe_analyses: latest_analyses,
                    data_freshness: self.get_data_freshness(symbol),
                };

                overviews.push(overview);
            }
        }

        Ok(overviews)
    }

    fn get_data_freshness(&self, symbol: &str) -> HashMap<String, i64> {
        let mut freshness = HashMap::new();
        let timeframes = self.cache.get_timeframes_for_symbol(symbol);
        
        for timeframe in timeframes {
            if let Some(latest_candle) = self.cache.get_latest_candle(symbol, &timeframe) {
                let age_seconds = (chrono::Utc::now().timestamp_millis() - latest_candle.close_time) / 1000;
                freshness.insert(timeframe, age_seconds);
            }
        }
        
        freshness
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOverview {
    pub symbol: String,
    pub current_price: f64,
    pub timeframe_analyses: HashMap<String, AnalysisResponse>,
    pub data_freshness: HashMap<String, i64>, // Age in seconds
} 