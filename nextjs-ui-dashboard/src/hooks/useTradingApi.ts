import { useState } from 'react'
import { apiClient } from '@/services/api'

interface TradeParams {
  symbol: string
  side: 'BUY' | 'SELL'
  quantity: number
  price?: number
  type: 'market' | 'limit'
  leverage?: number
  stop_loss?: number
  take_profit?: number
}

interface TradeResponse {
  trade_id: string
  status: 'executed' | 'pending' | 'failed'
  entry_price?: number
  quantity?: number
  timestamp?: number
  message?: string
}

/**
 * Hook for executing manual trades through the paper trading engine
 *
 * IMPORTANT: This connects to the backend API - NOT a mock implementation
 *
 * @spec:FR-TRADING-007 - Manual Trade Execution
 * @ref:specs/02-design/2.3-api/API-RUST-CORE.md
 */
export const useTradingApi = () => {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  /**
   * Execute a manual trade through the paper trading engine
   *
   * @param params - Trade parameters including symbol, side, quantity, etc.
   * @returns Trade execution result with trade ID and status
   */
  const executeTrade = async (params: TradeParams): Promise<TradeResponse> => {
    setIsLoading(true)
    setError(null)

    try {
      // Validate parameters
      if (!params.symbol || params.symbol.trim() === '') {
        throw new Error('Symbol is required')
      }

      if (!params.side || (params.side !== 'BUY' && params.side !== 'SELL')) {
        throw new Error('Side must be either BUY or SELL')
      }

      if (!params.quantity || params.quantity <= 0) {
        throw new Error('Quantity must be greater than 0')
      }

      if (params.type === 'limit' && (!params.price || params.price <= 0)) {
        throw new Error('Price is required for limit orders')
      }

      // Call backend API to execute manual trade
      // This endpoint will be added to rust-core-engine/src/api/paper_trading.rs
      const response = await apiClient.rust.client.post<TradeResponse>(
        '/api/paper-trading/execute-trade',
        {
          symbol: params.symbol,
          side: params.side,
          quantity: params.quantity,
          order_type: params.type,
          limit_price: params.price,
          leverage: params.leverage,
          stop_loss: params.stop_loss,
          take_profit: params.take_profit,
        }
      )

      return response.data
    } catch (err) {
      const error = err as { response?: { data?: { error?: string } }; message?: string }
      const errorMessage = error.response?.data?.error || error.message || 'Failed to execute trade'
      setError(errorMessage)
      throw new Error(errorMessage)
    } finally {
      setIsLoading(false)
    }
  }

  /**
   * Clear error state
   */
  const clearError = () => {
    setError(null)
  }

  return {
    executeTrade,
    isLoading,
    error,
    clearError
  }
}
