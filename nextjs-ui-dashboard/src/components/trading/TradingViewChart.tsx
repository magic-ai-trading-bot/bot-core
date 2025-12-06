/**
 * Trading View Chart
 *
 * Professional candlestick chart with real-time Binance data.
 * Features: MA lines, Volume bars, Hover tooltip, like Binance.
 *
 * @spec:FR-TRADING-015 - Real-time Price Display
 * @ref:specs/02-design/2.3-api/API-RUST-CORE.md
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { fetchBinanceKlines, fetchBinance24hTicker, BinanceKline } from '@/utils/binancePrice';
import logger from '@/utils/logger';

interface ChartDisplayData {
  time: string;
  timestamp: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

// Binance-style colors
const colors = {
  bg: {
    primary: '#0B0E11',
    secondary: '#1E2329',
    tooltip: '#1E2329',
  },
  border: {
    default: '#2B3139',
  },
  text: {
    primary: '#EAECEF',
    secondary: '#848E9C',
    tertiary: '#5E6673',
  },
  candle: {
    up: '#0ECB81',
    down: '#F6465D',
  },
  ma: {
    ma7: '#F0B90B',   // Yellow - MA(7)
    ma25: '#E377C2',  // Pink - MA(25)
    ma99: '#7B61FF',  // Purple - MA(99)
  },
  volume: {
    up: 'rgba(14, 203, 129, 0.5)',
    down: 'rgba(246, 70, 93, 0.5)',
  },
  grid: '#2B3139',
  crosshair: '#848E9C',
};

interface TradingViewChartProps {
  symbol?: string;
  timeframe?: string;
  showControls?: boolean;
}

// Calculate Simple Moving Average
function calculateSMA(data: ChartDisplayData[], period: number): (number | null)[] {
  const sma: (number | null)[] = [];
  for (let i = 0; i < data.length; i++) {
    if (i < period - 1) {
      sma.push(null);
    } else {
      let sum = 0;
      for (let j = 0; j < period; j++) {
        sum += data[i - j].close;
      }
      sma.push(sum / period);
    }
  }
  return sma;
}

// Available timeframes
const TIMEFRAMES = ['1m', '5m', '15m', '30m', '1H', '4H', '1D', '1W'];

// Map timeframe to Binance interval format (moved outside component)
const getBinanceInterval = (tf: string): string => {
  const normalizedTf = tf.toLowerCase();
  const intervalMap: Record<string, string> = {
    '1m': '1m',
    '5m': '5m',
    '15m': '15m',
    '30m': '30m',
    '1h': '1h',
    '4h': '4h',
    '1d': '1d',
    '1w': '1w',
  };
  return intervalMap[normalizedTf] || '15m';
};

export function TradingViewChart({
  symbol: propSymbol = 'BTCUSDT',
  timeframe: propTimeframe = '15m',
  showControls = true,
}: TradingViewChartProps) {
  const [symbol, setSymbol] = useState(propSymbol);
  const [timeRange, setTimeRange] = useState(propTimeframe);

  // Sync with props when they change
  useEffect(() => {
    setSymbol(propSymbol);
  }, [propSymbol]);

  useEffect(() => {
    setTimeRange(propTimeframe);
  }, [propTimeframe]);
  const [data, setData] = useState<ChartDisplayData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPrice, setCurrentPrice] = useState<number>(0);
  const [priceChange, setPriceChange] = useState<number>(0);
  const [high24h, setHigh24h] = useState<number>(0);
  const [low24h, setLow24h] = useState<number>(0);
  const [hoveredCandle, setHoveredCandle] = useState<ChartDisplayData | null>(null);
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(null);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const volumeCanvasRef = useRef<HTMLCanvasElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  // MA values for header display
  const [ma7, setMa7] = useState<number>(0);
  const [ma25, setMa25] = useState<number>(0);
  const [ma99, setMa99] = useState<number>(0);

  // Fetch REAL candlestick data directly from Binance API
  const fetchChartData = useCallback(async () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    try {
      setIsLoading(true);
      setError(null);

      const binanceInterval = getBinanceInterval(timeRange);

      const [klines, ticker] = await Promise.all([
        fetchBinanceKlines(symbol, binanceInterval, 100),
        fetchBinance24hTicker(symbol),
      ]);

      if (klines && klines.length > 0) {
        const displayData: ChartDisplayData[] = klines.map((kline: BinanceKline) => ({
          time: new Date(kline.openTime).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          }),
          timestamp: kline.openTime,
          open: kline.open,
          high: kline.high,
          low: kline.low,
          close: kline.close,
          volume: kline.volume,
        }));

        setData(displayData);

        const lastKline = klines[klines.length - 1];
        setCurrentPrice(lastKline.close);

        // Calculate MAs for header
        const ma7Values = calculateSMA(displayData, 7);
        const ma25Values = calculateSMA(displayData, 25);
        const ma99Values = calculateSMA(displayData, 99);

        const lastMa7 = ma7Values[ma7Values.length - 1];
        const lastMa25 = ma25Values[ma25Values.length - 1];
        const lastMa99 = ma99Values[ma99Values.length - 1];

        setMa7(lastMa7 || 0);
        setMa25(lastMa25 || 0);
        setMa99(lastMa99 || 0);

        if (ticker) {
          setPriceChange(ticker.priceChangePercent);
          setHigh24h(ticker.highPrice);
          setLow24h(ticker.lowPrice);
        } else {
          const highs = klines.map((k: BinanceKline) => k.high);
          const lows = klines.map((k: BinanceKline) => k.low);
          setHigh24h(Math.max(...highs));
          setLow24h(Math.min(...lows));
          const firstPrice = klines[0].open;
          const lastPrice = lastKline.close;
          const change = ((lastPrice - firstPrice) / firstPrice) * 100;
          setPriceChange(change);
        }

        logger.debug(`[TradingViewChart] Loaded ${klines.length} real klines for ${symbol}`);
      } else {
        setError('No kline data available');
      }
    } catch (err) {
      if (err instanceof Error && err.name === 'AbortError') {
        return;
      }
      logger.error('Failed to fetch Binance klines:', err);
      setError('Unable to connect to Binance');
    } finally {
      setIsLoading(false);
    }
  }, [symbol, timeRange]);

  useEffect(() => {
    fetchChartData();
    // Reduced polling interval since we have WebSocket for real-time updates
    const interval = setInterval(fetchChartData, 30000);
    return () => {
      clearInterval(interval);
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [fetchChartData]);

  // WebSocket for real-time kline updates
  useEffect(() => {
    // Close existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    const binanceInterval = getBinanceInterval(timeRange);
    const streamName = `${symbol.toLowerCase()}@kline_${binanceInterval}`;
    const wsUrl = `wss://stream.binance.com:9443/ws/${streamName}`;

    logger.debug(`[TradingViewChart] Connecting to WebSocket: ${wsUrl}`);

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      logger.debug(`[TradingViewChart] WebSocket connected for ${symbol} ${timeRange}`);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.e === 'kline') {
          const kline = message.k;
          const newCandle: ChartDisplayData = {
            time: new Date(kline.t).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
            }),
            timestamp: kline.t,
            open: parseFloat(kline.o),
            high: parseFloat(kline.h),
            low: parseFloat(kline.l),
            close: parseFloat(kline.c),
            volume: parseFloat(kline.v),
          };

          // Update current price immediately
          setCurrentPrice(newCandle.close);

          // Update data array - replace last candle or add new one
          setData((prevData) => {
            if (prevData.length === 0) return prevData;

            const lastCandle = prevData[prevData.length - 1];

            // If same candle (same timestamp), update it
            if (lastCandle.timestamp === newCandle.timestamp) {
              const newData = [...prevData];
              newData[newData.length - 1] = newCandle;
              return newData;
            }

            // If new candle (kline closed), add it and remove oldest
            if (kline.x) { // x = is kline closed
              const newData = [...prevData.slice(1), newCandle];
              return newData;
            }

            // Otherwise just update the last candle
            const newData = [...prevData];
            newData[newData.length - 1] = newCandle;
            return newData;
          });
        }
      } catch (err) {
        logger.error('[TradingViewChart] WebSocket message parse error:', err);
      }
    };

    ws.onerror = (error) => {
      logger.error('[TradingViewChart] WebSocket error:', error);
    };

    ws.onclose = () => {
      logger.debug(`[TradingViewChart] WebSocket closed for ${symbol} ${timeRange}`);
    };

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [symbol, timeRange]);

  // Draw main chart with candles and MA lines
  useEffect(() => {
    if (!canvasRef.current || data.length === 0) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;
    const padding = { top: 10, right: 70, bottom: 25, left: 10 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // Clear canvas
    ctx.fillStyle = colors.bg.primary;
    ctx.fillRect(0, 0, width, height);

    // Calculate price range
    const allPrices = data.flatMap((d) => [d.high, d.low]);
    const minPrice = Math.min(...allPrices);
    const maxPrice = Math.max(...allPrices);
    const priceRange = maxPrice - minPrice || 1;
    const paddedMin = minPrice - priceRange * 0.05;
    const paddedMax = maxPrice + priceRange * 0.05;
    const paddedRange = paddedMax - paddedMin;

    const priceToY = (price: number) =>
      padding.top + chartHeight - ((price - paddedMin) / paddedRange) * chartHeight;

    // Draw grid lines
    ctx.strokeStyle = colors.grid;
    ctx.lineWidth = 0.5;
    const gridLines = 6;
    for (let i = 0; i <= gridLines; i++) {
      const y = padding.top + (chartHeight / gridLines) * i;
      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(width - padding.right, y);
      ctx.stroke();

      // Price labels on right
      const price = paddedMax - (paddedRange / gridLines) * i;
      ctx.fillStyle = colors.text.tertiary;
      ctx.font = '10px monospace';
      ctx.textAlign = 'left';
      ctx.fillText(price.toFixed(2), width - padding.right + 5, y + 3);
    }

    const candleSpacing = chartWidth / data.length;
    const candleWidth = Math.max(3, candleSpacing * 0.7);

    // Draw candles
    data.forEach((candle, i) => {
      const x = padding.left + candleSpacing * i + candleSpacing / 2;
      const isUp = candle.close >= candle.open;
      const color = isUp ? colors.candle.up : colors.candle.down;

      // Wick
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, priceToY(candle.high));
      ctx.lineTo(x, priceToY(candle.low));
      ctx.stroke();

      // Body
      const bodyTop = priceToY(Math.max(candle.open, candle.close));
      const bodyBottom = priceToY(Math.min(candle.open, candle.close));
      const bodyHeight = Math.max(1, bodyBottom - bodyTop);

      ctx.fillStyle = color;
      ctx.fillRect(x - candleWidth / 2, bodyTop, candleWidth, bodyHeight);
    });

    // Draw MA lines
    const ma7Values = calculateSMA(data, 7);
    const ma25Values = calculateSMA(data, 25);
    const ma99Values = calculateSMA(data, 99);

    const drawMALine = (values: (number | null)[], color: string) => {
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      let started = false;
      values.forEach((val, i) => {
        if (val !== null) {
          const x = padding.left + candleSpacing * i + candleSpacing / 2;
          const y = priceToY(val);
          if (!started) {
            ctx.moveTo(x, y);
            started = true;
          } else {
            ctx.lineTo(x, y);
          }
        }
      });
      ctx.stroke();
    };

    drawMALine(ma7Values, colors.ma.ma7);
    drawMALine(ma25Values, colors.ma.ma25);
    drawMALine(ma99Values, colors.ma.ma99);

    // Draw time labels
    ctx.fillStyle = colors.text.tertiary;
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'center';
    const labelInterval = Math.ceil(data.length / 8);
    data.forEach((candle, i) => {
      if (i % labelInterval === 0) {
        const x = padding.left + candleSpacing * i + candleSpacing / 2;
        ctx.fillText(candle.time, x, height - 8);
      }
    });

    // Draw current price line and badge
    if (currentPrice > 0) {
      const currentY = priceToY(currentPrice);
      const isUp = priceChange >= 0;
      const badgeColor = isUp ? colors.candle.up : colors.candle.down;

      // Dashed line
      ctx.strokeStyle = badgeColor;
      ctx.lineWidth = 1;
      ctx.setLineDash([2, 2]);
      ctx.beginPath();
      ctx.moveTo(padding.left, currentY);
      ctx.lineTo(width - padding.right, currentY);
      ctx.stroke();
      ctx.setLineDash([]);

      // Price badge on right
      const badgeWidth = 65;
      const badgeHeight = 20;
      ctx.fillStyle = badgeColor;
      ctx.fillRect(width - padding.right, currentY - badgeHeight / 2, badgeWidth, badgeHeight);

      ctx.fillStyle = '#fff';
      ctx.font = 'bold 10px monospace';
      ctx.textAlign = 'left';
      ctx.fillText(currentPrice.toFixed(2), width - padding.right + 4, currentY + 4);
    }

    // Draw crosshair and hover info
    if (mousePos && hoveredCandle) {
      const candleIndex = data.findIndex(d => d.timestamp === hoveredCandle.timestamp);
      if (candleIndex >= 0) {
        const x = padding.left + candleSpacing * candleIndex + candleSpacing / 2;

        // Vertical line
        ctx.strokeStyle = colors.crosshair;
        ctx.lineWidth = 0.5;
        ctx.setLineDash([3, 3]);
        ctx.beginPath();
        ctx.moveTo(x, padding.top);
        ctx.lineTo(x, height - padding.bottom);
        ctx.stroke();

        // Horizontal line
        ctx.beginPath();
        ctx.moveTo(padding.left, mousePos.y);
        ctx.lineTo(width - padding.right, mousePos.y);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
  }, [data, currentPrice, priceChange, mousePos, hoveredCandle]);

  // Draw volume chart
  useEffect(() => {
    if (!volumeCanvasRef.current || data.length === 0) return;

    const canvas = volumeCanvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;
    const padding = { top: 5, right: 70, bottom: 0, left: 10 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // Clear
    ctx.fillStyle = colors.bg.primary;
    ctx.fillRect(0, 0, width, height);

    // Volume range
    const maxVolume = Math.max(...data.map(d => d.volume));

    const candleSpacing = chartWidth / data.length;
    const barWidth = Math.max(2, candleSpacing * 0.7);

    // Draw volume bars
    data.forEach((candle, i) => {
      const x = padding.left + candleSpacing * i + candleSpacing / 2;
      const isUp = candle.close >= candle.open;
      const color = isUp ? colors.volume.up : colors.volume.down;

      const barHeight = (candle.volume / maxVolume) * chartHeight;
      const y = height - barHeight;

      ctx.fillStyle = color;
      ctx.fillRect(x - barWidth / 2, y, barWidth, barHeight);
    });

    // Draw Volume MA line (like Binance cyan line)
    const volumeMA: (number | null)[] = [];
    const maPeriod = 20;
    for (let i = 0; i < data.length; i++) {
      if (i < maPeriod - 1) {
        volumeMA.push(null);
      } else {
        let sum = 0;
        for (let j = 0; j < maPeriod; j++) {
          sum += data[i - j].volume;
        }
        volumeMA.push(sum / maPeriod);
      }
    }

    ctx.strokeStyle = '#5AC8FA'; // Cyan like Binance
    ctx.lineWidth = 1;
    ctx.beginPath();
    let started = false;
    volumeMA.forEach((val, i) => {
      if (val !== null) {
        const x = padding.left + candleSpacing * i + candleSpacing / 2;
        const y = height - (val / maxVolume) * chartHeight;
        if (!started) {
          ctx.moveTo(x, y);
          started = true;
        } else {
          ctx.lineTo(x, y);
        }
      }
    });
    ctx.stroke();

    // Volume label
    ctx.fillStyle = colors.text.tertiary;
    ctx.font = '9px sans-serif';
    ctx.textAlign = 'left';
    const volLabel = maxVolume > 1000 ? `${(maxVolume / 1000).toFixed(0)}K` : maxVolume.toFixed(0);
    ctx.fillText(volLabel, width - padding.right + 5, 12);
  }, [data]);

  // Handle mouse events for hover tooltip
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas || data.length === 0) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const padding = { left: 10, right: 70 };
    const chartWidth = rect.width - padding.left - padding.right;
    const candleSpacing = chartWidth / data.length;

    const candleIndex = Math.floor((x - padding.left) / candleSpacing);

    if (candleIndex >= 0 && candleIndex < data.length) {
      setHoveredCandle(data[candleIndex]);
      setMousePos({ x, y });
    }
  }, [data]);

  const handleMouseLeave = useCallback(() => {
    setHoveredCandle(null);
    setMousePos(null);
  }, []);

  const formatPrice = (price: number) => {
    if (price >= 1000) {
      return price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    }
    return price.toFixed(4);
  };

  const formatVolume = (vol: number) => {
    if (vol >= 1000000) return `${(vol / 1000000).toFixed(2)}M`;
    if (vol >= 1000) return `${(vol / 1000).toFixed(2)}K`;
    return vol.toFixed(2);
  };

  return (
    <div className="h-full flex flex-col" style={{ backgroundColor: colors.bg.primary }}>
      {/* Timeframe selector bar - like Binance */}
      {showControls && (
        <div
          className="flex items-center gap-1 px-3 py-1.5 border-b text-xs"
          style={{ borderColor: colors.border.default }}
        >
          <span style={{ color: colors.text.tertiary }} className="mr-2">Time</span>
          {TIMEFRAMES.map((tf) => (
            <button
              key={tf}
              onClick={() => setTimeRange(tf.toLowerCase())}
              className="px-2 py-0.5 rounded transition-colors"
              style={{
                backgroundColor: timeRange === tf.toLowerCase() ? colors.bg.secondary : 'transparent',
                color: timeRange === tf.toLowerCase() ? colors.text.primary : colors.text.tertiary,
              }}
            >
              {tf}
            </button>
          ))}
        </div>
      )}

      {/* OHLC Header - Fixed (no jumping on hover) */}
      <div
        className="flex flex-wrap items-center gap-x-4 gap-y-1 px-3 py-2 border-b text-xs"
        style={{ borderColor: colors.border.default }}
      >
        {/* Always show latest candle info - no change on hover to prevent UI jumping */}
        <span style={{ color: colors.text.secondary }}>
          Open <span style={{ color: colors.text.primary }}>{formatPrice(data[data.length - 1]?.open || 0)}</span>
        </span>
        <span style={{ color: colors.text.secondary }}>
          High <span style={{ color: colors.candle.up }}>{formatPrice(high24h)}</span>
        </span>
        <span style={{ color: colors.text.secondary }}>
          Low <span style={{ color: colors.candle.down }}>{formatPrice(low24h)}</span>
        </span>
        <span style={{ color: colors.text.secondary }}>
          Close <span style={{ color: priceChange >= 0 ? colors.candle.up : colors.candle.down }}>
            {formatPrice(currentPrice)}
          </span>
        </span>
        <span style={{ color: colors.text.secondary }}>
          CHANGE <span style={{ color: priceChange >= 0 ? colors.candle.up : colors.candle.down }}>
            {priceChange >= 0 ? '+' : ''}{priceChange.toFixed(2)}%
          </span>
        </span>

        {/* MA indicators */}
        <div className="flex items-center gap-3 ml-auto">
          <span>
            <span style={{ color: colors.ma.ma7 }}>MA(7)</span>
            <span className="ml-1" style={{ color: colors.ma.ma7 }}>{formatPrice(ma7)}</span>
          </span>
          <span>
            <span style={{ color: colors.ma.ma25 }}>MA(25)</span>
            <span className="ml-1" style={{ color: colors.ma.ma25 }}>{formatPrice(ma25)}</span>
          </span>
          <span>
            <span style={{ color: colors.ma.ma99 }}>MA(99)</span>
            <span className="ml-1" style={{ color: colors.ma.ma99 }}>{formatPrice(ma99)}</span>
          </span>
        </div>
      </div>

      {/* Main Chart Canvas */}
      <div className="flex-1 relative min-h-0" style={{ minHeight: '200px' }}>
        {isLoading && data.length === 0 ? (
          <div
            className="absolute inset-0 flex items-center justify-center"
            style={{ backgroundColor: colors.bg.primary }}
          >
            <div className="flex items-center gap-2" style={{ color: colors.text.secondary }}>
              <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                  fill="none"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                />
              </svg>
              <span className="text-sm">Loading market data...</span>
            </div>
          </div>
        ) : (
          <canvas
            ref={canvasRef}
            className="w-full h-full cursor-crosshair"
            style={{ display: 'block' }}
            onMouseMove={handleMouseMove}
            onMouseLeave={handleMouseLeave}
          />
        )}

        {/* Floating tooltip box - like Binance gray box */}
        {mousePos && hoveredCandle && (
          <div
            className="absolute pointer-events-none text-xs rounded shadow-lg"
            style={{
              left: mousePos.x > 200 ? mousePos.x - 160 : mousePos.x + 20,
              top: Math.min(mousePos.y, 200),
              backgroundColor: colors.bg.tooltip,
              border: `1px solid ${colors.border.default}`,
              padding: '8px 12px',
              minWidth: '140px',
              zIndex: 50,
            }}
          >
            <div style={{ color: colors.text.tertiary, marginBottom: '4px' }}>
              {new Date(hoveredCandle.timestamp).toLocaleDateString('en-US', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
              })} {hoveredCandle.time}
            </div>
            <div className="flex justify-between" style={{ color: colors.text.secondary }}>
              <span>Open</span>
              <span style={{ color: colors.text.primary }}>{formatPrice(hoveredCandle.open)}</span>
            </div>
            <div className="flex justify-between" style={{ color: colors.text.secondary }}>
              <span>High</span>
              <span style={{ color: colors.candle.up }}>{formatPrice(hoveredCandle.high)}</span>
            </div>
            <div className="flex justify-between" style={{ color: colors.text.secondary }}>
              <span>Low</span>
              <span style={{ color: colors.candle.down }}>{formatPrice(hoveredCandle.low)}</span>
            </div>
            <div className="flex justify-between" style={{ color: colors.text.secondary }}>
              <span>Close</span>
              <span style={{ color: hoveredCandle.close >= hoveredCandle.open ? colors.candle.up : colors.candle.down }}>
                {formatPrice(hoveredCandle.close)}
              </span>
            </div>
            <div className="flex justify-between" style={{ color: colors.text.secondary }}>
              <span>Vol</span>
              <span style={{ color: colors.text.primary }}>{formatVolume(hoveredCandle.volume)}</span>
            </div>
          </div>
        )}

        {/* Date label box at bottom - like Binance */}
        {mousePos && hoveredCandle && (
          <div
            className="absolute pointer-events-none text-xs rounded"
            style={{
              left: mousePos.x - 60,
              bottom: 5,
              backgroundColor: colors.bg.secondary,
              border: `1px solid ${colors.border.default}`,
              padding: '2px 8px',
              zIndex: 50,
            }}
          >
            <span style={{ color: colors.text.primary }}>
              {new Date(hoveredCandle.timestamp).toLocaleDateString('en-US', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
              })} {hoveredCandle.time}
            </span>
          </div>
        )}

        {/* Price label on right side when hovering */}
        {mousePos && (
          <div
            className="absolute pointer-events-none text-xs"
            style={{
              right: 5,
              top: mousePos.y - 10,
              backgroundColor: colors.bg.secondary,
              border: `1px solid ${colors.border.default}`,
              padding: '2px 6px',
              borderRadius: '2px',
              zIndex: 50,
            }}
          >
            <span style={{ color: colors.text.primary }}>
              {(() => {
                const canvas = canvasRef.current;
                if (!canvas || data.length === 0) return '0.00';
                const rect = canvas.getBoundingClientRect();
                const padding = { top: 10, bottom: 25 };
                const chartHeight = rect.height - padding.top - padding.bottom;
                const allPrices = data.flatMap((d) => [d.high, d.low]);
                const minPrice = Math.min(...allPrices);
                const maxPrice = Math.max(...allPrices);
                const priceRange = maxPrice - minPrice || 1;
                const paddedMin = minPrice - priceRange * 0.05;
                const paddedMax = maxPrice + priceRange * 0.05;
                const paddedRange = paddedMax - paddedMin;
                const price = paddedMax - ((mousePos.y - padding.top) / chartHeight) * paddedRange;
                return formatPrice(price);
              })()}
            </span>
          </div>
        )}

        {error && (
          <div
            className="absolute top-2 right-2 text-xs px-2 py-1 rounded"
            style={{ backgroundColor: 'rgba(246, 70, 93, 0.15)', color: colors.candle.down }}
          >
            {error}
          </div>
        )}
      </div>

      {/* Volume Chart */}
      <div className="border-t" style={{ borderColor: colors.border.default, height: '60px' }}>
        <div className="flex items-center gap-2 px-3 py-1 text-xs" style={{ color: colors.text.tertiary }}>
          <span>Vol(BTC)</span>
          <span style={{ color: colors.text.primary }}>{formatVolume(data[data.length - 1]?.volume || 0)}</span>
        </div>
        <canvas
          ref={volumeCanvasRef}
          className="w-full"
          style={{ display: 'block', height: '40px' }}
        />
      </div>
    </div>
  );
}
