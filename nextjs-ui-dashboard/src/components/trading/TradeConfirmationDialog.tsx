/**
 * Trade Confirmation Dialog
 *
 * Modal dialog for confirming real money trades.
 * Requires explicit confirmation with checkbox.
 */

import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import type { OrderFormData } from './OrderForm';

interface TradeConfirmationDialogProps {
  open: boolean;
  order: OrderFormData | null;
  onConfirm: () => void;
  onCancel: () => void;
}

export function TradeConfirmationDialog({
  open,
  order,
  onConfirm,
  onCancel,
}: TradeConfirmationDialogProps) {
  const [confirmed, setConfirmed] = useState(false);

  const handleConfirm = () => {
    if (confirmed) {
      onConfirm();
      setConfirmed(false); // Reset for next time
    }
  };

  const handleCancel = () => {
    onCancel();
    setConfirmed(false); // Reset
  };

  if (!order) return null;

  const orderValue = order.price
    ? order.quantity * order.price
    : order.quantity * 50000; // Estimate for market orders
  const leveragedValue = orderValue * order.leverage;

  // Calculate estimated risk (stop loss distance)
  const riskPercent = 2; // Default 2% risk
  const estimatedRisk = leveragedValue * (riskPercent / 100);

  return (
    <Dialog open={open} onOpenChange={handleCancel}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-xl">
            ⚠️ Confirm Real Money Trade
          </DialogTitle>
          <DialogDescription>
            This order will execute with <strong>real funds</strong> on the live
            exchange. Please review carefully.
          </DialogDescription>
        </DialogHeader>

        {/* Order Summary */}
        <div className="space-y-4 rounded-lg border-2 border-destructive/50 bg-destructive/5 p-4">
          {/* Symbol and Side */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold">{order.symbol}</span>
              <Badge
                variant={order.side === 'buy' ? 'default' : 'destructive'}
                className="text-sm"
              >
                {order.side.toUpperCase()}
              </Badge>
            </div>
            <Badge variant="outline">{order.orderType.toUpperCase()}</Badge>
          </div>

          {/* Details Grid */}
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground">Quantity</p>
              <p className="font-semibold">{order.quantity}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Leverage</p>
              <p className="font-semibold">{order.leverage}x</p>
            </div>
            {order.price && (
              <div>
                <p className="text-muted-foreground">Price</p>
                <p className="font-semibold">${order.price.toFixed(2)}</p>
              </div>
            )}
            {order.stopPrice && (
              <div>
                <p className="text-muted-foreground">Stop Price</p>
                <p className="font-semibold">${order.stopPrice.toFixed(2)}</p>
              </div>
            )}
            <div>
              <p className="text-muted-foreground">Order Value</p>
              <p className="font-semibold">${orderValue.toFixed(2)}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Leveraged Exposure</p>
              <p className="font-bold text-destructive">
                ${leveragedValue.toFixed(2)}
              </p>
            </div>
          </div>

          {/* Risk Warning */}
          <div className="rounded-md bg-destructive/10 p-3 text-sm">
            <p className="font-semibold text-destructive">⚠️ Risk Warning</p>
            <p className="mt-1 text-xs text-muted-foreground">
              Estimated risk (2% stop loss): <strong>${estimatedRisk.toFixed(2)}</strong>
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              Maximum loss with {order.leverage}x leverage could be{' '}
              <strong>up to ${leveragedValue.toFixed(2)}</strong>
            </p>
          </div>
        </div>

        {/* Confirmation Checkbox */}
        <div className="flex items-start space-x-2">
          <Checkbox
            id="confirm-real-trade"
            checked={confirmed}
            onCheckedChange={(checked) => setConfirmed(checked === true)}
          />
          <Label
            htmlFor="confirm-real-trade"
            className="cursor-pointer text-sm leading-tight"
          >
            I understand this is <strong>real money</strong> and I accept the risk of
            loss. This trade will be executed immediately on the live exchange.
          </Label>
        </div>

        {/* Actions */}
        <DialogFooter>
          <Button variant="outline" onClick={handleCancel}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={handleConfirm}
            disabled={!confirmed}
          >
            ⚠️ Execute Real Trade
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
