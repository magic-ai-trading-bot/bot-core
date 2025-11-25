// Trading component shared types
export interface SymbolConfig {
  enabled: boolean;
  leverage: number;
  position_size_pct: number;
  stop_loss_pct: number;
  take_profit_pct: number;
  max_positions: number;
}

export interface Portfolio {
  current_balance: number;
  equity: number;
  total_pnl: number;
  total_pnl_percentage: number;
  total_trades: number;
  win_rate: number;
  margin_used: number;
  free_margin: number;
  average_win: number;
  profit_factor: number;
  max_drawdown: number;
  max_drawdown_percentage: number;
}

export interface Trade {
  id: string;
  symbol: string;
  trade_type: string;
  entry_price: number;
  exit_price?: number;
  quantity: number;
  leverage: number;
  pnl?: number;
  pnl_percentage: number;
  stop_loss?: number;
  take_profit?: number;
  open_time: string | Date;
  close_time?: string | Date;
  status: string;
  close_reason?: string; // TakeProfit, StopLoss, Manual, AISignal, RiskManagement, MarginCall, TimeBasedExit
}
