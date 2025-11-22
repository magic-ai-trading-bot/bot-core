import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { usePaperTrading } from '../../hooks/usePaperTrading'

describe('Debug Reset Portfolio Error', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.stubGlobal('WebSocket', class {
      readyState = 1 // OPEN
      close() {}
    })
  })

  it('should debug reset portfolio error flow', async () => {
    let fetchCallCount = 0

    const mockFetch = vi.fn().mockImplementation(async (url: string) => {
      fetchCallCount++
      console.log(`\n🔵 FETCH #${fetchCallCount}: ${url}`)

      // Mount calls
      if (fetchCallCount === 1) {
        console.log('   → Returning status (is_running: false)')
        return { json: async () => ({ success: true, data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() } }) }
      }
      if (fetchCallCount === 2) {
        console.log('   → Returning open trades []')
        return { json: async () => ({ success: true, data: [] }) }
      }
      if (fetchCallCount === 3) {
        console.log('   → Returning closed trades []')
        return { json: async () => ({ success: true, data: [] }) }
      }
      if (fetchCallCount === 4) {
        console.log('   → Returning settings')
        return { json: async () => ({ success: true, data: { basic: {}, risk: {} } }) }
      }
      // AI signals (4 calls)
      if (fetchCallCount >= 5 && fetchCallCount <= 8) {
        console.log(`   → Returning AI signal ${fetchCallCount - 4}/4`)
        return { json: async () => ({ success: true, data: {} }) }
      }
      // Reset portfolio error
      if (url.includes('/reset')) {
        console.log('   → 🔴 Returning RESET ERROR: "Reset failed"')
        return { json: async () => ({ success: false, error: 'Reset failed' }) }
      }
      // Fallback
      console.log('   → Fallback response')
      return { json: async () => ({ success: true, data: {} }) }
    })

    vi.stubGlobal('fetch', mockFetch)

    console.log('\n═══ RENDERING HOOK ═══')
    const { result } = renderHook(() => usePaperTrading())

    console.log('\nInitial state:', {
      isActive: result.current.isActive,
      isLoading: result.current.isLoading,
      error: result.current.error
    })

    console.log('\n═══ WAITING FOR MOUNT TO COMPLETE ═══')
    await waitFor(() => {
      console.log(`Check: isActive=${result.current.isActive}, isLoading=${result.current.isLoading}`)
      expect(result.current.isActive).toBe(false)
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 5000 })

    console.log('\n✅ Mount complete. Fetch count:', fetchCallCount)
    console.log('State after mount:', {
      isActive: result.current.isActive,
      isLoading: result.current.isLoading,
      error: result.current.error
    })

    console.log('\n═══ CALLING resetPortfolio() ═══')
    await act(async () => {
      console.log('🚀 Invoking resetPortfolio...')
      await result.current.resetPortfolio()
      console.log('✅ resetPortfolio completed')
    })

    console.log('\nState after resetPortfolio:', {
      isActive: result.current.isActive,
      isLoading: result.current.isLoading,
      error: result.current.error
    })

    console.log('\n═══ WAITING FOR ERROR STATE ═══')
    await waitFor(() => {
      console.log(`Check: error="${result.current.error}", isLoading=${result.current.isLoading}`)
      expect(result.current.error).toBe('Reset failed')
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 2000 })

    console.log('\n✅✅✅ TEST PASSED!')
  })
})
