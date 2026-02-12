/**
 * Formatters Utilities - Functional Tests
 * Target: Boost coverage from 87.5% to 95%+
 * Focus: ALL edge cases, locales, special values, boundary conditions
 */

import { describe, it, expect } from 'vitest'
import {
  formatCurrency,
  formatPercentage,
  formatNumber,
  formatCryptoAmount,
  formatTimestamp,
  formatVolume,
  formatPnL,
} from '@/utils/formatters'

describe('formatters - Functional Tests', () => {
  describe('formatCurrency - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatCurrency(null)).toBe('$0.00')
      expect(formatCurrency(undefined)).toBe('$0.00')
      expect(formatCurrency(NaN)).toBe('$0.00')
    })

    it('should handle positive infinity with all currencies', () => {
      expect(formatCurrency(Infinity)).toBe('$∞')
      expect(formatCurrency(Infinity, 2, 'USD')).toBe('$∞')
      expect(formatCurrency(Infinity, 2, 'EUR')).toBe('€∞')
      expect(formatCurrency(Infinity, 2, 'BTC')).toBe('₿∞')
      expect(formatCurrency(Infinity, 2, 'ETH')).toBe('Ξ∞')
    })

    it('should handle negative infinity with all currencies', () => {
      expect(formatCurrency(-Infinity)).toBe('-$∞')
      expect(formatCurrency(-Infinity, 2, 'USD')).toBe('-$∞')
      expect(formatCurrency(-Infinity, 2, 'EUR')).toBe('-€∞')
      expect(formatCurrency(-Infinity, 2, 'BTC')).toBe('-₿∞')
      expect(formatCurrency(-Infinity, 2, 'ETH')).toBe('-Ξ∞')
    })

    it('should fallback to $ for unknown currencies with infinity', () => {
      expect(formatCurrency(Infinity, 2, 'JPY')).toBe('$∞')
      expect(formatCurrency(Infinity, 2, 'UNKNOWN')).toBe('$∞')
      expect(formatCurrency(-Infinity, 2, 'CHF')).toBe('-$∞')
    })

    it('should handle German locale with positive values', () => {
      const result = formatCurrency(1234.56, 2, 'EUR', 'de-DE')
      expect(result).toContain('1.234,56')
      expect(result).toContain('€')
    })

    it('should handle German locale with negative values', () => {
      const result = formatCurrency(-1234.56, 2, 'EUR', 'de-DE')
      expect(result).toContain('-')
      expect(result).toContain('1.234,56')
      expect(result).toContain('€')
    })

    it('should handle German locale with zero', () => {
      const result = formatCurrency(0, 2, 'EUR', 'de-DE')
      expect(result).toContain('0,00')
      expect(result).toContain('€')
    })

    it('should handle non-German locale (en-US)', () => {
      const result = formatCurrency(1234.56, 2, 'USD', 'en-US')
      expect(result).toBe('$1,234.56')
    })

    it('should handle unknown currency fallback', () => {
      expect(formatCurrency(100, 2, 'UNKNOWN')).toBe('$100.00')
      expect(formatCurrency(-100, 2, 'FAKE')).toBe('-$100.00')
    })

    it('should handle very large decimals', () => {
      expect(formatCurrency(123.456789, 8)).toBe('$123.45678900')
    })

    it('should handle zero decimals', () => {
      expect(formatCurrency(1234.99, 0)).toBe('$1,235')
    })

    it('should handle very small values', () => {
      expect(formatCurrency(0.000001, 8)).toBe('$0.00000100')
    })

    it('should handle very large values', () => {
      expect(formatCurrency(999999999999.99, 2)).toBe('$999,999,999,999.99')
    })

    it('should handle negative zero', () => {
      expect(formatCurrency(-0)).toBe('$0.00')
    })
  })

  describe('formatPercentage - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatPercentage(null)).toBe('0.00%')
      expect(formatPercentage(undefined)).toBe('0.00%')
      expect(formatPercentage(NaN)).toBe('0.00%')
    })

    it('should handle exact zero with correct sign', () => {
      expect(formatPercentage(0)).toBe('0.00%')
      expect(formatPercentage(-0)).toBe('0.00%')
    })

    it('should handle positive values with + sign', () => {
      expect(formatPercentage(0.1234)).toBe('+12.34%')
      expect(formatPercentage(0.01)).toBe('+1.00%')
    })

    it('should handle negative values with - sign', () => {
      expect(formatPercentage(-0.1234)).toBe('-12.34%')
      expect(formatPercentage(-0.01)).toBe('-1.00%')
    })

    it('should handle very small positive percentage', () => {
      expect(formatPercentage(0.000001, 6)).toBe('+0.000100%')
    })

    it('should handle very small negative percentage', () => {
      expect(formatPercentage(-0.000001, 6)).toBe('-0.000100%')
    })

    it('should handle very large positive percentage', () => {
      expect(formatPercentage(10, 2)).toBe('+1000.00%')
    })

    it('should handle very large negative percentage', () => {
      expect(formatPercentage(-10, 2)).toBe('-1000.00%')
    })

    it('should handle zero decimals', () => {
      expect(formatPercentage(0.1234, 0)).toBe('+12%')
    })

    it('should handle many decimals', () => {
      expect(formatPercentage(0.123456789, 8)).toBe('+12.34567890%')
    })
  })

  describe('formatNumber - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatNumber(null)).toBe('0')
      expect(formatNumber(undefined)).toBe('0')
      expect(formatNumber(NaN)).toBe('0')
    })

    it('should auto-detect decimals for integers', () => {
      expect(formatNumber(1234)).toBe('1,234')
      expect(formatNumber(0)).toBe('0')
      expect(formatNumber(-5678)).toBe('-5,678')
    })

    it('should auto-detect decimals for decimals', () => {
      expect(formatNumber(1234.5)).toBe('1,234.5')
      expect(formatNumber(1234.56)).toBe('1,234.56')
      expect(formatNumber(1234.567)).toBe('1,234.567')
    })

    it('should auto-detect very long decimals', () => {
      const result = formatNumber(1234.123456789012345)
      expect(result).toContain('1,234.')
      expect(result.length).toBeGreaterThan(10)
    })

    it('should handle specified decimals', () => {
      expect(formatNumber(1234.5678, 2)).toBe('1,234.57')
      expect(formatNumber(1234.5678, 4)).toBe('1,234.5678')
    })

    it('should handle zero decimals (rounding)', () => {
      expect(formatNumber(1234.4, 0)).toBe('1,234')
      expect(formatNumber(1234.5, 0)).toBe('1,235')
      expect(formatNumber(1234.9, 0)).toBe('1,235')
    })

    it('should handle very small numbers', () => {
      expect(formatNumber(0.000123, 6)).toBe('0.000123')
    })

    it('should handle very large numbers', () => {
      expect(formatNumber(9999999999, 0)).toBe('9,999,999,999')
    })

    it('should handle negative numbers', () => {
      expect(formatNumber(-1234.56, 2)).toBe('-1,234.56')
    })
  })

  describe('formatCryptoAmount - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatCryptoAmount(null)).toBe('0')
      expect(formatCryptoAmount(undefined)).toBe('0')
      expect(formatCryptoAmount(NaN)).toBe('0')
    })

    it('should use 8 decimals for very small values', () => {
      expect(formatCryptoAmount(0.00000001, 8)).toBe('0.00000001')
      expect(formatCryptoAmount(0.00000123, 8)).toBe('0.00000123')
    })

    it('should use 8 decimals for values below 0.01', () => {
      expect(formatCryptoAmount(0.001, 8)).toBe('0.00100000')
      expect(formatCryptoAmount(0.009, 8)).toBe('0.00900000')
    })

    it('should handle boundary at 0.01', () => {
      expect(formatCryptoAmount(0.009999, 8)).toContain('0.009')
      expect(formatCryptoAmount(0.01, 8)).toContain('0.01')
    })

    it('should cap decimals at 4 for values >= 0.01', () => {
      const result = formatCryptoAmount(1.123456789, 8)
      expect(result).toBe('1.1235')
    })

    it('should handle large crypto amounts', () => {
      const result = formatCryptoAmount(21000000.123456, 8)
      expect(result).toContain('21,000,000.1235')
    })

    it('should handle zero value', () => {
      const result = formatCryptoAmount(0, 8)
      expect(result).toMatch(/^0(\.0+)?$/)
    })

    it('should handle custom decimals parameter', () => {
      expect(formatCryptoAmount(0.001, 4)).toContain('0.001')
      expect(formatCryptoAmount(0.001, 2)).toContain('0.00')
    })

    it('should respect min(decimals, 4) cap', () => {
      const result = formatCryptoAmount(100, 8)
      expect(result).toContain('100')
    })
  })

  describe('formatTimestamp - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined cases', () => {
      expect(formatTimestamp(null)).toBe('N/A')
      expect(formatTimestamp(undefined)).toBe('N/A')
      expect(formatTimestamp(0)).toBe('N/A')
    })

    it('should handle Date object input', () => {
      const date = new Date('2024-01-15T14:30:45Z')
      const result = formatTimestamp(date)
      expect(result).toMatch(/Jan 15, 2024/)
    })

    it('should handle timestamp number input', () => {
      const timestamp = new Date('2024-01-15T14:30:45Z').getTime()
      const result = formatTimestamp(timestamp)
      expect(result).toMatch(/Jan 15, 2024/)
    })

    it('should handle date format', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'date')
      expect(result).toMatch(/\d{1,2}\/\d{1,2}\/\d{4}/)
    })

    it('should handle time format', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'time')
      expect(result).toMatch(/\d{1,2}:\d{2}/)
    })

    it('should handle datetime format', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'datetime')
      expect(result).toContain('2024')
    })

    it('should handle default format', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'default')
      expect(result).toMatch(/Jan 15, 2024/)
    })

    it('should handle custom format with all placeholders', () => {
      const timestamp = new Date('2024-06-15T14:30:45Z').getTime()
      const result = formatTimestamp(timestamp, 'yyyy-MM-dd HH:mm:ss')
      expect(result).toContain('2024')
      expect(result).toContain('06')
      expect(result).toContain('15')
      expect(result).toMatch(/\d{2}:\d{2}:\d{2}/)
    })

    it('should replace yyyy placeholder', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      expect(formatTimestamp(timestamp, 'yyyy')).toBe('2024')
    })

    it('should replace MM placeholder with zero padding', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      expect(formatTimestamp(timestamp, 'MM')).toBe('01')

      const timestamp2 = new Date('2024-12-15T14:30:00Z').getTime()
      expect(formatTimestamp(timestamp2, 'MM')).toBe('12')
    })

    it('should replace dd placeholder with zero padding', () => {
      const timestamp = new Date('2024-01-05T14:30:00Z').getTime()
      expect(formatTimestamp(timestamp, 'dd')).toBe('05')

      const timestamp2 = new Date('2024-01-25T14:30:00Z').getTime()
      expect(formatTimestamp(timestamp2, 'dd')).toBe('25')
    })

    it('should replace HH placeholder', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'HH')
      expect(result).toMatch(/\d{2}/)
    })

    it('should replace mm placeholder', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'mm')
      expect(result).toMatch(/\d{2}/)
    })

    it('should replace ss placeholder', () => {
      const timestamp = new Date('2024-01-15T14:30:45Z').getTime()
      const result = formatTimestamp(timestamp, 'ss')
      expect(result).toMatch(/\d{2}/)
    })

    it('should handle relative format - seconds ago', () => {
      const now = Date.now()
      const timestamp = now - 30000 // 30 seconds ago
      const result = formatTimestamp(timestamp, 'relative')
      expect(result).toContain('second')
    })

    it('should handle relative format - minutes ago', () => {
      const now = Date.now()
      const timestamp = now - 180000 // 3 minutes ago
      const result = formatTimestamp(timestamp, 'relative')
      expect(result).toContain('minute')
    })

    it('should handle relative format - hours ago', () => {
      const now = Date.now()
      const timestamp = now - 7200000 // 2 hours ago
      const result = formatTimestamp(timestamp, 'relative')
      expect(result).toContain('hour')
    })

    it('should handle relative format - days ago', () => {
      const now = Date.now()
      const timestamp = now - 172800000 // 2 days ago
      const result = formatTimestamp(timestamp, 'relative')
      expect(result).toContain('day')
    })

    it('should handle different locales', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'date', 'de-DE')
      expect(result).toContain('2024')
    })
  })

  describe('formatVolume - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatVolume(null)).toBe('0.00')
      expect(formatVolume(undefined)).toBe('0.00')
      expect(formatVolume(NaN)).toBe('0.00')
    })

    it('should handle zero precision for null', () => {
      expect(formatVolume(null, 0)).toBe('0')
      expect(formatVolume(undefined, 0)).toBe('0')
      expect(formatVolume(NaN, 0)).toBe('0')
    })

    it('should handle high precision for null', () => {
      expect(formatVolume(null, 5)).toBe('0.00000')
      expect(formatVolume(undefined, 3)).toBe('0.000')
    })

    it('should format billions correctly', () => {
      expect(formatVolume(1234567890, 2)).toBe('1.23B')
      expect(formatVolume(9876543210, 2)).toBe('9.88B')
    })

    it('should format negative billions correctly', () => {
      expect(formatVolume(-1234567890, 2)).toBe('-1.23B')
    })

    it('should format millions correctly', () => {
      expect(formatVolume(1234567, 2)).toBe('1.23M')
      expect(formatVolume(9876543, 2)).toBe('9.88M')
    })

    it('should format negative millions correctly', () => {
      expect(formatVolume(-1234567, 2)).toBe('-1.23M')
    })

    it('should format thousands correctly', () => {
      expect(formatVolume(1234, 2)).toBe('1.23K')
      expect(formatVolume(9876, 2)).toBe('9.88K')
    })

    it('should format exact thousands with zero precision', () => {
      expect(formatVolume(1000, 0)).toBe('1K')
      expect(formatVolume(5000, 0)).toBe('5K')
    })

    it('should return integer string for whole numbers < 1000', () => {
      expect(formatVolume(100, 2)).toBe('100')
      expect(formatVolume(999, 2)).toBe('999')
      expect(formatVolume(1, 2)).toBe('1')
    })

    it('should return decimal string for non-whole numbers < 1000', () => {
      expect(formatVolume(123.45, 2)).toBe('123.45')
      expect(formatVolume(999.99, 2)).toBe('999.99')
    })

    it('should handle negative whole numbers < 1000', () => {
      expect(formatVolume(-100, 2)).toBe('-100')
      expect(formatVolume(-999, 2)).toBe('-999')
    })

    it('should handle negative decimal numbers < 1000', () => {
      expect(formatVolume(-123.45, 2)).toBe('-123.45')
    })

    it('should handle zero value', () => {
      expect(formatVolume(0, 2)).toBe('0')
      expect(formatVolume(0, 0)).toBe('0')
    })

    it('should handle custom precision', () => {
      expect(formatVolume(1234567, 3)).toBe('1.235M')
      expect(formatVolume(1234567, 1)).toBe('1.2M')
      expect(formatVolume(1234567, 0)).toBe('1M')
    })

    it('should handle boundary values', () => {
      expect(formatVolume(999, 2)).toBe('999')
      expect(formatVolume(1000, 2)).toBe('1.00K')
      expect(formatVolume(999999, 2)).toBe('1000.00K')
      expect(formatVolume(1000000, 2)).toBe('1.00M')
      expect(formatVolume(999999999, 2)).toBe('1000.00M')
      expect(formatVolume(1000000000, 2)).toBe('1.00B')
    })
  })

  describe('formatPnL - Comprehensive Edge Cases', () => {
    it('should handle all null/undefined/NaN cases', () => {
      expect(formatPnL(null)).toBe('$0.00')
      expect(formatPnL(undefined)).toBe('$0.00')
      expect(formatPnL(NaN)).toBe('$0.00')
    })

    it('should format positive PnL with + sign', () => {
      expect(formatPnL(100)).toBe('+$100.00')
      expect(formatPnL(0.01)).toBe('+$0.01')
    })

    it('should format negative PnL with - sign', () => {
      expect(formatPnL(-100)).toBe('-$100.00')
      expect(formatPnL(-0.01)).toBe('-$0.01')
    })

    it('should format zero PnL without sign', () => {
      expect(formatPnL(0)).toBe('$0.00')
      expect(formatPnL(-0)).toBe('$0.00')
    })

    it('should include percentage when provided and valid', () => {
      expect(formatPnL(100, 0.05)).toContain('+$100.00')
      expect(formatPnL(100, 0.05)).toContain('(+5.00%)')
    })

    it('should include negative percentage', () => {
      expect(formatPnL(-100, -0.05)).toContain('-$100.00')
      expect(formatPnL(-100, -0.05)).toContain('(-5.00%)')
    })

    it('should handle positive value with negative percentage', () => {
      expect(formatPnL(100, -0.05)).toContain('+$100.00')
      expect(formatPnL(100, -0.05)).toContain('(-5.00%)')
    })

    it('should handle negative value with positive percentage', () => {
      expect(formatPnL(-100, 0.05)).toContain('-$100.00')
      expect(formatPnL(-100, 0.05)).toContain('(+5.00%)')
    })

    it('should handle zero with zero percentage', () => {
      expect(formatPnL(0, 0)).toContain('$0.00')
      expect(formatPnL(0, 0)).toContain('0.00%')
    })

    it('should ignore null percentage', () => {
      expect(formatPnL(100, null as any)).toBe('+$100.00')
      expect(formatPnL(-100, null as any)).toBe('-$100.00')
    })

    it('should ignore undefined percentage', () => {
      expect(formatPnL(100, undefined)).toBe('+$100.00')
      expect(formatPnL(-100, undefined)).toBe('-$100.00')
    })

    it('should ignore NaN percentage', () => {
      expect(formatPnL(100, NaN)).toBe('+$100.00')
      expect(formatPnL(-100, NaN)).toBe('-$100.00')
    })

    it('should handle different currencies', () => {
      expect(formatPnL(100, 0.05, 'EUR')).toContain('€')
      expect(formatPnL(100, 0.05, 'BTC')).toContain('₿')
      expect(formatPnL(100, 0.05, 'ETH')).toContain('Ξ')
    })

    it('should handle custom decimals', () => {
      expect(formatPnL(100.123, 0.051, 'USD', 4)).toContain('100.1230')
      expect(formatPnL(100.999, 0.051, 'USD', 0)).toContain('101')
    })

    it('should handle very small values', () => {
      expect(formatPnL(0.001, 0.0001, 'USD', 4)).toContain('0.0010')
    })

    it('should handle very large values', () => {
      expect(formatPnL(999999.99, 1.5, 'USD', 2)).toContain('999,999.99')
    })

    it('should handle negative zero', () => {
      expect(formatPnL(-0)).toBe('$0.00')
    })
  })

  describe('Locale-Specific Tests', () => {
    it('should format currency with en-US locale', () => {
      expect(formatCurrency(1234.56, 2, 'USD', 'en-US')).toBe('$1,234.56')
    })

    it('should format currency with de-DE locale', () => {
      const result = formatCurrency(1234.56, 2, 'EUR', 'de-DE')
      expect(result).toContain('1.234,56')
      expect(result).toContain('€')
    })

    it('should handle de-AT locale (German Austria)', () => {
      const result = formatCurrency(1234.56, 2, 'EUR', 'de-AT')
      expect(result).toContain('€')
    })

    it('should handle de-CH locale (German Switzerland)', () => {
      const result = formatCurrency(1234.56, 2, 'EUR', 'de-CH')
      expect(result).toContain('€')
    })

    it('should handle timestamp with different locales', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const enResult = formatTimestamp(timestamp, 'date', 'en-US')
      const deResult = formatTimestamp(timestamp, 'date', 'de-DE')

      expect(enResult).toBeTruthy()
      expect(deResult).toBeTruthy()
    })
  })

  describe('Boundary Value Tests', () => {
    it('should handle Number.MAX_SAFE_INTEGER', () => {
      expect(formatNumber(Number.MAX_SAFE_INTEGER)).toContain(',')
    })

    it('should handle Number.MIN_SAFE_INTEGER', () => {
      expect(formatNumber(Number.MIN_SAFE_INTEGER)).toContain(',')
    })

    it('should handle very small positive number', () => {
      // Use reasonable decimal places (max 20 for Intl.NumberFormat)
      expect(formatNumber(Number.MIN_VALUE, 10)).toBeTruthy()
    })

    it('should handle Number.EPSILON', () => {
      expect(formatNumber(Number.EPSILON, 20)).toBeTruthy()
    })
  })

  describe('Special Character Tests', () => {
    it('should handle all currency symbols correctly', () => {
      expect(formatCurrency(100, 2, 'USD')).toContain('$')
      expect(formatCurrency(100, 2, 'EUR')).toContain('€')
      expect(formatCurrency(100, 2, 'BTC')).toContain('₿')
      expect(formatCurrency(100, 2, 'ETH')).toContain('Ξ')
    })

    it('should handle infinity symbol correctly', () => {
      expect(formatCurrency(Infinity)).toContain('∞')
      expect(formatCurrency(-Infinity)).toContain('∞')
    })

    it('should handle percentage symbol correctly', () => {
      expect(formatPercentage(0.5)).toContain('%')
      expect(formatPercentage(-0.5)).toContain('%')
    })

    it('should handle volume suffixes correctly', () => {
      expect(formatVolume(1234567890)).toContain('B')
      expect(formatVolume(1234567)).toContain('M')
      expect(formatVolume(1234)).toContain('K')
    })
  })
})
