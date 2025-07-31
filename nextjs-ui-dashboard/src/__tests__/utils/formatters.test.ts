import { describe, it, expect } from 'vitest'
import {
  formatCurrency,
  formatPercentage,
  formatNumber,
  formatTimestamp,
  formatVolume,
  formatPnL,
} from '../../utils/formatters'

describe('formatters', () => {
  describe('formatCurrency', () => {
    it('formats USD currency', () => {
      expect(formatCurrency(1234.56)).toBe('$1,234.56')
      expect(formatCurrency(0)).toBe('$0.00')
      expect(formatCurrency(-500.75)).toBe('-$500.75')
    })

    it('formats with different decimal places', () => {
      expect(formatCurrency(1234.5678, 4)).toBe('$1,234.5678')
      expect(formatCurrency(1234.5678, 0)).toBe('$1,235')
    })

    it('formats with different currencies', () => {
      expect(formatCurrency(1234.56, 2, 'EUR')).toBe('€1,234.56')
      expect(formatCurrency(1234.56, 2, 'BTC')).toBe('₿1,234.56')
    })

    it('handles very large numbers', () => {
      expect(formatCurrency(1234567890.12)).toBe('$1,234,567,890.12')
    })

    it('handles very small numbers', () => {
      expect(formatCurrency(0.00001234, 8)).toBe('$0.00001234')
    })
  })

  describe('formatPercentage', () => {
    it('formats positive percentages', () => {
      expect(formatPercentage(0.1234)).toBe('+12.34%')
      expect(formatPercentage(0.05)).toBe('+5.00%')
    })

    it('formats negative percentages', () => {
      expect(formatPercentage(-0.1234)).toBe('-12.34%')
      expect(formatPercentage(-0.05)).toBe('-5.00%')
    })

    it('formats zero percentage', () => {
      expect(formatPercentage(0)).toBe('0.00%')
    })

    it('formats with different decimal places', () => {
      expect(formatPercentage(0.123456, 4)).toBe('+12.3456%')
      expect(formatPercentage(0.123456, 0)).toBe('+12%')
    })

    it('handles very small percentages', () => {
      expect(formatPercentage(0.000123, 6)).toBe('+0.012300%')
    })
  })

  describe('formatNumber', () => {
    it('formats numbers with commas', () => {
      expect(formatNumber(1234567)).toBe('1,234,567')
      expect(formatNumber(1234.567)).toBe('1,234.567')
    })

    it('formats with specified decimal places', () => {
      expect(formatNumber(1234.5678, 2)).toBe('1,234.57')
      expect(formatNumber(1234.1, 2)).toBe('1,234.10')
    })

    it('handles zero', () => {
      expect(formatNumber(0)).toBe('0')
      expect(formatNumber(0, 2)).toBe('0.00')
    })

    it('handles negative numbers', () => {
      expect(formatNumber(-1234.56)).toBe('-1,234.56')
    })
  })

  describe('formatTimestamp', () => {
    it('formats timestamp as date string', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp)
      expect(result).toMatch(/Jan 15, 2024/)
    })

    it('formats with custom format', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'MM/dd/yyyy HH:mm')
      expect(result).toBe('01/15/2024 14:30')
    })

    it('handles Date objects', () => {
      const date = new Date('2024-01-15T14:30:00Z')
      const result = formatTimestamp(date)
      expect(result).toMatch(/Jan 15, 2024/)
    })

    it('handles relative time', () => {
      const now = Date.now()
      const fiveMinutesAgo = now - 5 * 60 * 1000
      const result = formatTimestamp(fiveMinutesAgo, 'relative')
      expect(result).toBe('5 minutes ago')
    })
  })

  describe('formatVolume', () => {
    it('formats volume in K', () => {
      expect(formatVolume(1234)).toBe('1.23K')
      expect(formatVolume(5678)).toBe('5.68K')
    })

    it('formats volume in M', () => {
      expect(formatVolume(1234567)).toBe('1.23M')
      expect(formatVolume(5678901)).toBe('5.68M')
    })

    it('formats volume in B', () => {
      expect(formatVolume(1234567890)).toBe('1.23B')
      expect(formatVolume(5678901234)).toBe('5.68B')
    })

    it('formats small volumes without suffix', () => {
      expect(formatVolume(123)).toBe('123')
      expect(formatVolume(999)).toBe('999')
    })

    it('handles zero', () => {
      expect(formatVolume(0)).toBe('0')
    })

    it('formats with custom precision', () => {
      expect(formatVolume(1234567, 3)).toBe('1.235M')
      expect(formatVolume(1234567, 1)).toBe('1.2M')
    })
  })

  describe('formatPnL', () => {
    it('formats positive PnL', () => {
      expect(formatPnL(123.45)).toBe('+$123.45')
      expect(formatPnL(0.01)).toBe('+$0.01')
    })

    it('formats negative PnL', () => {
      expect(formatPnL(-123.45)).toBe('-$123.45')
      expect(formatPnL(-0.01)).toBe('-$0.01')
    })

    it('formats zero PnL', () => {
      expect(formatPnL(0)).toBe('$0.00')
    })

    it('includes percentage when provided', () => {
      expect(formatPnL(123.45, 0.05)).toBe('+$123.45 (+5.00%)')
      expect(formatPnL(-123.45, -0.05)).toBe('-$123.45 (-5.00%)')
    })

    it('formats with different currencies', () => {
      expect(formatPnL(123.45, undefined, 'BTC')).toBe('+₿123.45')
      expect(formatPnL(-123.45, -0.05, 'ETH')).toBe('-Ξ123.45 (-5.00%)')
    })

    it('handles very small PnL values', () => {
      expect(formatPnL(0.000123, undefined, 'BTC', 8)).toBe('+₿0.00012300')
    })
  })

  describe('edge cases', () => {
    it('handles null and undefined values', () => {
      expect(formatCurrency(null as unknown as number)).toBe('$0.00')
      expect(formatPercentage(undefined as unknown as number)).toBe('0.00%')
      expect(formatNumber(null as unknown as number)).toBe('0')
    })

    it('handles NaN values', () => {
      expect(formatCurrency(NaN)).toBe('$0.00')
      expect(formatPercentage(NaN)).toBe('0.00%')
      expect(formatNumber(NaN)).toBe('0')
    })

    it('handles Infinity values', () => {
      expect(formatCurrency(Infinity)).toBe('$∞')
      expect(formatCurrency(-Infinity)).toBe('-$∞')
    })

    it('handles very large timestamps', () => {
      const largeTimestamp = 9999999999999
      const result = formatTimestamp(largeTimestamp)
      expect(result).toMatch(/Nov.*2286/)
    })
  })

  describe('locale support', () => {
    it('formats numbers with different locales', () => {
      const result = formatCurrency(1234.56, 2, 'USD', 'de-DE')
      expect(result).toBe('1.234,56 $')
    })

    it('formats dates with different locales', () => {
      const timestamp = new Date('2024-01-15T14:30:00Z').getTime()
      const result = formatTimestamp(timestamp, 'default', 'de-DE')
      expect(result).toMatch(/15\\. Jan\\.? 2024/)
    })
  })
})