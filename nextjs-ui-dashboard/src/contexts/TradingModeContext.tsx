/**
 * Trading Mode Context
 *
 * Manages Paper/Real trading mode with safety confirmations.
 * Mode persists across page refresh (localStorage).
 */

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import logger from '@/utils/logger';

export type TradingMode = 'paper' | 'real';

export interface TradingModeContextType {
  mode: TradingMode;
  setMode: (mode: TradingMode) => void;
  requestModeSwitch: (targetMode: TradingMode) => void;
  isModeSwitchOpen: boolean;
  closeModeSwitchDialog: () => void;
  confirmModeSwitch: () => void;
  pendingMode: TradingMode | null;
}

const TradingModeContext = createContext<TradingModeContextType | undefined>(undefined);

const STORAGE_KEY = 'trading-mode';
const DEFAULT_MODE: TradingMode = 'paper';

interface TradingModeProviderProps {
  children: ReactNode;
}

export function TradingModeProvider({ children }: TradingModeProviderProps) {
  // Initialize mode from localStorage or default to paper
  const [mode, setModeState] = useState<TradingMode>(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored === 'paper' || stored === 'real') {
        logger.info(`📊 Restored trading mode from localStorage: ${stored}`);
        return stored;
      }
    } catch (error) {
      logger.error('Failed to read trading mode from localStorage:', error);
    }
    return DEFAULT_MODE;
  });

  // Dialog state for mode switching confirmation
  const [isModeSwitchOpen, setIsModeSwitchOpen] = useState(false);
  const [pendingMode, setPendingMode] = useState<TradingMode | null>(null);

  // Persist mode to localStorage whenever it changes
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, mode);
      logger.info(`💾 Saved trading mode to localStorage: ${mode}`);
    } catch (error) {
      logger.error('Failed to save trading mode to localStorage:', error);
    }
  }, [mode]);

  // Set mode directly (used internally after confirmation)
  const setMode = useCallback((newMode: TradingMode) => {
    if (newMode === mode) {
      logger.debug(`Trading mode already set to ${newMode}`);
      return;
    }

    logger.info(`🔄 Switching trading mode: ${mode} → ${newMode}`);
    setModeState(newMode);
  }, [mode]);

  // Request mode switch (triggers confirmation dialog)
  const requestModeSwitch = useCallback((targetMode: TradingMode) => {
    if (targetMode === mode) {
      logger.debug(`Trading mode already set to ${targetMode}`);
      return;
    }

    // Paper → Real requires confirmation
    if (targetMode === 'real') {
      logger.info('⚠️ User requesting switch to REAL trading mode - showing confirmation dialog');
      setPendingMode(targetMode);
      setIsModeSwitchOpen(true);
      return;
    }

    // Real → Paper can happen immediately
    logger.info('✅ Switching to paper trading mode (no confirmation needed)');
    setMode(targetMode);
  }, [mode, setMode]);

  // Close confirmation dialog
  const closeModeSwitchDialog = useCallback(() => {
    logger.info('❌ Mode switch cancelled by user');
    setIsModeSwitchOpen(false);
    setPendingMode(null);
  }, []);

  // Confirm mode switch
  const confirmModeSwitch = useCallback(() => {
    if (pendingMode === null) {
      logger.error('Cannot confirm mode switch: no pending mode');
      return;
    }

    logger.info(`✅ User confirmed switch to ${pendingMode} trading mode`);
    setMode(pendingMode);
    setIsModeSwitchOpen(false);
    setPendingMode(null);
  }, [pendingMode, setMode]);

  const value: TradingModeContextType = {
    mode,
    setMode,
    requestModeSwitch,
    isModeSwitchOpen,
    closeModeSwitchDialog,
    confirmModeSwitch,
    pendingMode,
  };

  return (
    <TradingModeContext.Provider value={value}>
      {children}
    </TradingModeContext.Provider>
  );
}

// Hook to use trading mode context
export function useTradingModeContext(): TradingModeContextType {
  const context = useContext(TradingModeContext);
  if (context === undefined) {
    throw new Error('useTradingModeContext must be used within TradingModeProvider');
  }
  return context;
}
