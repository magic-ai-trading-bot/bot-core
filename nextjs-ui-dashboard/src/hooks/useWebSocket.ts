import { useState, useEffect, useRef, useCallback } from "react";
import { Position, TradeHistory, BotStatus, AISignal } from "@/services/api";
import logger from "@/utils/logger";
import {
  WS_PING_INTERVAL_MS,
  WS_PONG_TIMEOUT_MS,
  WARNING_LATENCY_MS,
  ERROR_LATENCY_MS,
} from "@/constants/trading";

// WebSocket message types from Rust Trading Engine
export interface WebSocketMessage {


// @spec:FR-DASHBOARD-006 - WebSocket Integration
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-040

  type:
    | "PositionUpdate"
    | "TradeExecuted"
    | "AISignalReceived"
    | "BotStatusUpdate"
    | "ChartUpdate"
    | "MarketData"
    | "Connected"
    | "Ping"
    | "Pong"
    | "Error";
  data?:
    | PositionUpdateData
    | TradeExecutedData
    | AISignalReceivedData
    | BotStatusUpdateData
    | ChartUpdateData
    | MarketDataUpdateData
    | ErrorData;
  message?: string;
  timestamp: string;
}

export interface ErrorData {
  message: string;
  code?: string;
  details?: unknown;
}

export interface PositionUpdateData {
  symbol: string;
  side: string;
  pnl: number;
  current_price: number;
  unrealized_pnl: number;
  timestamp: number;
}

export interface TradeExecutedData {
  symbol: string;
  side: string;
  quantity: number;
  price: number;
  timestamp: number;
  pnl?: number;
}

export interface AISignalReceivedData {
  symbol: string;
  signal: string;
  confidence: number;
  timestamp: number;
  model_type: string;
  timeframe: string;
  reasoning?: string;
  strategy_scores?: Record<string, number>;
  market_analysis?: {
    trend_direction: string;
    trend_strength: number;
    support_levels: number[];
    resistance_levels: number[];
    volatility_level: string;
    volume_analysis: string;
  };
  risk_assessment?: {
    overall_risk: string;
    technical_risk: number;
    market_risk: number;
    recommended_position_size: number;
    stop_loss_suggestion: number | null;
    take_profit_suggestion: number | null;
  };
}

export interface BotStatusUpdateData {
  status: string;
  active_positions: number;
  total_pnl: number;
  total_trades: number;
  uptime: number;
}

export interface ChartUpdateData {
  symbol: string;
  timeframe: string;
  candle: {
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    is_closed: boolean;
  };
  latest_price: number;
  price_change_24h: number;
  price_change_percent_24h: number;
  volume_24h: number;
  timestamp: number;
}

export interface MarketDataUpdateData {
  symbol: string;
  price: number;
  price_change_24h: number;
  price_change_percent_24h: number;
  volume_24h: number;
  timestamp: number;
}

export interface WebSocketState {
  isConnected: boolean;
  isConnecting: boolean;
  error: string | null;
  lastMessage: WebSocketMessage | null;
  botStatus: BotStatus | null;
  positions: Position[];
  aiSignals: AISignal[];
  recentTrades: TradeHistory[];
  latency: number;
  connectionQuality: "good" | "slow" | "poor";
}

export interface OutgoingWebSocketMessage {
  type: string;
  data?: unknown;
  timestamp?: string;
}

export interface WebSocketHook {
  state: WebSocketState;
  connect: () => void;
  disconnect: () => void;
  sendMessage: (message: OutgoingWebSocketMessage) => void;
}

const WS_URL = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws";
const RECONNECT_INTERVAL = 5000; // 5 seconds
const MAX_RECONNECT_ATTEMPTS = 10;

