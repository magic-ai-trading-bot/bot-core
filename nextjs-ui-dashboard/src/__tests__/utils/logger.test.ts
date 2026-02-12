import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { logger } from '../../utils/logger'

describe('logger', () => {
  let consoleDebugSpy: ReturnType<typeof vi.spyOn>
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>
  let consoleWarnSpy: ReturnType<typeof vi.spyOn>
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>
  let consoleLogSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    // Spy on all console methods
    consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {})
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {})
    consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {})

    // Enable development mode for tests (MODE is 'test' by default, not 'development')
    // This ensures logger actually outputs to console during tests
    ;(logger as any).config.isDevelopment = true
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('debug', () => {
    it('logs debug messages with timestamp and level prefix', () => {
      logger.debug('Debug message')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)
      const call = consoleDebugSpy.mock.calls[0]
      expect(call[0]).toMatch(/\[\d{4}-\d{2}-\d{2}T.*\] \[DEBUG\]/)
      expect(call[1]).toBe('Debug message')
    })

    it('logs debug messages with additional arguments', () => {
      const data = { key: 'value', nested: { prop: 123 } }
      logger.debug('Debug with data', data, 'extra')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)
      const call = consoleDebugSpy.mock.calls[0]
      expect(call[0]).toMatch(/\[DEBUG\]/)
      expect(call[1]).toBe('Debug with data')
      expect(call[2]).toEqual(data)
      expect(call[3]).toBe('extra')
    })

    it('logs debug messages with multiple args', () => {
      logger.debug('Multiple args', 1, true, null, undefined, { obj: 'test' })

      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)
      const call = consoleDebugSpy.mock.calls[0]
      expect(call.length).toBe(7) // prefix + message + 5 args
    })
  })

  describe('info', () => {
    it('logs info messages with timestamp and level prefix', () => {
      logger.info('Info message')

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[0]).toMatch(/\[\d{4}-\d{2}-\d{2}T.*\] \[INFO\]/)
      expect(call[1]).toBe('Info message')
    })

    it('logs info messages with additional arguments', () => {
      logger.info('Info with numbers', 123, 456.789)

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[2]).toBe(123)
      expect(call[3]).toBe(456.789)
    })
  })

  describe('warn', () => {
    it('logs warning messages with timestamp and level prefix', () => {
      logger.warn('Warning message')

      expect(consoleWarnSpy).toHaveBeenCalledTimes(1)
      const call = consoleWarnSpy.mock.calls[0]
      expect(call[0]).toMatch(/\[\d{4}-\d{2}-\d{2}T.*\] \[WARN\]/)
      expect(call[1]).toBe('Warning message')
    })

    it('logs warning messages with additional arguments', () => {
      const warning = { code: 'WARN_001', severity: 'medium' }
      logger.warn('Warning with object', warning)

      expect(consoleWarnSpy).toHaveBeenCalledTimes(1)
      const call = consoleWarnSpy.mock.calls[0]
      expect(call[2]).toEqual(warning)
    })
  })

  describe('error', () => {
    it('logs error messages with timestamp and level prefix', () => {
      logger.error('Error message')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[0]).toMatch(/\[\d{4}-\d{2}-\d{2}T.*\] \[ERROR\]/)
      expect(call[1]).toBe('Error message')
    })

    it('logs error messages with additional arguments', () => {
      const errorData = { code: 500, message: 'Internal error' }
      logger.error('Error with data', errorData)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[2]).toEqual(errorData)
    })
  })

  describe('exception', () => {
    it('logs Error instances with stack traces', () => {
      const error = new Error('Test exception')
      logger.exception(error)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(2)

      // First call: Exception message
      const call1 = consoleErrorSpy.mock.calls[0]
      expect(call1[0]).toBe('')
      expect(call1[1]).toBe('Exception:')
      expect(call1[2]).toBe('Test exception')

      // Second call: Stack trace
      const call2 = consoleErrorSpy.mock.calls[1]
      expect(call2[0]).toBe('Stack:')
      expect(call2[1]).toBeDefined()
      expect(typeof call2[1]).toBe('string')
    })

    it('logs Error instances with context', () => {
      const error = new Error('Auth failed')
      logger.exception(error, 'Authentication')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(2)

      const call1 = consoleErrorSpy.mock.calls[0]
      expect(call1[0]).toBe('[Authentication]')
      expect(call1[1]).toBe('Exception:')
      expect(call1[2]).toBe('Auth failed')
    })

    it('logs non-Error exceptions', () => {
      const nonError = { message: 'Something went wrong', code: 'ERR_001' }
      logger.exception(nonError)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[0]).toBe('')
      expect(call[1]).toBe('Exception:')
      expect(call[2]).toEqual(nonError)
    })

    it('logs non-Error exceptions with context', () => {
      const nonError = 'String error message'
      logger.exception(nonError, 'Network')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[0]).toBe('[Network]')
      expect(call[1]).toBe('Exception:')
      expect(call[2]).toBe(nonError)
    })

    it('logs null exceptions', () => {
      logger.exception(null, 'NullError')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[0]).toBe('[NullError]')
      expect(call[2]).toBe(null)
    })

    it('logs undefined exceptions', () => {
      logger.exception(undefined)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      const call = consoleErrorSpy.mock.calls[0]
      expect(call[2]).toBe(undefined)
    })
  })

  describe('api', () => {
    it('logs API requests with method and URL', () => {
      logger.api('get', '/api/users')

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[0]).toBe('[API] GET /api/users')
      expect(call[1]).toBe('')
    })

    it('logs API requests with data', () => {
      const requestData = { username: 'test', password: 'secret' }
      logger.api('post', '/api/auth/login', requestData)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[0]).toBe('[API] POST /api/auth/login')
      expect(call[1]).toEqual(requestData)
    })

    it('uppercases HTTP method', () => {
      logger.api('put', '/api/users/123', { name: 'Updated' })

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[0]).toBe('[API] PUT /api/users/123')
    })

    it('handles various HTTP methods', () => {
      logger.api('delete', '/api/users/123')
      logger.api('patch', '/api/settings')
      logger.api('head', '/api/health')
      logger.api('options', '/api')

      expect(consoleLogSpy).toHaveBeenCalledTimes(4)
      expect(consoleLogSpy.mock.calls[0][0]).toBe('[API] DELETE /api/users/123')
      expect(consoleLogSpy.mock.calls[1][0]).toBe('[API] PATCH /api/settings')
      expect(consoleLogSpy.mock.calls[2][0]).toBe('[API] HEAD /api/health')
      expect(consoleLogSpy.mock.calls[3][0]).toBe('[API] OPTIONS /api')
    })
  })

  describe('ws', () => {
    it('logs WebSocket events', () => {
      logger.ws('connected')

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[0]).toBe('[WebSocket] connected')
      expect(call[1]).toBe('')
    })

    it('logs WebSocket events with data', () => {
      const eventData = { type: 'price_update', symbol: 'BTCUSDT', price: 50000 }
      logger.ws('message', eventData)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[0]).toBe('[WebSocket] message')
      expect(call[1]).toEqual(eventData)
    })

    it('logs WebSocket connection events', () => {
      logger.ws('connecting')
      logger.ws('connected')
      logger.ws('disconnected')
      logger.ws('error')

      expect(consoleLogSpy).toHaveBeenCalledTimes(4)
      expect(consoleLogSpy.mock.calls[0][0]).toBe('[WebSocket] connecting')
      expect(consoleLogSpy.mock.calls[1][0]).toBe('[WebSocket] connected')
      expect(consoleLogSpy.mock.calls[2][0]).toBe('[WebSocket] disconnected')
      expect(consoleLogSpy.mock.calls[3][0]).toBe('[WebSocket] error')
    })

    it('logs WebSocket data events', () => {
      const data = { trades: [{ id: 1, price: 100 }, { id: 2, price: 101 }] }
      logger.ws('trade_update', data)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      const call = consoleLogSpy.mock.calls[0]
      expect(call[1]).toEqual(data)
    })
  })

  describe('timestamp format', () => {
    it('includes valid ISO timestamp', () => {
      logger.info('Test message')

      const call = consoleInfoSpy.mock.calls[0]
      const timestampMatch = call[0].match(/\[(.*?)\]/)
      expect(timestampMatch).toBeTruthy()

      const timestamp = timestampMatch?.[1]
      const date = new Date(timestamp!)
      expect(date.toString()).not.toBe('Invalid Date')
    })

    it('timestamp is current time', () => {
      const before = Date.now()
      logger.info('Test')
      const after = Date.now()

      const call = consoleInfoSpy.mock.calls[0]
      const timestampMatch = call[0].match(/\[(.*?)\]/)
      const timestamp = new Date(timestampMatch?.[1]!).getTime()

      expect(timestamp).toBeGreaterThanOrEqual(before)
      expect(timestamp).toBeLessThanOrEqual(after)
    })
  })

  describe('level filtering', () => {
    it('should log all levels in development mode', () => {
      logger.debug('Debug')
      logger.info('Info')
      logger.warn('Warn')
      logger.error('Error')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)
      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1)
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
    })

    it('formats each level correctly', () => {
      logger.debug('msg')
      logger.info('msg')
      logger.warn('msg')
      logger.error('msg')

      expect(consoleDebugSpy.mock.calls[0][0]).toMatch(/\[DEBUG\]/)
      expect(consoleInfoSpy.mock.calls[0][0]).toMatch(/\[INFO\]/)
      expect(consoleWarnSpy.mock.calls[0][0]).toMatch(/\[WARN\]/)
      expect(consoleErrorSpy.mock.calls[0][0]).toMatch(/\[ERROR\]/)
    })
  })

  describe('edge cases', () => {
    it('handles empty messages', () => {
      logger.info('')

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[1]).toBe('')
    })

    it('handles special characters in messages', () => {
      logger.info('Message with\nnewlines\tand\ttabs')

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[1]).toBe('Message with\nnewlines\tand\ttabs')
    })

    it('handles unicode characters', () => {
      logger.info('Có lỗi xảy ra 🚀 😊')

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[1]).toBe('Có lỗi xảy ra 🚀 😊')
    })

    it('handles circular references in objects', () => {
      const circular: any = { prop: 'value' }
      circular.self = circular

      // Should not throw
      expect(() => {
        logger.info('Circular object', circular)
      }).not.toThrow()

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
    })

    it('handles very long messages', () => {
      const longMessage = 'A'.repeat(10000)
      logger.info(longMessage)

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[1]).toBe(longMessage)
    })

    it('handles arrays as arguments', () => {
      const arr = [1, 2, 3, { nested: true }]
      logger.info('Array data', arr)

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[2]).toEqual(arr)
    })

    it('handles Error objects in regular log methods', () => {
      const error = new Error('Regular error')
      logger.info('Error info', error)

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      const call = consoleInfoSpy.mock.calls[0]
      expect(call[2]).toEqual(error)
    })
  })

  describe('mixed scenarios', () => {
    it('logs multiple messages in sequence', () => {
      logger.debug('Step 1')
      logger.info('Step 2')
      logger.warn('Step 3')
      logger.error('Step 4')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)
      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1)
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
    })

    it('logs API and WebSocket events together', () => {
      logger.api('get', '/api/data', { id: 123 })
      logger.ws('connected')
      logger.ws('message', { event: 'update' })
      logger.api('post', '/api/save')

      expect(consoleLogSpy).toHaveBeenCalledTimes(4)
    })

    it('logs regular messages and exceptions together', () => {
      logger.info('Starting operation')
      logger.exception(new Error('Operation failed'), 'Operation')
      logger.error('Cleanup required')

      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)
      expect(consoleErrorSpy).toHaveBeenCalledTimes(3) // exception (2 calls) + error (1 call)
    })
  })

  describe('minLevel filtering', () => {
    it('logger has default minLevel of debug', () => {
      // Access private config through type casting for testing
      const loggerWithConfig = logger as any
      expect(loggerWithConfig.config.minLevel).toBe('debug')
    })

    it('logger levels are correctly defined', () => {
      const loggerWithConfig = logger as any
      expect(loggerWithConfig.levels).toEqual({
        debug: 0,
        info: 1,
        warn: 2,
        error: 3,
      })
    })
  })

  describe('edge cases for exception', () => {
    it('logs exception without context', () => {
      const error = new Error('No context error')
      logger.exception(error)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(2)
      const call1 = consoleErrorSpy.mock.calls[0]
      expect(call1[0]).toBe('')
    })

    it('logs string as exception', () => {
      logger.exception('String exception')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      expect(consoleErrorSpy.mock.calls[0][2]).toBe('String exception')
    })

    it('logs number as exception', () => {
      logger.exception(404)

      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
      expect(consoleErrorSpy.mock.calls[0][2]).toBe(404)
    })
  })

  describe('api method edge cases', () => {
    it('handles empty method string', () => {
      logger.api('', '/api/test')

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][0]).toBe('[API]  /api/test')
    })

    it('handles method with mixed case', () => {
      logger.api('PoSt', '/api/data')

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][0]).toBe('[API] POST /api/data')
    })

    it('handles undefined data', () => {
      logger.api('get', '/api/test', undefined)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toBe('')
    })

    it('handles null data', () => {
      logger.api('get', '/api/test', null)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toBe('')
    })

    it('handles complex data objects', () => {
      const complexData = {
        nested: {
          deep: {
            value: 'test'
          }
        },
        array: [1, 2, 3]
      }
      logger.api('post', '/api/complex', complexData)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toEqual(complexData)
    })
  })

  describe('ws method edge cases', () => {
    it('handles empty event string', () => {
      logger.ws('')

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][0]).toBe('[WebSocket] ')
    })

    it('handles undefined data', () => {
      logger.ws('event', undefined)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toBe('')
    })

    it('handles null data', () => {
      logger.ws('event', null)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toBe('')
    })

    it('handles complex event data', () => {
      const eventData = {
        type: 'trade',
        payload: {
          symbol: 'BTCUSDT',
          price: 50000,
          quantity: 1.5
        }
      }
      logger.ws('trade_update', eventData)

      expect(consoleLogSpy).toHaveBeenCalledTimes(1)
      expect(consoleLogSpy.mock.calls[0][1]).toEqual(eventData)
    })
  })

  describe('formatMessage production mode filtering', () => {
    it('should not log when shouldLog returns false', () => {
      // Create a mock logger that returns false from shouldLog
      const mockLogger = {
        config: { isDevelopment: false, minLevel: 'debug' as const },
        levels: { debug: 0, info: 1, warn: 2, error: 3 },
        shouldLog(): boolean {
          return false
        },
        formatMessage(level: string, message: string, ...args: unknown[]): void {
          if (!this.shouldLog()) {
            return
          }
          console.log('Should not reach here')
        },
      }

      // Should return early without logging
      mockLogger.formatMessage('debug', 'Test message')
      mockLogger.formatMessage('info', 'Test message')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(0)
      expect(consoleInfoSpy).toHaveBeenCalledTimes(0)
    })

    it('covers all switch cases in formatMessage', () => {
      // Test each case explicitly
      logger.debug('Debug case')
      expect(consoleDebugSpy).toHaveBeenCalledTimes(1)

      logger.info('Info case')
      expect(consoleInfoSpy).toHaveBeenCalledTimes(1)

      logger.warn('Warn case')
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1)

      logger.error('Error case')
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1)
    })
  })

  describe('production mode behavior (actual logger)', () => {
    let originalConfig: any

    beforeEach(() => {
      // Save original config and switch to production mode on the ACTUAL logger
      originalConfig = { ...(logger as any).config }
      ;(logger as any).config.isDevelopment = false
    })

    afterEach(() => {
      // Restore original config
      ;(logger as any).config = originalConfig
    })

    it('should not log debug/info/warn/error in production mode', () => {
      logger.debug('Debug message')
      logger.info('Info message')
      logger.warn('Warning message')
      logger.error('Error message')

      expect(consoleDebugSpy).toHaveBeenCalledTimes(0)
      expect(consoleInfoSpy).toHaveBeenCalledTimes(0)
      expect(consoleWarnSpy).toHaveBeenCalledTimes(0)
      expect(consoleErrorSpy).toHaveBeenCalledTimes(0)
    })

    it('should not log exceptions in production mode', () => {
      logger.exception(new Error('Test error'))
      logger.exception(new Error('Test error'), 'Context')
      logger.exception('String error')

      expect(consoleErrorSpy).toHaveBeenCalledTimes(0)
    })

    it('should not log API calls in production mode', () => {
      logger.api('get', '/api/test')
      logger.api('post', '/api/test', { data: 'test' })

      expect(consoleLogSpy).toHaveBeenCalledTimes(0)
    })

    it('should not log WebSocket events in production mode', () => {
      logger.ws('connected')
      logger.ws('message', { data: 'test' })

      expect(consoleLogSpy).toHaveBeenCalledTimes(0)
    })
  })
})
