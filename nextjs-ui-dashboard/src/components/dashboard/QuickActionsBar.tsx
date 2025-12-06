/**
 * QuickActionsBar Component
 *
 * Quick action buttons for common tasks.
 * Mode-specific styling and actions.
 */

import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { getModeColor } from '@/styles';
import { TrendingUp, Plus, Settings } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { cn } from '@/lib/utils';

interface QuickActionsBarProps {
  className?: string;
}

export function QuickActionsBar({ className }: QuickActionsBarProps) {
  const { mode } = useTradingModeContext();
  const navigate = useNavigate();
  const accentColor = getModeColor(mode, 'accent');

  const handleTradeNow = () => {
    navigate('/trading');
  };

  const handleAddFunds = () => {
    if (mode === 'paper') {
      navigate('/paper-trading/settings');
    } else {
      navigate('/deposit');
    }
  };

  const handleSettings = () => {
    navigate('/settings');
  };

  return (
    <div className={cn('flex flex-wrap gap-3', className)}>
      {/* Trade Now - Primary action */}
      <button
        onClick={handleTradeNow}
        className="flex items-center gap-2 px-6 py-3 rounded-lg font-semibold transition-all hover:shadow-lg"
        style={{
          backgroundColor: accentColor,
          color: 'white',
        }}
      >
        <TrendingUp className="w-5 h-5" />
        <span>Trade Now</span>
      </button>

      {/* Add Funds / Deposit */}
      <button
        onClick={handleAddFunds}
        className="flex items-center gap-2 px-6 py-3 rounded-lg font-semibold transition-all border hover:bg-slate-800/70"
        style={{
          borderColor: `${accentColor}40`,
          color: accentColor,
          backgroundColor: mode === 'paper' ? 'rgba(14, 165, 233, 0.05)' : 'rgba(239, 68, 68, 0.05)',
        }}
      >
        <Plus className="w-5 h-5" />
        <span>{mode === 'paper' ? 'Add Funds' : 'Deposit'}</span>
      </button>

      {/* Settings */}
      <button
        onClick={handleSettings}
        className="flex items-center gap-2 px-6 py-3 rounded-lg font-semibold transition-all border border-slate-700 hover:bg-slate-800/70 text-gray-300"
      >
        <Settings className="w-5 h-5" />
        <span>Settings</span>
      </button>
    </div>
  );
}
