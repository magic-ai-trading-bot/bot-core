/**
 * Achievements Component
 *
 * Display achievement badges with locked/unlocked states and progress.
 */

import { GlassCard } from '@/components/ui/GlassCard';
import { Progress } from '@/components/ui/progress';
import { Trophy, Lock, Star, TrendingUp, Target, Zap } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
  unlocked: boolean;
  progress?: number;
  maxProgress?: number;
  unlockedAt?: Date;
}

const ACHIEVEMENTS: Achievement[] = [
  {
    id: '1',
    name: 'First Trade',
    description: 'Execute your first trade',
    icon: Zap,
    unlocked: true,
    unlockedAt: new Date('2024-01-15'),
  },
  {
    id: '2',
    name: 'Profitable Week',
    description: 'Achieve positive P&L for 7 consecutive days',
    icon: TrendingUp,
    unlocked: true,
    unlockedAt: new Date('2024-02-01'),
  },
  {
    id: '3',
    name: 'Century',
    description: 'Complete 100 trades',
    icon: Trophy,
    unlocked: true,
    progress: 100,
    maxProgress: 100,
    unlockedAt: new Date('2024-02-20'),
  },
  {
    id: '4',
    name: 'High Accuracy',
    description: 'Maintain 70%+ win rate for 30 days',
    icon: Target,
    unlocked: false,
    progress: 23,
    maxProgress: 30,
  },
  {
    id: '5',
    name: 'Risk Master',
    description: 'Never exceed max drawdown for 90 days',
    icon: Star,
    unlocked: false,
    progress: 45,
    maxProgress: 90,
  },
  {
    id: '6',
    name: 'Marathon Trader',
    description: 'Complete 1000 trades',
    icon: Trophy,
    unlocked: false,
    progress: 342,
    maxProgress: 1000,
  },
];

export function Achievements() {
  const unlockedCount = ACHIEVEMENTS.filter((a) => a.unlocked).length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-semibold text-gray-100">Achievements</h3>
          <p className="text-sm text-gray-400 mt-1">
            {unlockedCount} of {ACHIEVEMENTS.length} unlocked
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Trophy className="w-5 h-5 text-yellow-500" />
          <span className="text-lg font-semibold text-gray-100">{unlockedCount}</span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {ACHIEVEMENTS.map((achievement) => {
          const Icon = achievement.icon;
          const progressPercent = achievement.maxProgress
            ? (achievement.progress! / achievement.maxProgress) * 100
            : 0;

          return (
            <GlassCard
              key={achievement.id}
              className={cn(
                'relative overflow-hidden transition-all',
                achievement.unlocked
                  ? 'border-yellow-500/30 hover:border-yellow-500/50'
                  : 'opacity-60 hover:opacity-80'
              )}
            >
              {/* Unlocked glow effect */}
              {achievement.unlocked && (
                <div className="absolute inset-0 bg-gradient-to-br from-yellow-500/10 to-transparent" />
              )}

              <div className="relative">
                {/* Icon */}
                <div
                  className={cn(
                    'w-16 h-16 rounded-full flex items-center justify-center mb-4',
                    achievement.unlocked
                      ? 'bg-gradient-to-br from-yellow-500 to-amber-600'
                      : 'bg-slate-800/50 border-2 border-slate-700'
                  )}
                >
                  {achievement.unlocked ? (
                    <Icon className="w-8 h-8 text-white" />
                  ) : (
                    <Lock className="w-8 h-8 text-gray-500" />
                  )}
                </div>

                {/* Details */}
                <div>
                  <h4
                    className={cn(
                      'font-semibold mb-1',
                      achievement.unlocked ? 'text-yellow-400' : 'text-gray-300'
                    )}
                  >
                    {achievement.name}
                  </h4>
                  <p className="text-sm text-gray-400 mb-3">{achievement.description}</p>

                  {/* Progress */}
                  {achievement.maxProgress && (
                    <div className="space-y-1">
                      <div className="flex items-center justify-between text-xs">
                        <span className="text-gray-400">Progress</span>
                        <span className="text-gray-300">
                          {achievement.progress}/{achievement.maxProgress}
                        </span>
                      </div>
                      <Progress value={progressPercent} className="h-2" />
                    </div>
                  )}

                  {/* Unlocked date */}
                  {achievement.unlocked && achievement.unlockedAt && (
                    <p className="text-xs text-gray-500 mt-2">
                      Unlocked {achievement.unlockedAt.toLocaleDateString()}
                    </p>
                  )}
                </div>
              </div>
            </GlassCard>
          );
        })}
      </div>
    </div>
  );
}
