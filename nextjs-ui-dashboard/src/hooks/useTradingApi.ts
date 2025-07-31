import { useState } from 'react'

interface TradeParams {
  symbol: string
  side: string
  quantity: number
  price: number
  type: string
}

export const useTradingApi = () => {
  const [isLoading, setIsLoading] = useState(false)

  const executeTrade = async (params: TradeParams) => {
    setIsLoading(true)
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      return { trade_id: 'trade123', status: 'executed' }
    } finally {
      setIsLoading(false)
    }
  }

  return { executeTrade, isLoading }
}