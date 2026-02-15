// Mock API response fixtures for MCP server integration tests

export const MOCK_HEALTH_RESPONSE = {
  status: "ok",
  version: "1.0.0",
  uptime: 86400,
  services: {
    database: "connected",
    binance: "connected",
    ai_service: "connected",
  },
};

export const MOCK_MARKET_PRICES = [
  { symbol: "BTCUSDT", price: 97500.0, change_24h: 2.3 },
  { symbol: "ETHUSDT", price: 3850.0, change_24h: -0.5 },
  { symbol: "BNBUSDT", price: 625.0, change_24h: 1.1 },
  { symbol: "SOLUSDT", price: 185.0, change_24h: 3.2 },
];

export const MOCK_PAPER_STATUS = {
  is_running: true,
  trading_pair: "BTCUSDT",
  start_time: "2026-02-15T00:00:00Z",
  total_trades: 42,
  open_positions: 2,
};

export const MOCK_PAPER_PORTFOLIO = {
  initial_balance: 10000.0,
  current_balance: 10450.0,
  total_pnl: 450.0,
  total_pnl_percent: 4.5,
  positions: [
    {
      symbol: "BTCUSDT",
      side: "LONG",
      entry_price: 96000.0,
      current_price: 97500.0,
      quantity: 0.01,
      unrealized_pnl: 15.0,
    },
  ],
};

export const MOCK_BASIC_SETTINGS = {
  rsi_oversold: 30,
  rsi_overbought: 70,
  stop_loss_percent: 2.0,
  take_profit_percent: 4.0,
  position_size_percent: 5.0,
  max_positions: 4,
  leverage: 10,
  confidence_threshold: 0.65,
  max_daily_loss_percent: 10.0,
};

export const MOCK_PERFORMANCE = {
  total_trades: 100,
  winning_trades: 65,
  losing_trades: 35,
  win_rate: 0.65,
  total_pnl: 1250.0,
  sharpe_ratio: 1.6,
  max_drawdown: 5.2,
};
