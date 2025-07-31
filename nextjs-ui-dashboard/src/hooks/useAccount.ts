import { useState, useEffect } from 'react'

interface AccountData {
  balance: Record<string, number>
  total_balance_usdt: number
  total_pnl: number
  daily_pnl: number
}

export const useAccount = () => {
  const [data, setData] = useState<AccountData>({
    balance: { USDT: 0, BTC: 0, ETH: 0 },
    total_balance_usdt: 0,
    total_pnl: 0,
    daily_pnl: 0
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