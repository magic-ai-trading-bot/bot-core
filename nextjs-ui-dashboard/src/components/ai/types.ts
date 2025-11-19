// AI signals shared types
export interface CombinedSignal {
  signal: string;
  confidence: number;
  timestamp: string | number;
  symbol?: string;
  reasoning?: string;
  strategy_scores?: Record<string, number>;
  market_analysis?: AIMarketAnalysis;
  risk_assessment?: AIRiskAssessment;
  source: string;
  model_type?: string;
}

export interface FormattedSignal {
  id: string;
  signal: "LONG" | "SHORT" | "NEUTRAL";
  confidence: number;
  timestamp: string;
  pair: string;
  reason: string;
  active: boolean;
  marketAnalysis?: AIMarketAnalysis;
  riskAssessment?: AIRiskAssessment;
  strategyScores?: Record<string, number>;
  source: string;
  isWebSocket: boolean;
}

export interface AIMarketAnalysis {
  trend_direction: string;
  trend_strength: number;
  support_levels: number[];
  resistance_levels: number[];
  volatility_level: string;
  volume_analysis: string;
}

export interface AIRiskAssessment {
  overall_risk: string;
  technical_risk: number;
  market_risk: number;
  recommended_position_size: number;
  stop_loss_suggestion: number | null;
  take_profit_suggestion: number | null;
}
