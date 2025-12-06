/**
 * Order Form
 *
 * Trade execution form with Market, Limit, Stop-Limit order types.
 * Mode-aware: Paper=direct submit, Real=confirmation dialog.
 */

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useTradingMode } from '@/hooks/useTradingMode';
import { useToast } from '@/hooks/use-toast';
import logger from '@/utils/logger';

export interface OrderFormData {
  symbol: string;
  orderType: 'market' | 'limit' | 'stop-limit';
  side: 'buy' | 'sell';
  quantity: number;
  price?: number;
  stopPrice?: number;
  leverage: number;
}

interface OrderFormProps {
  symbol?: string;
  onSubmit?: (order: OrderFormData) => void;
  onConfirmationRequired?: (order: OrderFormData) => void;
}

const SYMBOLS = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT'];

export function OrderForm({
  symbol: initialSymbol = 'BTCUSDT',
  onSubmit,
  onConfirmationRequired,
}: OrderFormProps) {
  const { mode } = useTradingMode();
  const { toast } = useToast();

  const [symbol, setSymbol] = useState(initialSymbol);
  const [orderType, setOrderType] = useState<'market' | 'limit' | 'stop-limit'>('market');
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [quantity, setQuantity] = useState('');
  const [price, setPrice] = useState('');
  const [stopPrice, setStopPrice] = useState('');
  const [leverage, setLeverage] = useState('10');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // Validation
    if (!quantity || parseFloat(quantity) <= 0) {
      toast({
        title: 'Invalid Quantity',
        description: 'Please enter a valid quantity',
        variant: 'destructive',
      });
      return;
    }

    if (orderType === 'limit' && (!price || parseFloat(price) <= 0)) {
      toast({
        title: 'Invalid Price',
        description: 'Please enter a valid limit price',
        variant: 'destructive',
      });
      return;
    }

    if (orderType === 'stop-limit' && (!stopPrice || parseFloat(stopPrice) <= 0)) {
      toast({
        title: 'Invalid Stop Price',
        description: 'Please enter a valid stop price',
        variant: 'destructive',
      });
      return;
    }

    const orderData: OrderFormData = {
      symbol,
      orderType,
      side,
      quantity: parseFloat(quantity),
      leverage: parseInt(leverage),
      ...(orderType !== 'market' && { price: parseFloat(price) }),
      ...(orderType === 'stop-limit' && { stopPrice: parseFloat(stopPrice) }),
    };

    logger.info(`Submitting ${mode} order:`, orderData);

    // Real mode requires confirmation
    if (mode === 'real') {
      if (onConfirmationRequired) {
        onConfirmationRequired(orderData);
      }
    } else {
      // Paper mode submits directly
      if (onSubmit) {
        onSubmit(orderData);
      } else {
        toast({
          title: 'Order Submitted (Paper)',
          description: `${side.toUpperCase()} ${quantity} ${symbol}`,
        });
      }
    }
  };

  const isBuy = side === 'buy';

  return (
    <Card>
      <CardHeader>
        <CardTitle>Place Order</CardTitle>
      </CardHeader>

      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Symbol Selector */}
          <div>
            <Label htmlFor="symbol">Symbol</Label>
            <Select value={symbol} onValueChange={setSymbol}>
              <SelectTrigger id="symbol">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SYMBOLS.map((sym) => (
                  <SelectItem key={sym} value={sym}>
                    {sym}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Buy/Sell Tabs */}
          <Tabs value={side} onValueChange={(value) => setSide(value as 'buy' | 'sell')}>
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="buy" className="data-[state=active]:bg-green-500">
                Buy
              </TabsTrigger>
              <TabsTrigger value="sell" className="data-[state=active]:bg-red-500">
                Sell
              </TabsTrigger>
            </TabsList>

            <TabsContent value={side} className="space-y-4">
              {/* Order Type */}
              <div>
                <Label htmlFor="orderType">Order Type</Label>
                <Select
                  value={orderType}
                  onValueChange={(value) =>
                    setOrderType(value as 'market' | 'limit' | 'stop-limit')
                  }
                >
                  <SelectTrigger id="orderType">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="market">Market</SelectItem>
                    <SelectItem value="limit">Limit</SelectItem>
                    <SelectItem value="stop-limit">Stop-Limit</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Limit Price (for limit and stop-limit orders) */}
              {(orderType === 'limit' || orderType === 'stop-limit') && (
                <div>
                  <Label htmlFor="price">Limit Price (USDT)</Label>
                  <Input
                    id="price"
                    type="number"
                    step="0.01"
                    placeholder="0.00"
                    value={price}
                    onChange={(e) => setPrice(e.target.value)}
                  />
                </div>
              )}

              {/* Stop Price (for stop-limit orders) */}
              {orderType === 'stop-limit' && (
                <div>
                  <Label htmlFor="stopPrice">Stop Price (USDT)</Label>
                  <Input
                    id="stopPrice"
                    type="number"
                    step="0.01"
                    placeholder="0.00"
                    value={stopPrice}
                    onChange={(e) => setStopPrice(e.target.value)}
                  />
                </div>
              )}

              {/* Quantity */}
              <div>
                <Label htmlFor="quantity">Quantity</Label>
                <Input
                  id="quantity"
                  type="number"
                  step="0.001"
                  placeholder="0.000"
                  value={quantity}
                  onChange={(e) => setQuantity(e.target.value)}
                />
              </div>

              {/* Leverage */}
              <div>
                <Label htmlFor="leverage">Leverage</Label>
                <Select value={leverage} onValueChange={setLeverage}>
                  <SelectTrigger id="leverage">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {[1, 2, 3, 5, 10, 20, 50, 100].map((lev) => (
                      <SelectItem key={lev} value={lev.toString()}>
                        {lev}x
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Order Summary */}
              <div className="rounded-md border p-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Order Value:</span>
                  <span className="font-semibold">
                    {quantity && price
                      ? `$${(parseFloat(quantity) * parseFloat(price)).toFixed(2)}`
                      : '--'}
                  </span>
                </div>
                <div className="mt-1 flex justify-between">
                  <span className="text-muted-foreground">With Leverage:</span>
                  <span className="font-semibold">
                    {quantity && price && leverage
                      ? `$${(
                          parseFloat(quantity) *
                          parseFloat(price) *
                          parseInt(leverage)
                        ).toFixed(2)}`
                      : '--'}
                  </span>
                </div>
              </div>

              {/* Submit Button */}
              <Button
                type="submit"
                className="w-full"
                variant={isBuy ? 'default' : 'destructive'}
              >
                {mode === 'real' ? '⚠️ ' : ''}
                {isBuy ? 'Buy' : 'Sell'} {symbol}
                {mode === 'real' ? ' (Real Money)' : ''}
              </Button>

              {mode === 'real' && (
                <p className="text-center text-xs text-muted-foreground">
                  You will be asked to confirm before submitting
                </p>
              )}
            </TabsContent>
          </Tabs>
        </form>
      </CardContent>
    </Card>
  );
}
