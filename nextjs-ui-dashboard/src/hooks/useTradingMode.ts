/**
 * Trading Mode Hook
 *
 * Wrapper hook for easy access to TradingModeContext.
 */

import { useTradingModeContext } from '@/contexts/TradingModeContext';

export function useTradingMode() {
  return useTradingModeContext();
}
