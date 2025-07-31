import { useState, useEffect } from 'react'

interface Position {
  symbol: string
  side: string
  quantity: number
  entryPrice: number
  currentPrice: number
  pnl: number
  pnlPercentage: number
}

export const usePositions = () => {
  const [data, setData] = useState<Position[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    // Simulate API call
    setTimeout(() => {
      setIsLoading(false)
    }, 100)
  }, [])

  return { data, isLoading, error }
}