// Formatting utilities

export const formatCurrency = (value: number | null | undefined): string => {
  if (value == null || isNaN(value)) return '$0.00'
  
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value)
}

export const formatPercentage = (value: number | null | undefined): string => {
  if (value == null || isNaN(value)) return '0.00%'
  
  return `${value.toFixed(2)}%`
}

export const formatNumber = (value: number | null | undefined, decimals = 0): string => {
  if (value == null || isNaN(value)) return '0'
  
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(value)
}

export const formatCryptoAmount = (value: number | null | undefined, decimals = 8): string => {
  if (value == null || isNaN(value)) return '0'
  
  // For very small values, use more decimals
  if (value > 0 && value < 0.01) {
    return value.toFixed(decimals)
  }
  
  return formatNumber(value, Math.min(decimals, 4))
}

export const formatTimestamp = (
  timestamp: number | null | undefined,
  format: 'date' | 'time' | 'datetime' | 'relative' | 'default' = 'default',
  locale = 'en-US'
): string => {
  if (!timestamp) return 'N/A'
  
  const date = new Date(timestamp)
  
  switch (format) {
    case 'date':
      return date.toLocaleDateString(locale)
    case 'time':
      return date.toLocaleTimeString(locale)
    case 'datetime':
      return date.toLocaleString(locale)
    case 'relative':
      const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })
      const diff = (Date.now() - timestamp) / 1000 // in seconds
      
      if (diff < 60) return rtf.format(-Math.floor(diff), 'second')
      if (diff < 3600) return rtf.format(-Math.floor(diff / 60), 'minute')
      if (diff < 86400) return rtf.format(-Math.floor(diff / 3600), 'hour')
      return rtf.format(-Math.floor(diff / 86400), 'day')
    default:
      return date.toLocaleDateString(locale, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      })
  }
}