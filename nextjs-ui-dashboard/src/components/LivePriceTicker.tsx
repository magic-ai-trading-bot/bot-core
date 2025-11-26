import { useEffect, useState, useCallback } from 'react';
import { useWebSocketContext } from '@/contexts/WebSocketContext';

interface PriceData {
  symbol: string;
  price: number;
  change24h?: number;
  changePercent24h?: number;
}

const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

export function LivePriceTicker() {
  const [prices, setPrices] = useState<Record<string, PriceData>>({});
  const [symbols, setSymbols] = useState<string[]>([]);

  const { state: wsState } = useWebSocketContext();
  const lastMessage = wsState.lastMessage;

  // Fetch symbols dynamically from API
  const fetchSymbols = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/api/market/symbols`);
      const data = await response.json();
      // FIX: API returns {success: true, data: {symbols: [...]}} - access data.data.symbols
      if (data.success && data.data && data.data.symbols) {
        const symbols = data.data.symbols;
        setSymbols(symbols);
        // Initialize prices for all symbols
        const initialPrices: Record<string, PriceData> = {};
        symbols.forEach((symbol: string) => {
          initialPrices[symbol] = {
            symbol: symbol.replace('USDT', ''),
            price: 0,
          };
        });
        setPrices(initialPrices);
      }
    } catch (error) {
      // Fallback to default symbols if API fails
      const defaultSymbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT'];
      setSymbols(defaultSymbols);
      const initialPrices: Record<string, PriceData> = {};
      defaultSymbols.forEach((symbol) => {
        initialPrices[symbol] = {
          symbol: symbol.replace('USDT', ''),
          price: 0,
        };
      });
      setPrices(initialPrices);
    }
  }, []);

  // Load symbols on mount
  useEffect(() => {
    fetchSymbols();
  }, [fetchSymbols]);

  useEffect(() => {
    if (lastMessage) {
      try {
        const message = typeof lastMessage === 'string' ? JSON.parse(lastMessage) : lastMessage;

        // Handle price update messages from paper trading engine
        if (message.type === 'price_update' && message.data) {
          const priceUpdates: Record<string, number> = message.data;

          // eslint-disable-next-line react-hooks/set-state-in-effect
          setPrices(prev => {
            const updated = { ...prev };
            Object.entries(priceUpdates).forEach(([symbol, price]) => {
              if (updated[symbol]) {
                const oldPrice = updated[symbol].price;
                const change24h = price - oldPrice;
                const changePercent24h = oldPrice > 0 ? ((change24h / oldPrice) * 100) : 0;

                updated[symbol] = {
                  symbol: symbol.replace('USDT', ''),
                  price,
                  change24h,
                  changePercent24h,
                };
              }
            });
            return updated;
          });
        }
      } catch (error) {
        // eslint-disable-next-line no-console
        console.error('Error parsing price update:', error);
      }
    }
  }, [lastMessage]);

  return (
    <div className="flex gap-4 p-4 bg-gray-900 rounded-lg border border-gray-800">
      {symbols.map(symbolKey => {
        const data = prices[symbolKey];
        const isPositive = (data?.changePercent24h ?? 0) >= 0;
        const hasPrice = data?.price > 0;

        return (
          <div key={symbolKey} className="flex flex-col">
            <span className="text-sm text-gray-400 font-medium">{data?.symbol || symbolKey.replace('USDT', '')}</span>
            <span className="text-lg font-bold text-white">
              {hasPrice ? (
                `$${data.price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
              ) : (
                <span className="text-gray-600">Loading...</span>
              )}
            </span>
            {data?.changePercent24h !== undefined && (
              <span className={`text-sm font-medium ${isPositive ? 'text-green-500' : 'text-red-500'}`}>
                {isPositive ? '+' : ''}{data.changePercent24h.toFixed(2)}%
              </span>
            )}
          </div>
        );
      })}
      <div className="flex items-center ml-auto text-xs text-gray-500">
        <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse mr-2"></div>
        Live Prices
      </div>
    </div>
  );
}
