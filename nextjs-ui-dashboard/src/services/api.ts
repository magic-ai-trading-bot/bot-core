// API Client for Bot Core Services Integration
import axios, { AxiosInstance, AxiosRequestConfig } from "axios";
import logger from "@/utils/logger";

// Environment variables
const RUST_API_URL =
  import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";
const PYTHON_AI_URL =
  import.meta.env.VITE_PYTHON_AI_URL || "http://localhost:8000";
const API_TIMEOUT = parseInt(import.meta.env.VITE_API_TIMEOUT || "10000");

// Type definitions for API responses
export interface BotStatus {
  status: "running" | "stopped" | "error";
  uptime: number;
  active_positions: number;
  total_trades: number;
  total_pnl: number;
  last_update: string;
}

export interface Position {
  symbol: string;
  side: "BUY" | "SELL";
  size: number;
  entry_price: number;
  current_price: number;
  unrealized_pnl: number;
  stop_loss?: number;
  take_profit?: number;
  timestamp: string;
}

export interface TradeHistory {
  id: string;
  symbol: string;
  side: "BUY" | "SELL";
  quantity: number;
  entry_price: number;
  exit_price?: number;
  pnl?: number;
  entry_time: string;
  exit_time?: string;
  status: "open" | "closed";
}

export interface AISignal {
  signal: "long" | "short" | "neutral";
  confidence: number;
  probability: number;
  timestamp: string;
  model_type: string;
  symbol: string;
  timeframe: string;
}

// NEW AI TYPES - Compatible with Python AI Service
export interface CandleDataAI {
  open_time: number;
  close_time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  quote_volume: number;
  trades: number;
  is_closed: boolean;
}

export interface AIStrategyContext {
  selected_strategies: string[];
  market_condition: string;
  risk_level: string;
  user_preferences: Record<string, unknown>;
  technical_indicators: Record<string, unknown>;
}

export interface AIAnalysisRequest {
  symbol: string;
  timeframe_data: Record<string, CandleDataAI[]>;
  current_price: number;
  volume_24h: number;
  timestamp: number;
  strategy_context: AIStrategyContext;
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
  stop_loss_suggestion?: number;
  take_profit_suggestion?: number;
}

export interface AISignalResponse {
  signal: string;
  confidence: number;
  reasoning: string;
  strategy_scores: Record<string, number>;
  market_analysis: AIMarketAnalysis;
  risk_assessment: AIRiskAssessment;
  timestamp: number;
  symbol?: string; // Add symbol field for display
}

export interface StrategyRecommendation {
  strategy_name: string;
  suitability_score: number;
  reasoning: string;
  recommended_config: Record<string, unknown>;
}

export interface MarketConditionAnalysis {
  condition_type: string;
  confidence: number;
  characteristics: string[];
  recommended_strategies: string[];
  market_phase: string;
}

export interface AIServiceInfo {
  service_name: string;
  version: string;
  model_version: string;
  supported_timeframes: string[];
  supported_symbols: string[];
  capabilities: string[];
  last_trained?: string;
}

export interface AIModelInfo {
  model_type: string;
  model_loaded: boolean;
  training_samples?: number;
  validation_samples?: number;
  feature_count?: number;
  training_accuracy?: number;
  trained_timestamp?: string;
}

export interface PerformanceStats {
  total_pnl: number;
  win_rate: number;
  total_trades: number;
  avg_trade_duration: number;
  max_drawdown: number;
  sharpe_ratio?: number;
  best_trade: number;
  worst_trade: number;
  profit_factor: number;
}

export interface CandleData {
  timestamp: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

// NEW: Chart data types
export interface ChartData {
  symbol: string;
  timeframe: string;
  candles: CandleData[];
  latest_price: number;
  volume_24h: number;
  price_change_24h: number;
  price_change_percent_24h: number;
}

export interface SupportedSymbols {
  symbols: string[];
  available_timeframes: string[];
}

export interface AddSymbolRequest {
  symbol: string;
  timeframes: string[];
}

// Authentication types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  full_name?: string;
}

