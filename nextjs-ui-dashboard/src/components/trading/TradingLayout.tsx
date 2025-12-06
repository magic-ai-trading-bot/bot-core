/**
 * Trading Layout
 *
 * 3-column responsive layout for trading interface:
 * - Left: Chart + Market Info
 * - Center: Order Form
 * - Right: Positions + History
 *
 * Stacks vertically on mobile.
 */

import { ReactNode } from 'react';

interface TradingLayoutProps {
  chart: ReactNode;
  orderForm: ReactNode;
  positions: ReactNode;
  className?: string;
}

export function TradingLayout({
  chart,
  orderForm,
  positions,
  className = '',
}: TradingLayoutProps) {
  return (
    <div
      className={`grid grid-cols-1 gap-4 lg:grid-cols-12 ${className}`}
    >
      {/* Left Column: Chart + Market Info */}
      <div className="lg:col-span-5 xl:col-span-6">
        {chart}
      </div>

      {/* Center Column: Order Form */}
      <div className="lg:col-span-3 xl:col-span-3">
        {orderForm}
      </div>

      {/* Right Column: Positions + History */}
      <div className="lg:col-span-4 xl:col-span-3">
        {positions}
      </div>
    </div>
  );
}
