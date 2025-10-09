import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useTrades } from '../../hooks/useTrades'

describe('useTrades', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with default trades data', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toEqual({
      trades: [],
      pagination: { page: 1, limit: 10, total: 0, pages: 0 }
    })
    expect(result.current.isLoading).toBe(true)
    expect(result.current.error).toBe(null)
  })

  it('sets loading to false after timeout', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(true)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('maintains trades data structure while loading changes', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const initialData = result.current.data

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.data).toEqual(initialData)
  })

  it('error state remains null', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.error).toBe(null)
  })

  it('has correct trades and pagination structure', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toHaveProperty('trades')
    expect(result.current.data).toHaveProperty('pagination')
    expect(Array.isArray(result.current.data.trades)).toBe(true)

    expect(result.current.data.pagination).toHaveProperty('page')
    expect(result.current.data.pagination).toHaveProperty('limit')
    expect(result.current.data.pagination).toHaveProperty('total')
    expect(result.current.data.pagination).toHaveProperty('pages')
  })

  it('pagination has correct default values', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data.pagination.page).toBe(1)
    expect(result.current.data.pagination.limit).toBe(10)
    expect(result.current.data.pagination.total).toBe(0)
    expect(result.current.data.pagination.pages).toBe(0)
  })

  it('trades array is empty initially', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data.trades.length).toBe(0)
  })

  it('returns all three expected properties', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('data')
    expect(result.current).toHaveProperty('isLoading')
    expect(result.current).toHaveProperty('error')
  })

  it('does not re-trigger loading on re-render', async () => {
    const { result, rerender } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    rerender()

    expect(result.current.isLoading).toBe(false)
  })

  it('maintains data structure after loading completes', async () => {
    const { result } = renderHook(() => useTrades())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.data.trades).toEqual([])
    expect(result.current.data.pagination).toEqual({
      page: 1,
      limit: 10,
      total: 0,
      pages: 0
    })
  })
})