export interface LoginResponse {
  token: string;
  user: UserProfile;
}

export interface UserProfile {
  id: string;
  email: string;
  full_name?: string;
  is_active: boolean;
  is_admin: boolean;
  created_at: string;
  last_login?: string;
  settings: UserSettings;
}

export interface UserSettings {
  trading_enabled: boolean;
  risk_level: "Low" | "Medium" | "High";
  max_positions: number;
  default_quantity: number;
  notifications: NotificationSettings;
}

export interface NotificationSettings {
  email_alerts: boolean;
  trade_notifications: boolean;
  system_alerts: boolean;
}

// Account Information types
export interface AccountInfo {
  account_id: string;
  balance: number;
  available_balance: number;
  currency: string;
  created_at: string;
  updated_at: string;
  account_type: string;
  status: string;
}

export interface TradingConfig {
  enabled: boolean;
  max_positions: number;
  risk_percentage: number;
  stop_loss_percentage: number;
  take_profit_percentage: number;
  max_daily_trades: number;
  trading_hours: {
    start: string;
    end: string;
  };
}

export interface MarketOverview {
  total_symbols: number;
  active_symbols: number;
  market_status: string;
  last_update: string;
  top_performers: Array<{
    symbol: string;
    price: number;
    change_percent: number;
    volume: number;
  }>;
  market_stats: {
    total_volume: number;
    total_trades: number;
    avg_price_change: number;
  };
}

export interface ModelConfig {
  model_type: string;
  sequence_length: number;
  batch_size: number;
  learning_rate: number;
  epochs: number;
  validation_split: number;
  features: string[];
}

export interface AITradingConfig {
  enabled: boolean;
  model_confidence_threshold: number;
  max_signal_age_minutes: number;
  supported_timeframes: string[];
  risk_management: {
    max_position_size: number;
    stop_loss_percentage: number;
    take_profit_percentage: number;
  };
}

export interface DataConfig {
  data_source: string;
  update_interval_seconds: number;
  historical_data_days: number;
  required_indicators: string[];
  cache_enabled: boolean;
  cache_ttl_minutes: number;
}

// Base API Client class with retry logic
class BaseApiClient {
  protected client: AxiosInstance;

  constructor(baseURL: string, serviceName: string) {
    this.client = axios.create({
      baseURL,
      timeout: API_TIMEOUT,
      headers: {
        "Content-Type": "application/json",
        "X-Client": "BotCoreDashboard/1.0",
      },
    });

    // Add auth token to requests if available
    this.client.interceptors.request.use((config) => {
      let token = null;
      try {
        if (typeof window !== 'undefined' && window?.localStorage) {
          token = window.localStorage.getItem("authToken");
        }
      } catch (error) {
        // Handle SecurityError in test environments
        token = null;
      }
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Request interceptor
    this.client.interceptors.request.use(
      (config) => config,
      (error) => Promise.reject(error)
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        logger.error(
          `API Error [${serviceName}]:`,
          error.response?.status || "Network Error",
          error.message
        );
        return Promise.reject(error);
      }
    );
  }

  protected async requestWithRetry<T>(
    request: () => Promise<T>,
    maxRetries: number = 3,
    backoffMs: number = 1000
  ): Promise<T> {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await request();
      } catch (error) {
        if (attempt === maxRetries) {
          throw error;
        }

        // Exponential backoff
        const delay = backoffMs * Math.pow(2, attempt - 1);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
    throw new Error("Max retries exceeded");
  }
}

// Rust Trading Engine API Client
class RustTradingApiClient extends BaseApiClient {
  constructor() {
    super(RUST_API_URL, "RustAPI");
  }

