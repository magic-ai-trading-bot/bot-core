/**
 * ActivityTimeline Component
 *
 * Display recent user activity and trade highlights.
 */

import { GlassCard } from '@/components/ui/GlassCard';
import { Badge } from '@/components/ui/badge';
import { TrendingUp, TrendingDown, Settings, Trophy, Key, Bell } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Activity {
  id: string;
  type: 'trade' | 'setting' | 'achievement' | 'security' | 'notification';
  title: string;
  description: string;
  timestamp: Date;
  metadata?: {
    profit?: number;
    symbol?: string;
    status?: string;
  };
}

const RECENT_ACTIVITIES: Activity[] = [
  {
    id: '1',
    type: 'trade',
    title: 'Trade Executed',
    description: 'Opened long position on BTC/USDT',
    timestamp: new Date(Date.now() - 1000 * 60 * 15),
    metadata: { symbol: 'BTCUSDT', profit: 234.50 },
  },
  {
    id: '2',
    type: 'achievement',
    title: 'Achievement Unlocked',
    description: 'Completed 100 trades milestone',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
  },
  {
    id: '3',
    type: 'trade',
    title: 'Trade Closed',
    description: 'Closed position on ETH/USDT with profit',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5),
    metadata: { symbol: 'ETHUSDT', profit: 145.20 },
  },
  {
    id: '4',
    type: 'setting',
    title: 'Settings Updated',
    description: 'Changed default leverage to 5x',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24),
  },
  {
    id: '5',
    type: 'security',
    title: 'Security Update',
    description: 'Added new API key for Binance',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2),
  },
  {
    id: '6',
    type: 'trade',
    title: 'Trade Closed',
    description: 'Stop loss triggered on SOL/USDT',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 3),
    metadata: { symbol: 'SOLUSDT', profit: -85.30 },
  },
];

const getActivityIcon = (type: Activity['type']) => {
  switch (type) {
    case 'trade':
      return TrendingUp;
    case 'achievement':
      return Trophy;
    case 'setting':
      return Settings;
    case 'security':
      return Key;
    case 'notification':
      return Bell;
    default:
      return TrendingUp;
  }
};

const getActivityColor = (type: Activity['type']) => {
  switch (type) {
    case 'trade':
      return 'text-sky-500 bg-sky-500/10 border-sky-500/30';
    case 'achievement':
      return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/30';
    case 'setting':
      return 'text-purple-500 bg-purple-500/10 border-purple-500/30';
    case 'security':
      return 'text-green-500 bg-green-500/10 border-green-500/30';
    case 'notification':
      return 'text-blue-500 bg-blue-500/10 border-blue-500/30';
    default:
      return 'text-gray-500 bg-gray-500/10 border-gray-500/30';
  }
};

const formatTimeAgo = (date: Date) => {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return 'Just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
};

export function ActivityTimeline() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-semibold text-gray-100">Recent Activity</h3>
        <p className="text-sm text-gray-400 mt-1">Your latest actions and trade highlights</p>
      </div>

      <GlassCard padding="none">
        <div className="divide-y divide-slate-700/50">
          {RECENT_ACTIVITIES.map((activity, index) => {
            const Icon = getActivityIcon(activity.type);
            const colorClass = getActivityColor(activity.type);
            const isProfit = activity.metadata?.profit ? activity.metadata.profit > 0 : null;

            return (
              <div
                key={activity.id}
                className={cn(
                  'p-4 hover:bg-slate-800/30 transition-colors',
                  index === 0 && 'rounded-t-xl',
                  index === RECENT_ACTIVITIES.length - 1 && 'rounded-b-xl'
                )}
              >
                <div className="flex items-start gap-4">
                  {/* Icon */}
                  <div className={cn('p-2 rounded-full border', colorClass)}>
                    <Icon className="w-4 h-4" />
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1">
                        <h4 className="font-medium text-gray-100">{activity.title}</h4>
                        <p className="text-sm text-gray-400 mt-0.5">{activity.description}</p>

                        {/* Trade metadata */}
                        {activity.type === 'trade' && activity.metadata && (
                          <div className="flex items-center gap-2 mt-2">
                            <Badge variant="outline" className="border-slate-700 text-gray-300">
                              {activity.metadata.symbol}
                            </Badge>
                            {activity.metadata.profit !== undefined && (
                              <span
                                className={cn(
                                  'text-sm font-semibold',
                                  isProfit ? 'text-green-500' : 'text-red-500'
                                )}
                              >
                                {isProfit ? '+' : ''}${Math.abs(activity.metadata.profit).toFixed(2)}
                              </span>
                            )}
                          </div>
                        )}
                      </div>

                      <span className="text-xs text-gray-500 whitespace-nowrap">
                        {formatTimeAgo(activity.timestamp)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </GlassCard>
    </div>
  );
}
