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

/**
 * Kline/Candlestick data from Binance
 */
export interface BinanceKline {
  openTime: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  closeTime: number;
}

// Kline cache configuration
const KLINE_CACHE_TTL_MS = 30000; // 30 seconds cache for klines

interface KlineCache {
  klines: BinanceKline[];
  timestamp: number;
}

// In-memory cache for klines
const klineCache: Map<string, KlineCache> = new Map();

/**
 * Fetch real candlestick/kline data from Binance API
 *
 * Binance Klines API returns:
 * [[openTime, open, high, low, close, volume, closeTime, quoteVolume, trades, buyBaseVolume, buyQuoteVolume, ignore], ...]
 *
 * @param symbol Trading pair symbol (e.g., "BTCUSDT")
 * @param interval Kline interval (e.g., "1m", "5m", "15m", "1h", "4h", "1d")
 * @param limit Number of klines to fetch (default 100, max 1000)
 * @returns Array of kline data
 */
export async function fetchBinanceKlines(
  symbol: string,
  interval: string = "15m",
  limit: number = 100
): Promise<BinanceKline[]> {
  const cacheKey = `${symbol}_${interval}_${limit}`;

  // Check cache first
  const cached = klineCache.get(cacheKey);
  const now = Date.now();

  if (cached && now - cached.timestamp < KLINE_CACHE_TTL_MS) {
    logger.debug(`[BinanceKlines] Cache hit for ${cacheKey}`);
    return cached.klines;
  }

  // Fetch from Binance Klines API
  try {
    const baseUrl = "https://api.binance.com/api/v3/klines";
    const url = `${baseUrl}?symbol=${symbol}&interval=${interval}&limit=${limit}`;

    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    // Parse Binance kline data format
    // Each kline is an array: [openTime, open, high, low, close, volume, closeTime, ...]
    const klines: BinanceKline[] = data.map((k: (string | number)[]) => ({
      openTime: Number(k[0]),
      open: parseFloat(String(k[1])),
      high: parseFloat(String(k[2])),
      low: parseFloat(String(k[3])),
      close: parseFloat(String(k[4])),
      volume: parseFloat(String(k[5])),
      closeTime: Number(k[6]),
    }));

    // Update cache
    klineCache.set(cacheKey, { klines, timestamp: now });
    logger.debug(`[BinanceKlines] Fetched ${klines.length} klines for ${symbol} ${interval}`);

    return klines;
  } catch (error) {
    logger.error(`[BinanceKlines] Failed to fetch ${symbol} ${interval}:`, error);

    // Return cached value if available (even if expired)
    if (cached) {
      logger.warn(`[BinanceKlines] Using stale cache for ${cacheKey}`);
      return cached.klines;
    }

    return [];
  }
}

/**
 * Get 24h ticker data from Binance
 */
export interface Binance24hTicker {
  symbol: string;
  priceChange: number;
  priceChangePercent: number;
  highPrice: number;
  lowPrice: number;
  lastPrice: number;
  volume: number;
}

/**
 * Fetch 24h ticker statistics from Binance
 *
 * @param symbol Trading pair symbol (e.g., "BTCUSDT")
 * @returns 24h ticker data
 */
export async function fetchBinance24hTicker(symbol: string): Promise<Binance24hTicker | null> {
  try {
    const url = `https://api.binance.com/api/v3/ticker/24hr?symbol=${symbol}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = await response.json();

    return {
      symbol: data.symbol,
      priceChange: parseFloat(data.priceChange),
      priceChangePercent: parseFloat(data.priceChangePercent),
      highPrice: parseFloat(data.highPrice),
      lowPrice: parseFloat(data.lowPrice),
      lastPrice: parseFloat(data.lastPrice),
      volume: parseFloat(data.volume),
    };
  } catch (error) {
    logger.error(`[Binance24hTicker] Failed to fetch ${symbol}:`, error);
    return null;
  }
}

/**
 * Clear kline cache
 */
export function clearKlineCache(): void {
  klineCache.clear();
}