  // Bot Status and Control
  async getBotStatus(): Promise<BotStatus> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/status");
      return response.data;
    });
  }

  async startBot(): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/bot/start");
      return response.data;
    });
  }

  async stopBot(): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/bot/stop");
      return response.data;
    });
  }

  // Position Management
  async getPositions(): Promise<Position[]> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/positions");
      return response.data;
    });
  }

  async closePosition(
    symbol: string
  ): Promise<{ message: string; exit_price: number; pnl: number }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post(`/api/positions/${symbol}/close`);
      return response.data;
    });
  }

  async closeAllPositions(): Promise<{
    message: string;
    closed_positions: number;
  }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/positions/close-all");
      return response.data;
    });
  }

  // Trading History
  async getTradeHistory(limit: number = 100): Promise<TradeHistory[]> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get(
        `/api/trades/history?limit=${limit}`
      );
      return response.data;
    });
  }

  // Performance Statistics
  async getPerformanceStats(): Promise<PerformanceStats> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/performance/stats");
      return response.data;
    });
  }

  // Account Information
  async getAccountInfo(): Promise<AccountInfo> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/account");
      return response.data;
    });
  }

  // Configuration
  async updateTradingConfig(config: {
    enabled?: boolean;
    max_positions?: number;
    risk_percentage?: number;
    stop_loss_percentage?: number;
    take_profit_percentage?: number;
  }): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.put("/api/config/trading", config);
      return response.data;
    });
  }

  async getTradingConfig(): Promise<TradingConfig> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/config/trading");
      return response.data;
    });
  }

  // Market Data
  async getMarketData(
    symbol: string,
    timeframe: string,
    limit: number = 100
  ): Promise<CandleData[]> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get(
        `/api/market-data/${symbol}/${timeframe}?limit=${limit}`
      );
      return response.data;
    });
  }

  // NEW: Chart Data Methods
  async getChartData(
    symbol: string,
    timeframe: string,
    limit?: number,
    signal?: AbortSignal
  ): Promise<ChartData> {
    return this.requestWithRetry(async () => {
      const params = limit ? `?limit=${limit}` : "";
      const response = await this.client.get(
        `/api/market/chart/${symbol}/${timeframe}${params}`,
        { signal }
      );
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  // Fast chart data loading without retry (for initial page load)
  async getChartDataFast(
    symbol: string,
    timeframe: string,
    limit?: number,
    signal?: AbortSignal
  ): Promise<ChartData> {
    const params = limit ? `?limit=${limit}` : "";
    const response = await this.client.get(
      `/api/market/chart/${symbol}/${timeframe}${params}`,
      { signal }
    );
    return response.data.data || response.data;
  }

  async getMultiChartData(
    symbols: string[],
    timeframes: string[],
    limit?: number
  ): Promise<ChartData[]> {
    return this.requestWithRetry(async () => {
      const symbolsParam = symbols.join(",");
      const timeframesParam = timeframes.join(",");
      const limitParam = limit ? `&limit=${limit}` : "";
      const response = await this.client.get(
        `/api/market/charts?symbols=${symbolsParam}&timeframes=${timeframesParam}${limitParam}`
      );
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  async getSupportedSymbols(signal?: AbortSignal): Promise<SupportedSymbols> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/market/symbols", { signal });
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  async addSymbol(request: AddSymbolRequest): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/market/symbols", request);
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  async removeSymbol(symbol: string): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.delete(
        `/api/market/symbols/${symbol}`
      );
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  async getLatestPrices(): Promise<Record<string, number>> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/market/prices");
      // Extract data from {success, data, error} wrapper
      return response.data.data || response.data;
    });
  }

  async getMarketOverview(): Promise<MarketOverview> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/market/overview");
      return response.data;
    });
  }

  // NEW: AI Integration - Routes through Rust to Python AI Service
  async analyzeAI(request: AIAnalysisRequest): Promise<AISignalResponse> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/ai/analyze", request);
      return response.data.data || response.data;
    });
  }

  async getStrategyRecommendations(data: {
    symbol: string;
    timeframe_data: Record<string, CandleDataAI[]>;
    current_price: number;
    available_strategies: string[];
    timestamp: number;
  }): Promise<StrategyRecommendation[]> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post(
        "/api/ai/strategy-recommendations",
        data
      );
      return response.data.data || response.data;
    });
  }

  async analyzeMarketCondition(data: {
    symbol: string;
    timeframe_data: Record<string, CandleDataAI[]>;
    current_price: number;
    volume_24h: number;
    timestamp: number;
  }): Promise<MarketConditionAnalysis> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/ai/market-condition", data);
      return response.data.data || response.data;
    });
  }

  async sendAIFeedback(feedback: {
    signal_id: string;
    symbol: string;
    predicted_signal: string;
    actual_outcome: string;
    profit_loss: number;
    confidence_was_accurate: boolean;
    feedback_notes?: string;
    timestamp: number;
  }): Promise<{ message: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/ai/feedback", feedback);
      return response.data.data || response.data;
    });
  }

  async getAIServiceInfo(): Promise<AIServiceInfo> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/ai/info");
      return response.data.data || response.data;
    });
  }

  async getSupportedStrategies(): Promise<{ strategies: string[] }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/ai/strategies");
      return response.data.data || response.data;
    });
  }

  // Health Check
  async healthCheck(): Promise<{ status: string }> {
    const response = await this.client.get("/api/health");
    return response.data;
  }
}

