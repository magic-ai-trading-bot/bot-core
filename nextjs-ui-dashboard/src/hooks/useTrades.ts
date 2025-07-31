import { useState, useEffect } from 'react'

interface Trade {
  id: string
  symbol: string
  side: string
  price: number
  quantity: number
  timestamp: number
}

interface TradesData {
  trades: Trade[]
  pagination: {
    page: number
    limit: number
    total: number
    pages: number
  }
}

export const useTrades = () => {
  const [data, setData] = useState<TradesData>({
    trades: [],
    pagination: { page: 1, limit: 10, total: 0, pages: 0 }
  })
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    setTimeout(() => {
      setIsLoading(false)
    }, 100)
  }, [])

  return { data, isLoading, error }
}