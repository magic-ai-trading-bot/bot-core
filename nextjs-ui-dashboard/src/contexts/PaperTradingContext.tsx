import React, { createContext, useContext, ReactNode } from "react";
import { usePaperTrading } from "@/hooks/usePaperTrading";

// Export types from usePaperTrading for convenience
export type {
  PaperTradingSettings,
  PaperTrade,
  PortfolioMetrics,
  AISignal,
} from "@/hooks/usePaperTrading";

// Type for the context value - matches what usePaperTrading returns
type PaperTradingContextType = ReturnType<typeof usePaperTrading>;

const PaperTradingContext = createContext<PaperTradingContextType | undefined>(
  undefined
);

/**
 * PaperTradingProvider - Provides shared paper trading state to all children
 *
 * This prevents multiple components from creating separate instances of
 * usePaperTrading hook, which would cause duplicate API calls.
 *
 * Usage:
 * 1. Wrap your app/page with PaperTradingProvider
 * 2. Use usePaperTradingContext() in child components instead of usePaperTrading()
 */
export const PaperTradingProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  // Single instance of the hook - shared by all consumers
  const paperTrading = usePaperTrading();

  return (
    <PaperTradingContext.Provider value={paperTrading}>
      {children}
    </PaperTradingContext.Provider>
  );
};

/**
 * usePaperTradingContext - Access shared paper trading state
 *
 * Must be used within a PaperTradingProvider.
 * All components using this hook share the same state and don't make duplicate API calls.
 */
export const usePaperTradingContext = (): PaperTradingContextType => {
  const context = useContext(PaperTradingContext);
  if (!context) {
    throw new Error(
      "usePaperTradingContext must be used within PaperTradingProvider"
    );
  }
  return context;
};

// Also export the provider's display name for debugging
PaperTradingProvider.displayName = "PaperTradingProvider";