// Authentication API Client
class AuthApiClient extends BaseApiClient {
  constructor() {
    super(RUST_API_URL, "AuthAPI");
  }

  async login(request: LoginRequest): Promise<LoginResponse> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/auth/login", request);
      // Extract data from {success, data, error} wrapper
      if (response.data.success) {
        return response.data.data;
      } else {
        throw new Error(response.data.error || "Login failed");
      }
    });
  }

  async register(request: RegisterRequest): Promise<LoginResponse> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/api/auth/register", request);
      // Extract data from {success, data, error} wrapper
      if (response.data.success) {
        return response.data.data;
      } else {
        throw new Error(response.data.error || "Registration failed");
      }
    });
  }

  async verifyToken(): Promise<{
    user_id: string;
    email: string;
    is_admin: boolean;
    exp: number;
  }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/auth/verify");
      // Extract data from {success, data, error} wrapper
      if (response.data.success) {
        return response.data.data;
      } else {
        throw new Error(response.data.error || "Token verification failed");
      }
    });
  }

  async getProfile(): Promise<UserProfile> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/api/auth/profile");
      // Extract data from {success, data, error} wrapper
      if (response.data.success) {
        return response.data.data;
      } else {
        throw new Error(response.data.error || "Failed to get profile");
      }
    });
  }

  // Utility methods
  setAuthToken(token: string): void {
    try {
      if (typeof window !== 'undefined' && window?.localStorage) {
        window.localStorage.setItem("authToken", token);
      }
    } catch (error) {
      // Handle SecurityError in test environments
    }
  }

  removeAuthToken(): void {
    try {
      if (typeof window !== 'undefined' && window?.localStorage) {
        window.localStorage.removeItem("authToken");
      }
    } catch (error) {
      // Handle SecurityError in test environments
    }
  }

  getAuthToken(): string | null {
    try {
      if (typeof window !== 'undefined' && window?.localStorage) {
        return window.localStorage.getItem("authToken");
      }
    } catch (error) {
      // Handle SecurityError in test environments
      return null;
    }
    return null;
  }

  isTokenExpired(token?: string): boolean {
    const authToken = token || this.getAuthToken();
    if (!authToken) return true;

    try {
      const payload = JSON.parse(atob(authToken.split(".")[1]));
      const exp = payload.exp * 1000; // Convert to milliseconds
      return Date.now() >= exp;
    } catch {
      return true;
    }
  }
}

// Python AI Service API Client
class PythonAIApiClient extends BaseApiClient {
  constructor() {
    super(PYTHON_AI_URL, "PythonAI");
  }

