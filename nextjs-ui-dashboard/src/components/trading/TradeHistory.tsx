/**
 * Trade History
 *
 * Historical trades with filters and P&L per trade.
 */

import { useState, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { formatDistanceToNow } from 'date-fns';
import type { PaperTrade } from '@/hooks/usePaperTrading';

interface TradeHistoryProps {
  trades: PaperTrade[];
  isLoading?: boolean;
}

export function TradeHistory({ trades, isLoading = false }: TradeHistoryProps) {
  const [filter, setFilter] = useState<'all' | 'wins' | 'losses'>('all');
  const [symbolFilter, setSymbolFilter] = useState<string>('all');

  // Get unique symbols
  const symbols = useMemo(() => {
    const uniqueSymbols = new Set(trades.map((t) => t.symbol));
    return Array.from(uniqueSymbols);
  }, [trades]);

  // Filter trades
  const filteredTrades = useMemo(() => {
    let filtered = trades;

    // Filter by symbol
    if (symbolFilter !== 'all') {
      filtered = filtered.filter((t) => t.symbol === symbolFilter);
    }

    // Filter by win/loss
    if (filter === 'wins') {
      filtered = filtered.filter((t) => (t.pnl || 0) > 0);
    } else if (filter === 'losses') {
      filtered = filtered.filter((t) => (t.pnl || 0) < 0);
    }

    return filtered;
  }, [trades, filter, symbolFilter]);

  // Calculate stats
  const stats = useMemo(() => {
    const wins = filteredTrades.filter((t) => (t.pnl || 0) > 0).length;
    const losses = filteredTrades.filter((t) => (t.pnl || 0) < 0).length;
    const totalPnl = filteredTrades.reduce((sum, t) => sum + (t.pnl || 0), 0);
    const winRate = filteredTrades.length > 0 ? (wins / filteredTrades.length) * 100 : 0;

    return {
      total: filteredTrades.length,
      wins,
      losses,
      totalPnl,
      winRate,
    };
  }, [filteredTrades]);

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-sm">Trade History</CardTitle>
          <Badge variant="outline">{stats.total} trades</Badge>
        </div>

        {/* Filters */}
        <div className="mt-4 flex flex-wrap gap-2">
          <Select value={filter} onValueChange={(value: any) => setFilter(value)}>
            <SelectTrigger className="w-[120px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Trades</SelectItem>
              <SelectItem value="wins">Wins Only</SelectItem>
              <SelectItem value="losses">Losses Only</SelectItem>
            </SelectContent>
          </Select>

          <Select value={symbolFilter} onValueChange={setSymbolFilter}>
            <SelectTrigger className="w-[140px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Symbols</SelectItem>
              {symbols.map((symbol) => (
                <SelectItem key={symbol} value={symbol}>
                  {symbol}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Stats Summary */}
        <div className="mt-4 grid grid-cols-4 gap-2 rounded-md border bg-muted/50 p-3 text-xs">
          <div>
            <p className="text-muted-foreground">Win Rate</p>
            <p className="font-semibold">{stats.winRate.toFixed(1)}%</p>
          </div>
          <div>
            <p className="text-muted-foreground">Wins</p>
            <p className="font-semibold text-green-500">{stats.wins}</p>
          </div>
          <div>
            <p className="text-muted-foreground">Losses</p>
            <p className="font-semibold text-red-500">{stats.losses}</p>
          </div>
          <div>
            <p className="text-muted-foreground">Total P&L</p>
            <p
              className={`font-bold ${
                stats.totalPnl >= 0 ? 'text-green-500' : 'text-red-500'
              }`}
            >
              ${stats.totalPnl.toFixed(2)}
            </p>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            Loading history...
          </div>
        ) : filteredTrades.length === 0 ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            No trades found
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <div className="space-y-3">
              {filteredTrades.map((trade) => {
                const isProfitable = (trade.pnl || 0) >= 0;
                const pnlColor = isProfitable ? 'text-green-500' : 'text-red-500';
                const closeTime = trade.close_time
                  ? new Date(trade.close_time)
                  : new Date();

                return (
                  <div
                    key={trade.id}
                    className="rounded-lg border bg-card p-3 text-sm"
                  >
                    {/* Header */}
                    <div className="mb-2 flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold">{trade.symbol}</span>
                        <Badge
                          variant={trade.trade_type === 'Long' ? 'default' : 'destructive'}
                        >
                          {trade.trade_type}
                        </Badge>
                        <Badge variant="outline" className="text-xs">
                          {trade.leverage}x
                        </Badge>
                      </div>
                      <span className="text-xs text-muted-foreground">
                        {formatDistanceToNow(closeTime, { addSuffix: true })}
                      </span>
                    </div>

                    {/* Details Grid */}
                    <div className="grid grid-cols-3 gap-2 text-xs">
                      <div>
                        <p className="text-muted-foreground">Entry</p>
                        <p className="font-semibold">${trade.entry_price.toFixed(2)}</p>
                      </div>
                      <div>
                        <p className="text-muted-foreground">Exit</p>
                        <p className="font-semibold">
                          ${trade.exit_price?.toFixed(2) || '--'}
                        </p>
                      </div>
                      <div>
                        <p className="text-muted-foreground">Quantity</p>
                        <p className="font-semibold">{trade.quantity.toFixed(4)}</p>
                      </div>
                    </div>

                    {/* P&L */}
                    <div className="mt-2 border-t pt-2">
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-muted-foreground">Profit/Loss</span>
                        <div className="text-right">
                          <p className={`text-sm font-bold ${pnlColor}`}>
                            {isProfitable ? '+' : ''}
                            ${(trade.pnl || 0).toFixed(2)}
                          </p>
                          <p className={`text-xs ${pnlColor}`}>
                            {isProfitable ? '+' : ''}
                            {trade.pnl_percentage.toFixed(2)}%
                          </p>
                        </div>
                      </div>

                      {trade.duration_ms && (
                        <p className="mt-1 text-xs text-muted-foreground">
                          Duration: {Math.floor(trade.duration_ms / 60000)}m{' '}
                          {Math.floor((trade.duration_ms % 60000) / 1000)}s
                        </p>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
