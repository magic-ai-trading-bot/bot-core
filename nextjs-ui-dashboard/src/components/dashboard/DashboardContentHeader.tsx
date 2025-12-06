/**
 * DashboardContentHeader Component
 *
 * Dashboard content header with portfolio summary, mode indicator, and quick actions.
 */

import { PortfolioSummaryCard } from './PortfolioSummaryCard';
import { QuickActionsBar } from './QuickActionsBar';
import { ModeBadge } from '@/components/ui/ModeBadge';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { cn } from '@/lib/utils';

interface DashboardContentHeaderProps {
  balance?: number;
  pnl?: number;
  pnlPercentage?: number;
  isLoading?: boolean;
  className?: string;
}

export function DashboardContentHeader({
  balance = 10000,
  pnl = 0,
  pnlPercentage = 0,
  isLoading = false,
  className,
}: DashboardContentHeaderProps) {
  const { mode } = useTradingModeContext();

  return (
    <div className={cn('space-y-6', className)}>
      {/* Top row: Mode indicator and greeting */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-100">Dashboard</h1>
          <p className="text-gray-400 mt-1">
            Welcome back! Here's your portfolio overview.
          </p>
        </div>
        <ModeBadge mode={mode} size="lg" />
      </div>

      {/* Portfolio summary card */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <PortfolioSummaryCard
            balance={balance}
            pnl={pnl}
            pnlPercentage={pnlPercentage}
            isLoading={isLoading}
          />
        </div>

        {/* Quick actions */}
        <div className="flex items-center">
          <QuickActionsBar className="w-full lg:flex-col lg:items-stretch" />
        </div>
      </div>
    </div>
  );
}
