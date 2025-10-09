import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { usePositions } from '../../hooks/usePositions'

describe('usePositions', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with empty positions array', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toEqual([])
    expect(result.current.isLoading).toBe(true)
    expect(result.current.error).toBe(null)
  })

  it('sets loading to false after timeout', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(true)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('maintains empty array while loading changes', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const initialData = result.current.data

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.data).toEqual([])
    expect(result.current.data).toBe(initialData)
  })

  it('error state remains null', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.error).toBe(null)
  })

  it('returns all three expected properties', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('data')
    expect(result.current).toHaveProperty('isLoading')
    expect(result.current).toHaveProperty('error')
  })

  it('data is an array', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(Array.isArray(result.current.data)).toBe(true)
  })

  it('does not re-trigger loading on re-render', async () => {
    const { result, rerender } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    rerender()

    expect(result.current.isLoading).toBe(false)
  })

  it('has correct initial and final loading state', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(true)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('data array length is zero initially and after loading', async () => {
    const { result } = renderHook(() => usePositions())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data.length).toBe(0)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(result.current.data.length).toBe(0)
  })
})
