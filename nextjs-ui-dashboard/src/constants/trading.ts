// Trading Configuration Constants

/**
 * Leverage limits
 */
export const MAX_LEVERAGE = 20;
export const MIN_LEVERAGE = 1;
export const DEFAULT_LEVERAGE = 5;

/**
 * Risk management limits (as decimal, e.g., 0.05 = 5%)
 */
export const MAX_RISK_PERCENT = 0.05; // 5%
export const MIN_RISK_PERCENT = 0.01; // 1%
export const DEFAULT_RISK_PERCENT = 0.02; // 2%

/**
 * Position sizing limits
 */
export const MAX_POSITION_SIZE_PERCENT = 0.5; // 50% of balance
export const MIN_POSITION_SIZE_PERCENT = 0.01; // 1% of balance
export const DEFAULT_POSITION_SIZE_PERCENT = 0.1; // 10% of balance

/**
 * Stop loss / Take profit limits (as decimal, e.g., 0.02 = 2%)
 */
export const MAX_STOP_LOSS_PERCENT = 0.1; // 10%
export const MIN_STOP_LOSS_PERCENT = 0.005; // 0.5%
export const DEFAULT_STOP_LOSS_PERCENT = 0.02; // 2%

export const MAX_TAKE_PROFIT_PERCENT = 0.5; // 50%
export const MIN_TAKE_PROFIT_PERCENT = 0.01; // 1%
export const DEFAULT_TAKE_PROFIT_PERCENT = 0.05; // 5%

/**
 * Trading limits
 */
export const MAX_OPEN_POSITIONS = 10;
export const MIN_TRADE_AMOUNT_USDT = 10;
export const MAX_TRADE_AMOUNT_USDT = 100000;

/**
 * Indicator parameters - RSI
 */
export const RSI_PERIOD = 14;
export const RSI_OVERSOLD = 30;
export const RSI_OVERBOUGHT = 70;

/**
 * Indicator parameters - MACD
 */
export const MACD_FAST_PERIOD = 12;
export const MACD_SLOW_PERIOD = 26;
export const MACD_SIGNAL_PERIOD = 9;

/**
 * Indicator parameters - Bollinger Bands
 */
export const BB_PERIOD = 20;
export const BB_STD_DEV = 2;

/**
 * Indicator parameters - Volume
 */
export const VOLUME_MA_PERIOD = 20;
export const VOLUME_SPIKE_THRESHOLD = 2.0; // 2x average volume

/**
 * WebSocket configuration
 */
export const WS_RECONNECT_DELAY_MS = 3000;
export const WS_MAX_RECONNECT_ATTEMPTS = 5;
export const WS_PING_INTERVAL_MS = 30000; // 30 seconds
export const WS_PONG_TIMEOUT_MS = 5000; // 5 seconds

/**
 * API configuration
 */
export const API_TIMEOUT_MS = 10000; // 10 seconds
export const API_RETRY_ATTEMPTS = 3;
export const API_RETRY_DELAY_MS = 1000;

/**
 * UI/UX constants
 */
export const DEBOUNCE_DELAY_MS = 300;
export const TOAST_DURATION_MS = 3000;
export const PAGINATION_PAGE_SIZE = 10;
export const MAX_VISIBLE_TRADES = 50;

/**
 * Chart configuration
 */
export const CHART_UPDATE_INTERVAL_MS = 1000; // 1 second
export const CHART_MAX_DATA_POINTS = 100;
export const CHART_CANDLE_INTERVAL = '1m'; // 1 minute

/**
 * Performance thresholds
 */
export const GOOD_LATENCY_MS = 100;
export const WARNING_LATENCY_MS = 500;
export const ERROR_LATENCY_MS = 1000;

/**
 * Balance & Portfolio constants
 */
export const INITIAL_PAPER_BALANCE_USDT = 10000;
export const MIN_BALANCE_WARNING_USDT = 1000;
export const LOW_BALANCE_PERCENT = 0.2; // 20% of initial balance
