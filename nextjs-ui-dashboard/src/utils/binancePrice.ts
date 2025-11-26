/**
 * Binance Price Utility
 *
 * Centralized price fetching from Binance API with caching to avoid
 * duplicate API calls and rate limiting.
 */

import logger from "./logger";

// Cache configuration
const CACHE_TTL_MS = 5000; // 5 seconds cache
// Use environment variable with fallback to default Binance API
const BINANCE_API_URL = import.meta.env.VITE_BINANCE_API_URL || "https://api.binance.com/api/v3/ticker/price";

interface PriceCache {
  price: number;
  timestamp: number;
}

// In-memory cache for prices
const priceCache: Map<string, PriceCache> = new Map();

/**
 * Fetch price from Binance API for a single symbol
 *
 * Features:
 * - Caching with 5 second TTL to reduce API calls
 * - Error handling with fallback support
 * - Type-safe response parsing
 *
 * @param symbol Trading pair symbol (e.g., "BTCUSDT")
 * @param fallbackFn Optional fallback function if Binance fails
 * @returns Price as number, or 0 if all sources fail
 */
export async function fetchBinancePrice(
  symbol: string,
  fallbackFn?: () => Promise<number>
): Promise<number> {
  // Check cache first
  const cached = priceCache.get(symbol);
  const now = Date.now();

  if (cached && now - cached.timestamp < CACHE_TTL_MS) {
    logger.debug(`[BinancePrice] Cache hit for ${symbol}: ${cached.price}`);
    return cached.price;
  }

  // Fetch from Binance
  try {
    const response = await fetch(`${BINANCE_API_URL}?symbol=${symbol}`);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();
    const price = parseFloat(data.price);

    // Validate parsed price
    if (isNaN(price) || price <= 0) {
      throw new Error(`Invalid price data: ${data.price}`);
    }

    // Update cache
    priceCache.set(symbol, { price, timestamp: now });
    logger.debug(`[BinancePrice] Fetched ${symbol}: ${price}`);

    return price;
  } catch (error) {
    logger.error(`[BinancePrice] Failed to fetch ${symbol}:`, error);

    // Try fallback if provided
    if (fallbackFn) {
      try {
        const fallbackPrice = await fallbackFn();
        if (fallbackPrice > 0) {
          // Cache fallback result too
          priceCache.set(symbol, { price: fallbackPrice, timestamp: now });
          logger.debug(`[BinancePrice] Fallback for ${symbol}: ${fallbackPrice}`);
          return fallbackPrice;
        }
      } catch (fallbackError) {
        logger.error(`[BinancePrice] Fallback failed for ${symbol}:`, fallbackError);
      }
    }

    // Return cached value if available (even if expired)
    if (cached) {
      logger.warn(`[BinancePrice] Using stale cache for ${symbol}: ${cached.price}`);
      return cached.price;
    }

    return 0;
  }
}

/**
 * Fetch prices for multiple symbols in parallel
 *
 * @param symbols Array of trading pair symbols
 * @param fallbackFn Optional fallback function
 * @returns Map of symbol -> price
 */
export async function fetchMultipleBinancePrices(
  symbols: string[],
  fallbackFn?: (symbol: string) => Promise<number>
): Promise<Map<string, number>> {
  const results = new Map<string, number>();

  const pricePromises = symbols.map(async (symbol) => {
    const price = await fetchBinancePrice(
      symbol,
      fallbackFn ? () => fallbackFn(symbol) : undefined
    );
    results.set(symbol, price);
  });

  await Promise.all(pricePromises);
  return results;
}

/**
 * Clear the price cache (useful for testing)
 */
export function clearPriceCache(): void {
  priceCache.clear();
}

/**
 * Get cache statistics (useful for debugging)
 */
export function getPriceCacheStats(): { size: number; symbols: string[] } {
  return {
    size: priceCache.size,
    symbols: Array.from(priceCache.keys()),
  };
}