  // AI Model Management
  async getModelInfo(): Promise<AIModelInfo> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/model/info");
      return response.data;
    });
  }

  async trainModel(data: {
    symbol: string;
    model_type?: string;
    retrain?: boolean;
    candles: CandleData[];
  }): Promise<{
    message: string;
    model_type: string;
    training_samples: number;
    status: string;
  }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/train", data);
      return response.data;
    }, 1); // No retry for training as it's a long operation
  }

  async loadModel(
    modelPath?: string
  ): Promise<{ message: string; model_type: string; status: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/model/load", {
        model_path: modelPath,
      });
      return response.data;
    });
  }

  async saveModel(
    modelName?: string
  ): Promise<{ message: string; model_type: string; timestamp: string }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/model/save", {
        model_name: modelName,
      });
      return response.data;
    });
  }

  // AI Analysis
  async analyzeMarket(data: {
    symbol: string;
    timeframe: string;
    candles: CandleData[];
  }): Promise<AISignal> {
    return this.requestWithRetry(async () => {
      const response = await this.client.post("/analyze", data);
      return response.data;
    });
  }

  // Configuration
  async getConfig(): Promise<{
    supported_timeframes: string[];
    model_config: ModelConfig;
    trading_config: AITradingConfig;
    data_config: DataConfig;
  }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.get("/config");
      return response.data;
    });
  }

  // Model Cleanup
  async cleanupOldModels(
    keepCount: number = 5
  ): Promise<{ message: string; deleted_models: number; kept_models: number }> {
    return this.requestWithRetry(async () => {
      const response = await this.client.delete(
        `/model/cleanup?keep_count=${keepCount}`
      );
      return response.data;
    });
  }

  // Health Check
  async healthCheck(): Promise<{
    status: string;
    timestamp: string;
    model_loaded: boolean;
    version: string;
  }> {
    const response = await this.client.get("/health");
    return response.data;
  }
}

// Combined API Client
export class BotCoreApiClient {
  public rust: RustTradingApiClient;
  public python: PythonAIApiClient;
  public auth: AuthApiClient;

  constructor() {
    this.rust = new RustTradingApiClient();
    this.python = new PythonAIApiClient();
    this.auth = new AuthApiClient();
  }

  // Combined health check for both services
  async healthCheck(): Promise<{
    rust: { status: string; healthy: boolean };
    python: { status: string; healthy: boolean; model_loaded: boolean };
    overall: boolean;
  }> {
    try {
      const [rustHealth, pythonHealth] = await Promise.allSettled([
        this.rust.healthCheck(),
        this.python.healthCheck(),
      ]);

      const rustHealthy = rustHealth.status === "fulfilled";
      const pythonHealthy = pythonHealth.status === "fulfilled";

      return {
        rust: {
          status: rustHealthy ? rustHealth.value.status : "error",
          healthy: rustHealthy,
        },
        python: {
          status: pythonHealthy ? pythonHealth.value.status : "error",
          healthy: pythonHealthy,
          model_loaded: pythonHealthy ? pythonHealth.value.model_loaded : false,
        },
        overall: rustHealthy && pythonHealthy,
      };
    } catch (error) {
      logger.error("Health check failed:", error);
      return {
        rust: { status: "error", healthy: false },
        python: { status: "error", healthy: false, model_loaded: false },
        overall: false,
      };
    }
  }

  // Get comprehensive dashboard data
  async getDashboardData(): Promise<{
    botStatus: BotStatus;
    positions: Position[];
    aiModelInfo: AIModelInfo;
    performanceStats: PerformanceStats;
    recentTrades: TradeHistory[];
  }> {
    try {
      const [
        botStatus,
        positions,
        aiModelInfo,
        performanceStats,
        recentTrades,
      ] = await Promise.all([
        this.rust.getBotStatus(),
        this.rust.getPositions(),
        this.python.getModelInfo(),
        this.rust.getPerformanceStats(),
        this.rust.getTradeHistory(20),
      ]);

      return {
        botStatus,
        positions,
        aiModelInfo,
        performanceStats,
        recentTrades,
      };
    } catch (error) {
      logger.error("Failed to fetch dashboard data:", error);
      throw error;
    }
  }
}

// Create singleton instance
export const apiClient = new BotCoreApiClient();

// Export individual clients for specific use cases
export const rustApi = apiClient.rust;
export const pythonAI = apiClient.python;
