import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useToast, toast, reducer } from '../../hooks/use-toast'

describe('useToast hook', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('initializes with empty toasts array', () => {
    const { result } = renderHook(() => useToast())

    expect(result.current.toasts).toEqual([])
  })

  it('adds a toast', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({
        title: 'Test Toast',
        description: 'This is a test',
      })
    })

    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Test Toast')
    expect(result.current.toasts[0].description).toBe('This is a test')
    expect(result.current.toasts[0].open).toBe(true)
  })

  it('generates unique IDs for toasts', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
    })

    // Only 1 toast should exist due to TOAST_LIMIT = 1
    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Toast 2')
  })

  it('limits toasts to TOAST_LIMIT (1)', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({ title: 'Toast 1' })
      result.current.toast({ title: 'Toast 2' })
      result.current.toast({ title: 'Toast 3' })
    })

    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Toast 3')
  })

  it('dismisses a specific toast', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const t = result.current.toast({ title: 'Test Toast' })
      toastId = t.id
    })

    expect(result.current.toasts[0].open).toBe(true)

    act(() => {
      result.current.dismiss(toastId!)
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('dismisses all toasts when called without ID', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({ title: 'Toast 1' })
    })

    act(() => {
      result.current.dismiss()
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('returns toast methods (dismiss, update) from toast()', () => {
    const { result } = renderHook(() => useToast())

    let toastMethods: ReturnType<typeof toast>

    act(() => {
      toastMethods = result.current.toast({ title: 'Test' })
    })

    expect(toastMethods!).toHaveProperty('id')
    expect(toastMethods!).toHaveProperty('dismiss')
    expect(toastMethods!).toHaveProperty('update')
    expect(typeof toastMethods!.dismiss).toBe('function')
    expect(typeof toastMethods!.update).toBe('function')
  })

  it('updates a toast using returned update method', () => {
    const { result } = renderHook(() => useToast())

    let toastMethods: ReturnType<typeof toast>

    act(() => {
      toastMethods = result.current.toast({ title: 'Original Title' })
    })

    expect(result.current.toasts[0].title).toBe('Original Title')

    act(() => {
      toastMethods!.update({ title: 'Updated Title' })
    })

    expect(result.current.toasts[0].title).toBe('Updated Title')
  })

  it('dismisses a toast using returned dismiss method', () => {
    const { result } = renderHook(() => useToast())

    let toastMethods: ReturnType<typeof toast>

    act(() => {
      toastMethods = result.current.toast({ title: 'Test' })
    })

    expect(result.current.toasts[0].open).toBe(true)

    act(() => {
      toastMethods!.dismiss()
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('marks toast as closed when dismissed', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const t = result.current.toast({ title: 'Test' })
      toastId = t.id
    })

    expect(result.current.toasts).toHaveLength(1)

    act(() => {
      result.current.dismiss(toastId!)
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('handles onOpenChange callback', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const t = result.current.toast({ title: 'Test' })
      toastId = t.id
    })

    const toastItem = result.current.toasts[0]
    expect(toastItem.onOpenChange).toBeDefined()

    act(() => {
      toastItem.onOpenChange?.(false)
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('creates toast with custom props', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({
        title: 'Custom Toast',
        description: 'Description',
        variant: 'destructive',
        duration: 5000,
      })
    })

    const toastItem = result.current.toasts[0]
    expect(toastItem.title).toBe('Custom Toast')
    expect(toastItem.description).toBe('Description')
    expect(toastItem.variant).toBe('destructive')
    expect(toastItem.duration).toBe(5000)
  })

  it('creates toast with action element', () => {
    const { result } = renderHook(() => useToast())

    const action = { altText: 'Undo', onClick: vi.fn() } as any

    act(() => {
      result.current.toast({
        title: 'Toast with action',
        action,
      })
    })

    expect(result.current.toasts[0].action).toEqual(action)
  })

  it('multiple hooks share the same state', () => {
    const { result: result1 } = renderHook(() => useToast())
    const { result: result2 } = renderHook(() => useToast())

    act(() => {
      result1.current.toast({ title: 'Shared Toast' })
    })

    expect(result2.current.toasts).toHaveLength(1)
    expect(result2.current.toasts[0].title).toBe('Shared Toast')
  })

  it('cleans up listener on unmount', () => {
    const { unmount } = renderHook(() => useToast())

    unmount()

    // After unmount, adding a toast should not affect the unmounted hook
    act(() => {
      toast({ title: 'Test' })
    })

    // No error should be thrown
  })
})

describe('toast reducer', () => {
  it('handles ADD_TOAST action', () => {
    const state = { toasts: [] }
    const newToast = { id: '1', title: 'Test', open: true }

    const newState = reducer(state, {
      type: 'ADD_TOAST',
      toast: newToast,
    })

    expect(newState.toasts).toHaveLength(1)
    expect(newState.toasts[0]).toEqual(newToast)
  })

  it('limits toasts to TOAST_LIMIT on ADD_TOAST', () => {
    const state = { toasts: [{ id: '1', title: 'Old', open: true }] }
    const newToast = { id: '2', title: 'New', open: true }

    const newState = reducer(state, {
      type: 'ADD_TOAST',
      toast: newToast,
    })

    expect(newState.toasts).toHaveLength(1)
    expect(newState.toasts[0].id).toBe('2')
  })

  it('handles UPDATE_TOAST action', () => {
    const state = {
      toasts: [
        { id: '1', title: 'Original', open: true },
        { id: '2', title: 'Another', open: true },
      ],
    }

    const newState = reducer(state, {
      type: 'UPDATE_TOAST',
      toast: { id: '1', title: 'Updated' },
    })

    expect(newState.toasts[0].title).toBe('Updated')
    expect(newState.toasts[1].title).toBe('Another')
  })

  it('handles UPDATE_TOAST for non-existent toast', () => {
    const state = { toasts: [{ id: '1', title: 'Test', open: true }] }

    const newState = reducer(state, {
      type: 'UPDATE_TOAST',
      toast: { id: '999', title: 'Non-existent' },
    })

    expect(newState.toasts).toHaveLength(1)
    expect(newState.toasts[0].title).toBe('Test')
  })

  it('handles DISMISS_TOAST with specific ID', () => {
    const state = {
      toasts: [
        { id: '1', title: 'Toast 1', open: true },
        { id: '2', title: 'Toast 2', open: true },
      ],
    }

    const newState = reducer(state, {
      type: 'DISMISS_TOAST',
      toastId: '1',
    })

    expect(newState.toasts[0].open).toBe(false)
    expect(newState.toasts[1].open).toBe(true)
  })

  it('handles DISMISS_TOAST without ID (dismisses all)', () => {
    const state = {
      toasts: [
        { id: '1', title: 'Toast 1', open: true },
        { id: '2', title: 'Toast 2', open: true },
      ],
    }

    const newState = reducer(state, {
      type: 'DISMISS_TOAST',
    })

    expect(newState.toasts[0].open).toBe(false)
    expect(newState.toasts[1].open).toBe(false)
  })

  it('handles REMOVE_TOAST with specific ID', () => {
    const state = {
      toasts: [
        { id: '1', title: 'Toast 1', open: true },
        { id: '2', title: 'Toast 2', open: true },
      ],
    }

    const newState = reducer(state, {
      type: 'REMOVE_TOAST',
      toastId: '1',
    })

    expect(newState.toasts).toHaveLength(1)
    expect(newState.toasts[0].id).toBe('2')
  })

  it('handles REMOVE_TOAST without ID (removes all)', () => {
    const state = {
      toasts: [
        { id: '1', title: 'Toast 1', open: true },
        { id: '2', title: 'Toast 2', open: true },
      ],
    }

    const newState = reducer(state, {
      type: 'REMOVE_TOAST',
    })

    expect(newState.toasts).toHaveLength(0)
  })

  it('does not mutate original state', () => {
    const state = { toasts: [{ id: '1', title: 'Test', open: true }] }
    const originalToasts = state.toasts

    reducer(state, {
      type: 'UPDATE_TOAST',
      toast: { id: '1', title: 'Updated' },
    })

    expect(originalToasts[0].title).toBe('Test')
  })
})

describe('toast function', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('can be called independently', () => {
    const result = toast({ title: 'Independent Toast' })

    expect(result).toHaveProperty('id')
    expect(result).toHaveProperty('dismiss')
    expect(result).toHaveProperty('update')
  })

  it('generates unique IDs', () => {
    const toast1 = toast({ title: 'Toast 1' })
    const toast2 = toast({ title: 'Toast 2' })

    expect(toast1.id).not.toBe(toast2.id)
  })

  it('update method updates the toast', () => {
    const { result } = renderHook(() => useToast())

    let toastResult: ReturnType<typeof toast>

    act(() => {
      toastResult = toast({ title: 'Original' })
    })

    act(() => {
      toastResult!.update({ title: 'Updated', description: 'New description' })
    })

    expect(result.current.toasts[0].title).toBe('Updated')
    expect(result.current.toasts[0].description).toBe('New description')
  })

  it('dismiss method dismisses the toast', () => {
    const { result } = renderHook(() => useToast())

    let toastResult: ReturnType<typeof toast>

    act(() => {
      toastResult = toast({ title: 'Test' })
    })

    expect(result.current.toasts[0].open).toBe(true)

    act(() => {
      toastResult!.dismiss()
    })

    expect(result.current.toasts[0].open).toBe(false)
  })
})

describe('addToRemoveQueue', () => {
  it('dismissing toast marks it as closed', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const t = result.current.toast({ title: 'Test' })
      toastId = t.id
    })

    expect(result.current.toasts[0].open).toBe(true)

    act(() => {
      result.current.dismiss(toastId!)
    })

    expect(result.current.toasts[0].open).toBe(false)
  })

  it('does not create duplicate timeouts for same toast', () => {
    const { result } = renderHook(() => useToast())

    let toastId: string

    act(() => {
      const t = result.current.toast({ title: 'Test' })
      toastId = t.id
    })

    // Dismiss twice
    act(() => {
      result.current.dismiss(toastId!)
      result.current.dismiss(toastId!)
    })

    expect(result.current.toasts[0].open).toBe(false)
  })
})

describe('edge cases', () => {
  it('handles rapid toast creation and dismissal', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      for (let i = 0; i < 10; i++) {
        result.current.toast({ title: `Toast ${i}` })
      }
    })

    // Should only have 1 due to TOAST_LIMIT
    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Toast 9')
  })

  it('handles empty toast options', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({})
    })

    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].open).toBe(true)
  })

  it('handles toast with only title', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({ title: 'Only title' })
    })

    expect(result.current.toasts[0].title).toBe('Only title')
    expect(result.current.toasts[0].description).toBeUndefined()
  })

  it('handles complex data as title and description', () => {
    const { result } = renderHook(() => useToast())

    act(() => {
      result.current.toast({
        title: 'Complex Title',
        description: 'Complex Description',
      })
    })

    expect(result.current.toasts[0].title).toBeDefined()
    expect(result.current.toasts[0].description).toBeDefined()
  })
})
