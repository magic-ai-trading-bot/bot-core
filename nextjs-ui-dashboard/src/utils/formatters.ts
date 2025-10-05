// Formatting utilities

export const formatCurrency = (
  value: number | null | undefined,
  decimals = 2,
  currency = 'USD',
  locale = 'en-US'
): string => {
  if (value == null || isNaN(value)) return '$0.00'

  // Handle Infinity
  if (!isFinite(value)) {
    const currencySymbols: Record<string, string> = {
      USD: '$',
      EUR: '€',
      BTC: '₿',
      ETH: 'Ξ',
    }
    const symbol = currencySymbols[currency] || '$'
    return value > 0 ? `${symbol}∞` : `-${symbol}∞`
  }

  const currencySymbols: Record<string, string> = {
    USD: '$',
    EUR: '€',
    BTC: '₿',
    ETH: 'Ξ',
  }

  const symbol = currencySymbols[currency] || '$'
  const isNegative = value < 0
  const absValue = Math.abs(value)

  const formatted = new Intl.NumberFormat(locale, {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(absValue)

  // For German locale, put symbol after number
  if (locale.startsWith('de')) {
    return isNegative ? `-${formatted} ${symbol}` : `${formatted} ${symbol}`
  }

  return isNegative ? `-${symbol}${formatted}` : `${symbol}${formatted}`
}

export const formatPercentage = (
  value: number | null | undefined,
  decimals = 2
): string => {
  if (value == null || isNaN(value)) return '0.00%'

  // Convert decimal to percentage (0.1234 -> 12.34%)
  const percentage = value * 100
  const sign = percentage > 0 ? '+' : percentage < 0 ? '-' : ''
  const absValue = Math.abs(percentage)

  return `${sign}${absValue.toFixed(decimals)}%`
}

export const formatNumber = (
  value: number | null | undefined,
  decimals?: number
): string => {
  if (value == null || isNaN(value)) return '0'

  // If decimals not specified, use auto decimals based on value
  if (decimals === undefined) {
    // Count decimal places in original number
    const str = value.toString()
    const decimalIndex = str.indexOf('.')
    decimals = decimalIndex >= 0 ? str.length - decimalIndex - 1 : 0
  }

  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(value)
}

export const formatCryptoAmount = (
  value: number | null | undefined,
  decimals = 8
): string => {
  if (value == null || isNaN(value)) return '0'

  // For very small values, use more decimals
  if (value > 0 && value < 0.01) {
    return value.toFixed(decimals)
  }

  return formatNumber(value, Math.min(decimals, 4))
}

export const formatTimestamp = (
  timestamp: number | Date | null | undefined,
  format:
    | 'date'
    | 'time'
    | 'datetime'
    | 'relative'
    | 'default'
    | string = 'default',
  locale = 'en-US'
): string => {
  if (!timestamp) return 'N/A'

  const date = timestamp instanceof Date ? timestamp : new Date(timestamp)

  // Handle custom format strings like 'MM/dd/yyyy HH:mm'
  if (
    format !== 'date' &&
    format !== 'time' &&
    format !== 'datetime' &&
    format !== 'relative' &&
    format !== 'default'
  ) {
    // Simple custom format handler
    let result = format
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    const hours = String(date.getHours()).padStart(2, '0')
    const minutes = String(date.getMinutes()).padStart(2, '0')
    const seconds = String(date.getSeconds()).padStart(2, '0')

    result = result.replace('yyyy', String(year))
    result = result.replace('MM', month)
    result = result.replace('dd', day)
    result = result.replace('HH', hours)
    result = result.replace('mm', minutes)
    result = result.replace('ss', seconds)

    return result
  }

  switch (format) {
    case 'date':
      return date.toLocaleDateString(locale)
    case 'time':
      return date.toLocaleTimeString(locale)
    case 'datetime':
      return date.toLocaleString(locale)
    case 'relative': {
      const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })
      const diff =
        (Date.now() - (timestamp instanceof Date ? timestamp.getTime() : timestamp)) /
        1000 // in seconds

      if (diff < 60) return rtf.format(-Math.floor(diff), 'second')
      if (diff < 3600) return rtf.format(-Math.floor(diff / 60), 'minute')
      if (diff < 86400) return rtf.format(-Math.floor(diff / 3600), 'hour')
      return rtf.format(-Math.floor(diff / 86400), 'day')
    }
    default:
      return date.toLocaleDateString(locale, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      })
  }
}

export const formatVolume = (
  value: number | null | undefined,
  precision = 2
): string => {
  if (value == null || isNaN(value)) {
    return precision > 0 ? '0.' + '0'.repeat(precision) : '0'
  }

  const absValue = Math.abs(value)

  if (absValue >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(precision)}B`
  } else if (absValue >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(precision)}M`
  } else if (absValue >= 1_000) {
    return `${(value / 1_000).toFixed(precision)}K`
  }

  // For small numbers, only use decimals if needed
  return value % 1 === 0 ? String(value) : value.toFixed(precision)
}

export const formatPnL = (
  value: number | null | undefined,
  percentage?: number,
  currency = 'USD',
  decimals = 2
): string => {
  if (value == null || isNaN(value)) return '$0.00'

  const formatted = formatCurrency(value, decimals, currency)

  if (percentage !== undefined && percentage !== null && !isNaN(percentage)) {
    const pctFormatted = formatPercentage(percentage, decimals) // Percentage already in decimal form
    const sign = value > 0 ? '+' : ''
    return `${sign}${formatted} (${pctFormatted})`
  }

  return value > 0 ? `+${formatted}` : value === 0 ? formatted : formatted
}
