import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useAccount } from '../../hooks/useAccount'

describe('useAccount', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with default account data', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toEqual({
      balance: { USDT: 0, BTC: 0, ETH: 0 },
      total_balance_usdt: 0,
      total_pnl: 0,
      daily_pnl: 0
    })
    expect(result.current.isLoading).toBe(true)
    expect(result.current.error).toBe(null)
  })

  it('sets loading to false after timeout', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(true)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('maintains account data structure while loading changes', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const initialData = result.current.data

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    // Data should remain unchanged
    expect(result.current.data).toEqual(initialData)
  })

  it('error state remains null', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.error).toBe(null)
  })

  it('has correct balance structure with USDT, BTC, ETH', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data.balance).toHaveProperty('USDT')
    expect(result.current.data.balance).toHaveProperty('BTC')
    expect(result.current.data.balance).toHaveProperty('ETH')
    expect(typeof result.current.data.balance.USDT).toBe('number')
    expect(typeof result.current.data.balance.BTC).toBe('number')
    expect(typeof result.current.data.balance.ETH).toBe('number')
  })

  it('has numeric total_balance_usdt', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.total_balance_usdt).toBe('number')
    expect(result.current.data.total_balance_usdt).toBe(0)
  })

  it('has numeric total_pnl', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.total_pnl).toBe('number')
    expect(result.current.data.total_pnl).toBe(0)
  })

  it('has numeric daily_pnl', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.daily_pnl).toBe('number')
    expect(result.current.data.daily_pnl).toBe(0)
  })

  it('returns all three expected properties', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('data')
    expect(result.current).toHaveProperty('isLoading')
    expect(result.current).toHaveProperty('error')
  })

  it('does not re-trigger loading on re-render', async () => {
    const { result, rerender } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    rerender()

    // Should still be false after re-render
    expect(result.current.isLoading).toBe(false)
  })

  it('timeout executes and changes loading state', async () => {
    const { result } = renderHook(() => useAccount())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // Initially loading
    expect(result.current.isLoading).toBe(true)

    // After timeout, loading should be false
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })
})
