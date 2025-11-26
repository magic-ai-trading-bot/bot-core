import React, { createContext, useContext, ReactNode } from "react";
import { useWebSocket, WebSocketHook } from "@/hooks/useWebSocket";

// Create context with undefined as initial value
const WebSocketContext = createContext<WebSocketHook | undefined>(undefined);

/**
 * WebSocketProvider - Provides shared WebSocket connection to all children
 *
 * This prevents multiple components from creating separate WebSocket connections,
 * which would cause duplicate connections and wasted resources.
 *
 * Usage:
 * 1. Wrap your app/page with WebSocketProvider
 * 2. Use useWebSocketContext() in child components instead of useWebSocket()
 */
export const WebSocketProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  // Single instance of the WebSocket hook - shared by all consumers
  const webSocket = useWebSocket();

  return (
    <WebSocketContext.Provider value={webSocket}>
      {children}
    </WebSocketContext.Provider>
  );
};

/**
 * useWebSocketContext - Access shared WebSocket connection
 *
 * Must be used within a WebSocketProvider.
 * All components using this hook share the same WebSocket connection.
 */
export const useWebSocketContext = (): WebSocketHook => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error(
      "useWebSocketContext must be used within WebSocketProvider"
    );
  }
  return context;
};

// Also export the provider's display name for debugging
WebSocketProvider.displayName = "WebSocketProvider";