export const useWebSocket = (): WebSocketHook => {
  const [state, setState] = useState<WebSocketState>({
    isConnected: false,
    isConnecting: false,
    error: null,
    lastMessage: null,
    botStatus: null,
    positions: [],
    aiSignals: [],
    recentTrades: [],
    latency: 0,
    connectionQuality: "good",
  });

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const shouldReconnectRef = useRef(true);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const pongTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastPingTimeRef = useRef<number>(0);
  // Ref to store latest connectWebSocket function (avoids stale closure in handleClose)
  const connectWebSocketRef = useRef<() => void>(() => {});

  const stopHeartbeat = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
    if (pongTimeoutRef.current) {
      clearTimeout(pongTimeoutRef.current);
      pongTimeoutRef.current = null;
    }
  }, []);

  const startHeartbeat = useCallback(() => {
    stopHeartbeat();

    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      return;
    }

    const sendPing = () => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        lastPingTimeRef.current = Date.now();
        wsRef.current.send(
          JSON.stringify({
            type: "Ping",
            timestamp: new Date().toISOString(),
          })
        );

        // Set timeout for pong response
        pongTimeoutRef.current = setTimeout(() => {
          logger.warn("Pong timeout - connection may be unstable");
          setState((prev) => ({
            ...prev,
            connectionQuality: "poor",
            latency: WS_PONG_TIMEOUT_MS,
          }));

          // Reconnect on timeout
          if (wsRef.current) {
            wsRef.current.close();
          }
        }, WS_PONG_TIMEOUT_MS);
      }
    };

    // Send initial ping
    sendPing();

    // Set up interval for periodic pings
    pingIntervalRef.current = setInterval(sendPing, WS_PING_INTERVAL_MS);
  }, [stopHeartbeat]);

  const handlePong = useCallback(() => {
    if (pongTimeoutRef.current) {
      clearTimeout(pongTimeoutRef.current);
      pongTimeoutRef.current = null;
    }

    const latency = Date.now() - lastPingTimeRef.current;
    let quality: "good" | "slow" | "poor" = "good";

    if (latency >= ERROR_LATENCY_MS) {
      quality = "poor";
    } else if (latency >= WARNING_LATENCY_MS) {
      quality = "slow";
    }

    setState((prev) => ({
      ...prev,
      latency,
      connectionQuality: quality,
    }));

    logger.debug(`WebSocket latency: ${latency}ms (${quality})`);
  }, []);

  const updatePosition = useCallback((positionData: PositionUpdateData) => {
    setState((prev) => ({
      ...prev,
      positions: prev.positions.map((position) =>
        position.symbol === positionData.symbol
          ? {
              ...position,
              current_price: positionData.current_price,
              unrealized_pnl: positionData.unrealized_pnl,
              timestamp: new Date(positionData.timestamp).toISOString(),
            }
          : position
      ),
    }));
  }, []);

  const addTradeToHistory = useCallback((tradeData: TradeExecutedData) => {
    const newTrade: TradeHistory = {
      id: `${tradeData.timestamp}-${tradeData.symbol}`,
      symbol: tradeData.symbol,
      side: tradeData.side as "BUY" | "SELL",
      quantity: tradeData.quantity,
      entry_price: tradeData.price,
      exit_price: tradeData.side === "SELL" ? tradeData.price : undefined,
      pnl: tradeData.pnl,
      entry_time: new Date(tradeData.timestamp).toISOString(),
      exit_time:
        tradeData.side === "SELL"
          ? new Date(tradeData.timestamp).toISOString()
          : undefined,
      status: tradeData.side === "SELL" ? "closed" : "open",
    };

    setState((prev) => ({
      ...prev,
      recentTrades: [newTrade, ...prev.recentTrades.slice(0, 19)], // Keep last 20 trades
    }));
  }, []);

  const addAISignal = useCallback((signalData: AISignalReceivedData) => {
    const newSignal: AISignal = {
      signal: signalData.signal as "long" | "short" | "neutral",
      confidence: signalData.confidence,
      probability: signalData.confidence,
      timestamp:
        typeof signalData.timestamp === "number"
          ? new Date(signalData.timestamp).toISOString()
          : new Date(signalData.timestamp).toISOString(),
      model_type: signalData.model_type,
      symbol: signalData.symbol,
      timeframe: signalData.timeframe,
      reasoning: signalData.reasoning, // Pass through reasoning from backend
      strategy_scores: signalData.strategy_scores, // Pass through strategy scores from backend
    };

    setState((prev) => ({
      ...prev,
      aiSignals: [newSignal, ...prev.aiSignals.slice(0, 19)], // Keep last 20 signals
    }));
  }, []);

  const updateBotStatus = useCallback((statusData: BotStatusUpdateData) => {
    const newStatus: BotStatus = {
      status: statusData.status as "running" | "stopped" | "error",
      uptime: statusData.uptime,
      active_positions: statusData.active_positions,
      total_trades: statusData.total_trades,
      total_pnl: statusData.total_pnl,
      last_update: new Date().toISOString(),
    };

    setState((prev) => ({
      ...prev,
      botStatus: newStatus,
    }));
  }, []);

  const handleMessage = useCallback(
    (event: MessageEvent) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        setState((prev) => ({ ...prev, lastMessage: message }));

        switch (message.type) {
          case "Connected":
            // WebSocket connected successfully
            break;
          case "Ping":
            // Server ping - respond with pong
            if (wsRef.current?.readyState === WebSocket.OPEN) {
              wsRef.current.send(
                JSON.stringify({
                  type: "Pong",
                  timestamp: new Date().toISOString(),
                })
              );
            }
            break;
          case "Pong":
            // Keep-alive response - update latency
            handlePong();
            break;
          case "PositionUpdate":
            if (message.data)
              updatePosition(message.data as PositionUpdateData);
            break;
          case "TradeExecuted":
            if (message.data)
              addTradeToHistory(message.data as TradeExecutedData);
            break;
          case "AISignalReceived":
            if (message.data) addAISignal(message.data as AISignalReceivedData);
            break;
          case "BotStatusUpdate":
            if (message.data)
              updateBotStatus(message.data as BotStatusUpdateData);
            break;
          case "ChartUpdate":
            // Handle chart update
            break;
          case "MarketData":
            // Handle market data update
            break;
          case "Error":
            logger.error("WebSocket error:", message.data);
            if (message.data) {
              setState((prev) => ({
                ...prev,
                error: (message.data as ErrorData).message,
              }));
            }
            break;
          default:
            logger.warn("Unknown message type:", message.type);
        }
      } catch (error) {
        logger.error("Failed to parse WebSocket message:", error);
        setState((prev) => ({
          ...prev,
          error: "Failed to parse WebSocket message",
        }));
      }
    },
    [updatePosition, addTradeToHistory, addAISignal, updateBotStatus, handlePong]
  );

  const handleOpen = useCallback(() => {
    reconnectAttemptsRef.current = 0;
    setState((prev) => ({
      ...prev,
      isConnected: true,
      isConnecting: false,
      error: null,
    }));

    // Start heartbeat monitoring
    startHeartbeat();
  }, [startHeartbeat]);

  const handleClose = useCallback(
    (event: CloseEvent) => {
      // Stop heartbeat monitoring
      stopHeartbeat();

      setState((prev) => ({
        ...prev,
        isConnected: false,
      }));

      // Attempt reconnection if not explicitly closed
    if (
      shouldReconnectRef.current &&
      reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS
    ) {
      const delay = Math.min(
        RECONNECT_INTERVAL * Math.pow(2, reconnectAttemptsRef.current),
        30000
      );

      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectAttemptsRef.current++;
        // Use ref to access latest connectWebSocket (avoids stale closure)
        connectWebSocketRef.current?.();
      }, delay);
    }
    },
    [stopHeartbeat]
  ); // connectWebSocket accessed via ref - no dependency needed

  const handleError = useCallback((event: Event) => {
    logger.error("WebSocket error:", event);
    setState((prev) => ({
      ...prev,
      error: "WebSocket connection error",
      isConnecting: false,
    }));
  }, []);

  // Remove from useCallback dependencies to prevent infinite loop
  const connectWebSocket = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    if (wsRef.current?.readyState === WebSocket.CONNECTING) {
      return; // Already connecting
    }

    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = handleOpen;
      ws.onclose = handleClose;
      ws.onerror = handleError;
      ws.onmessage = handleMessage;

      wsRef.current = ws;
    } catch (error) {
      logger.error("Failed to create WebSocket connection:", error);
      setState((prev) => ({
        ...prev,
        error: "Failed to create WebSocket connection",
      }));
    }
     
  }, []);

  // Update ref whenever connectWebSocket changes (for handleClose to access latest version)
  connectWebSocketRef.current = connectWebSocket;

  const connect = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isConnecting: true,
      error: null,
    }));

    connectWebSocket();
     
  }, []);

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;

    // Stop heartbeat monitoring
    stopHeartbeat();

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setState((prev) => ({
      ...prev,
      isConnected: false,
      isConnecting: false,
    }));
  }, [stopHeartbeat]);

  const sendMessage = useCallback((message: OutgoingWebSocketMessage) => {
    const messageStr = JSON.stringify(message);

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(messageStr);
    } else {
      logger.warn("WebSocket is not connected. Cannot send message.");
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      shouldReconnectRef.current = false;
      stopHeartbeat();
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [stopHeartbeat]);

  // Auto-connect on mount (only once)
  useEffect(() => {
    if (import.meta.env.VITE_ENABLE_REALTIME !== "false") {
      connect();
    }
     
  }, []); // Empty array: run only once on mount

  return {
    state,
    connect,
    disconnect,
    sendMessage,
  };
};
