import { useState, useEffect } from 'react'

interface MarketData {
  price: number
  change24h: number
  volume: number
}

export const useMarketData = () => {
  const [data, setData] = useState<MarketData>({
    price: 0,
    change24h: 0,
    volume: 0
  })
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    setTimeout(() => {
      setIsLoading(false)
    }, 100)
  }, [])

  return { data, isLoading }
}