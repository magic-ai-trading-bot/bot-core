import { useState, useEffect, useCallback } from 'react'
import { apiClient } from '@/services/api'

interface MarketData {
  price: number
  change24h: number
  volume: number
  high24h?: number
  low24h?: number
  priceChangePercent?: number
  lastUpdate?: string
}

/**
 * Hook for fetching real-time market data from the backend
 *
 * IMPORTANT: This connects to the backend API - NOT a mock implementation
 *
 * @spec:FR-MARKET-001 - Real-time Market Data
 * @ref:specs/02-design/2.3-api/API-RUST-CORE.md
 *
 * @param symbol - Trading symbol (e.g., 'BTCUSDT')
 * @param timeframe - Timeframe for chart data (default: '1h')
 * @param refreshInterval - Auto-refresh interval in ms (default: 5000ms = 5s)
 */
export const useMarketData = (
  symbol: string = 'BTCUSDT',
  timeframe: string = '1h',
  refreshInterval: number = 5000
) => {
  const [data, setData] = useState<MarketData>({
    price: 0,
    change24h: 0,
    volume: 0,
    high24h: 0,
    low24h: 0,
    priceChangePercent: 0,
  })
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  /**
   * Fetch market data from backend
   */
  const fetchMarketData = useCallback(async () => {
    try {
      // Call backend API to get chart data
      const chartData = await apiClient.rust.getChartData(symbol, timeframe, 100)

      // Extract market metrics from chart data
      setData({
        price: chartData.latest_price,
        change24h: chartData.price_change_24h,
        volume: chartData.volume_24h,
        priceChangePercent: chartData.price_change_percent_24h,
        high24h: Math.max(...chartData.candles.map(c => c.high)),
        low24h: Math.min(...chartData.candles.map(c => c.low)),
        lastUpdate: new Date().toISOString(),
      })

      setError(null)
    } catch (err) {
      const error = err as { response?: { data?: { error?: string } }; message?: string }
      const errorMessage = error.response?.data?.error || error.message || 'Failed to fetch market data'
      setError(errorMessage)

      // Keep previous data on error, don't reset to zero
      // This prevents UI from showing $0 during temporary network issues
    } finally {
      setIsLoading(false)
    }
  }, [symbol, timeframe])

  /**
   * Initial fetch on mount and when dependencies change
   */
  useEffect(() => {
    setIsLoading(true)
    fetchMarketData()
  }, [fetchMarketData])

  /**
   * Auto-refresh market data at specified interval
   */
  useEffect(() => {
    if (refreshInterval <= 0) {
      return // Disable auto-refresh if interval is 0 or negative
    }

    const intervalId = setInterval(() => {
      fetchMarketData()
    }, refreshInterval)

    return () => clearInterval(intervalId)
  }, [fetchMarketData, refreshInterval])

  /**
   * Manual refresh function
   */
  const refresh = useCallback(() => {
    setIsLoading(true)
    fetchMarketData()
  }, [fetchMarketData])

  return {
    data,
    isLoading,
    error,
    refresh
  }
}
