/**
 * Profile Page - Luxury OLED Design
 *
 * User profile with real trading stats from API, achievements, and activity timeline.
 * Uses usePaperTrading() hook for real portfolio metrics and useAuth() for user data.
 */

import { useState, useMemo, useEffect } from 'react';
import logger from "@/utils/logger";
import { motion } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useAuth } from '@/contexts/AuthContext';
import { usePaperTrading } from '@/hooks/usePaperTrading';
import { toast } from 'sonner';

// API Base URL
const API_BASE = import.meta.env.VITE_RUST_API_URL || 'http://localhost:8080';
import {
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  Badge,
  GlowIcon,
  SectionHeader,
  PageWrapper,
  StatCard,
  containerVariants,
  itemVariants,
  Divider,
} from '@/styles/luxury-design-system';
import { useThemeColors } from '@/hooks/useThemeColors';
import {
  Mail,
  Calendar,
  Camera,
  Edit2,
  Check,
  X,
  BadgeCheck,
  TrendingUp,
  TrendingDown,
  Activity,
  Target,
  Trophy,
  Award,
  Star,
  Clock,
  AlertCircle,
  RefreshCw,
} from 'lucide-react';

export function Profile() {
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  // Get real user data from auth context
  const { user: authUser, isAuthenticated } = useAuth();

  // Get real trading stats from paper trading API
  const {
    portfolio,
    closedTrades,
    isLoading,
    error,
    refreshAll,
  } = usePaperTrading();

  // Local state for editing
  const [isEditing, setIsEditing] = useState(false);
  const [displayName, setDisplayName] = useState('');
  const [tempName, setTempName] = useState('');

  // Initialize display name from auth user
  useEffect(() => {
    if (authUser?.full_name) {
      setDisplayName(authUser.full_name);
      setTempName(authUser.full_name);
    } else if (authUser?.email) {
      // Fallback to email username
      const emailName = authUser.email.split('@')[0];
      setDisplayName(emailName);
      setTempName(emailName);
    }
  }, [authUser]);

  // Avatar state
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [isUploadingAvatar, setIsUploadingAvatar] = useState(false);

  // Initialize avatar from auth user
  useEffect(() => {
    if (authUser?.avatar_url) {
      setAvatarUrl(authUser.avatar_url);
    }
  }, [authUser]);

  // Combine auth user with defaults
  const user = useMemo(() => ({
    name: authUser?.full_name || authUser?.email?.split('@')[0] || 'Trader',
    email: authUser?.email || 'Not logged in',
    avatarUrl: avatarUrl,
    memberSince: authUser?.created_at ? new Date(authUser.created_at) : new Date(),
    verified: authUser?.email_verified || false,
  }), [authUser, avatarUrl]);

  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    if (!tempName.trim()) {
      toast.error(t('profile.errors.emptyName'));
      return;
    }

    setIsSaving(true);
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`${API_BASE}/api/auth/profile`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({ display_name: tempName.trim() }),
      });

      if (response.ok) {
        setDisplayName(tempName.trim());
        setIsEditing(false);
        toast.success(t('profile.success.profileUpdated'));
      } else {
        const data = await response.json().catch(() => ({}));
        toast.error(data.message || t('profile.errors.updateFailed'));
      }
    } catch (error) {
      logger.error('Profile update error:', error);
      toast.error(t('profile.errors.updateFailed'));
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancel = () => {
    setTempName(displayName);
    setIsEditing(false);
  };

  const handleAvatarUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.type.startsWith('image/')) {
      toast.error(t('profile.errors.invalidImage'));
      return;
    }

    // Validate file size (max 500KB)
    if (file.size > 500 * 1024) {
      toast.error(t('profile.errors.imageTooLarge'));
      return;
    }

    setIsUploadingAvatar(true);
    try {
      // Convert to base64
      const reader = new FileReader();
      reader.onloadend = async () => {
        const base64 = reader.result as string;

        // Upload to API
        const token = localStorage.getItem('auth_token');
        const response = await fetch(`${API_BASE}/api/auth/profile`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
          },
          body: JSON.stringify({ avatar_base64: base64 }),
        });

        if (response.ok) {
          const data = await response.json();
          if (data.data?.avatar_url) {
            setAvatarUrl(data.data.avatar_url);
          } else {
            // Fallback: use the base64 directly
            setAvatarUrl(base64);
          }
          toast.success(t('profile.success.avatarUpdated'));
        } else {
          const data = await response.json().catch(() => ({}));
          toast.error(data.error || t('profile.errors.avatarUploadFailed'));
        }
        setIsUploadingAvatar(false);
      };
      reader.onerror = () => {
        toast.error(t('profile.errors.readImageFailed'));
        setIsUploadingAvatar(false);
      };
      reader.readAsDataURL(file);
    } catch (error) {
      logger.error('Avatar upload error:', error);
      toast.error(t('profile.errors.avatarUploadFailed'));
      setIsUploadingAvatar(false);
    }
  };

  // User initials for avatar
  const initials = useMemo(() => {
    return (displayName || 'T')
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase()
      .substring(0, 2);
  }, [displayName]);

  // Real trading stats from portfolio API
  const stats = useMemo(() => ({
    totalTrades: portfolio.total_trades || 0,
    winRate: portfolio.win_rate || 0,
    totalPnL: portfolio.total_pnl || 0,
    bestTrade: portfolio.largest_win || 0,
    avgProfit: portfolio.average_win || 0,
    avgLoss: portfolio.average_loss || 0,
    profitFactor: portfolio.profit_factor || 0,
    sharpeRatio: portfolio.sharpe_ratio || 0,
  }), [portfolio]);

  // Generate achievements based on real trading data
  const achievements = useMemo(() => {
    const earned: Array<{ icon: typeof Trophy; label: string; date: string; color: string; unlocked: boolean }> = [];

    // First Trade achievement
    if (closedTrades.length > 0) {
      const firstTrade = closedTrades[closedTrades.length - 1]; // Oldest trade
      earned.push({
        icon: Trophy,
        label: t('profile.achievements.firstTrade'),
        date: firstTrade.exit_time || firstTrade.entry_time,
        color: colors.amber,
        unlocked: true,
      });
    } else {
      earned.push({
        icon: Trophy,
        label: t('profile.achievements.firstTrade'),
        date: t('profile.achievements.notYet'),
        color: colors.textMuted,
        unlocked: false,
      });
    }

    // 10 Trades achievement
    if (stats.totalTrades >= 10) {
      earned.push({
        icon: Target,
        label: t('profile.achievements.tenTrades'),
        date: t('profile.achievements.achieved'),
        color: colors.cyan,
        unlocked: true,
      });
    } else {
      earned.push({
        icon: Target,
        label: `${t('profile.achievements.tenTrades')} (${stats.totalTrades}/10)`,
        date: t('profile.achievements.inProgress'),
        color: colors.textMuted,
        unlocked: false,
      });
    }

    // Profitable achievement (total PnL > 0)
    if (stats.totalPnL > 0) {
      earned.push({
        icon: TrendingUp,
        label: t('profile.achievements.inProfit'),
        date: t('profile.achievements.active'),
        color: colors.emerald,
        unlocked: true,
      });
    } else {
      earned.push({
        icon: TrendingUp,
        label: t('profile.achievements.inProfit'),
        date: t('profile.achievements.notYet'),
        color: colors.textMuted,
        unlocked: false,
      });
    }

    // Win Rate > 50%
    if (stats.winRate > 50) {
      earned.push({
        icon: Award,
        label: t('profile.achievements.winningStreak'),
        date: `${stats.winRate.toFixed(1)}% ${t('stats.winRate').toLowerCase()}`,
        color: colors.purple,
        unlocked: true,
      });
    } else {
      earned.push({
        icon: Award,
        label: `${t('profile.achievements.winningStreak')} (>50%)`,
        date: `${stats.winRate.toFixed(1)}% ${t('profile.achievements.current')}`,
        color: colors.textMuted,
        unlocked: false,
      });
    }

    return earned;
  }, [closedTrades, stats, t, colors]);

  // Recent activity from closed trades
  const activities = useMemo(() => {
    const recentTrades = closedTrades.slice(0, 5).map((trade) => {
      const pnl = trade.pnl || 0;
      const isProfitable = pnl >= 0;
      const timeAgo = getTimeAgo(trade.exit_time || trade.entry_time);
      const sideLabel = trade.side === 'BUY' ? t('profile.activity.long') : t('profile.activity.short');

      return {
        type: 'trade' as const,
        action: `${t('profile.activity.closed')} ${trade.symbol} ${sideLabel}`,
        result: `${isProfitable ? '+' : ''}$${pnl.toFixed(2)}`,
        time: timeAgo,
        profit: isProfitable,
      };
    });

    // Add milestone activities based on achievements
    if (stats.totalTrades === 10) {
      recentTrades.push({
        type: 'achievement' as const,
        action: `${t('profile.activity.unlocked')}: ${t('profile.achievements.tenTrades')}`,
        result: null as unknown as string,
        time: t('profile.activity.milestone'),
        profit: null as unknown as boolean,
      });
    }

    return recentTrades.slice(0, 5);
  }, [closedTrades, stats.totalTrades, t]);

  // Helper function for time ago
  function getTimeAgo(timestamp: string): string {
    const diff = Date.now() - new Date(timestamp).getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }

  return (
    <PageWrapper>
      <motion.div
        className="max-w-7xl mx-auto space-y-6"
        variants={containerVariants}
      >
        {/* Error Banner */}
        {error && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="p-4 rounded-xl"
            style={{
              background: 'rgba(239, 68, 68, 0.1)',
              border: '1px solid rgba(239, 68, 68, 0.2)',
            }}
          >
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5" style={{ color: colors.loss }} />
              <p className="text-sm" style={{ color: colors.loss }}>
                {error}
              </p>
              <button
                onClick={() => refreshAll()}
                className="ml-auto p-2 rounded-lg transition-colors"
                style={{ background: 'rgba(239, 68, 68, 0.2)' }}
              >
                <RefreshCw className="w-4 h-4" style={{ color: colors.loss }} />
              </button>
            </div>
          </motion.div>
        )}

        {/* Profile Header Card */}
        <GlassCard noPadding>
          <div className="relative overflow-hidden">
            {/* Premium gradient background */}
            <div
              className="absolute inset-0 h-32 opacity-30"
              style={{
                background: colors.gradientPremium,
                filter: 'blur(60px)',
              }}
            />

            <div className="relative p-6 md:p-8">
              <div className="flex flex-col md:flex-row items-center md:items-start gap-6">
                {/* Avatar with glow effect */}
                <div className="relative group">
                  <motion.div
                    whileHover={{ scale: 1.05 }}
                    className="relative"
                  >
                    {/* Avatar container with premium border */}
                    <div
                      className="w-28 h-28 md:w-32 md:h-32 rounded-full flex items-center justify-center overflow-hidden"
                      style={{
                        background: colors.gradientPremium,
                        padding: '3px',
                        boxShadow: colors.glowCyan,
                      }}
                    >
                      <div
                        className="w-full h-full rounded-full flex items-center justify-center font-black text-3xl"
                        style={{
                          backgroundColor: colors.bgPrimary,
                          color: colors.textPrimary,
                        }}
                      >
                        {user.avatarUrl ? (
                          <img src={user.avatarUrl} alt={displayName} className="w-full h-full object-cover" />
                        ) : (
                          initials
                        )}
                      </div>
                    </div>

                    {/* Camera upload overlay */}
                    <label
                      htmlFor="avatar-upload"
                      className={`absolute inset-0 flex items-center justify-center bg-black/80 rounded-full transition-all duration-300 cursor-pointer ${
                        isUploadingAvatar ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
                      }`}
                    >
                      {isUploadingAvatar ? (
                        <RefreshCw className="w-8 h-8 animate-spin" style={{ color: colors.cyan }} />
                      ) : (
                        <Camera className="w-8 h-8" style={{ color: colors.cyan }} />
                      )}
                      <input
                        id="avatar-upload"
                        type="file"
                        accept="image/*"
                        className="hidden"
                        onChange={handleAvatarUpload}
                        disabled={isUploadingAvatar}
                      />
                    </label>
                  </motion.div>
                </div>

                {/* User Info */}
                <div className="flex-1 text-center md:text-left">
                  <div className="flex items-center justify-center md:justify-start gap-3 mb-3">
                    {isEditing ? (
                      <div className="flex items-center gap-2">
                        <PremiumInput
                          value={tempName}
                          onChange={setTempName}
                          className="max-w-xs"
                        />
                        <PremiumButton size="sm" onClick={handleSave} disabled={isSaving}>
                          {isSaving ? (
                            <RefreshCw className="w-4 h-4 animate-spin" />
                          ) : (
                            <Check className="w-4 h-4" />
                          )}
                        </PremiumButton>
                        <PremiumButton
                          size="sm"
                          variant="ghost"
                          onClick={handleCancel}
                        >
                          <X className="w-4 h-4" />
                        </PremiumButton>
                      </div>
                    ) : (
                      <>
                        <GradientText className="text-3xl font-black">
                          {displayName || 'Trader'}
                        </GradientText>
                        {user.verified && (
                          <BadgeCheck
                            className="w-6 h-6"
                            style={{ color: colors.cyan }}
                          />
                        )}
                        <motion.button
                          whileHover={{ scale: 1.1 }}
                          whileTap={{ scale: 0.95 }}
                          onClick={() => setIsEditing(true)}
                          className="p-1.5 rounded-lg transition-colors"
                          style={{
                            color: colors.textMuted,
                            backgroundColor: colors.bgSecondary,
                          }}
                        >
                          <Edit2 className="w-4 h-4" />
                        </motion.button>
                      </>
                    )}
                  </div>

                  <div className="flex items-center justify-center md:justify-start gap-2 mb-4">
                    <Mail className="w-4 h-4" style={{ color: colors.textMuted }} />
                    <span style={{ color: colors.textSecondary }}>{user.email}</span>
                  </div>

                  <div className="flex flex-wrap items-center justify-center md:justify-start gap-4">
                    <Badge variant="info" glow>
                      <Calendar className="w-3 h-3 inline mr-1" />
                      {t('profile.memberSince')} {user.memberSince.toLocaleDateString('en-US', {
                        month: 'short',
                        year: 'numeric',
                      })}
                    </Badge>
                    {isAuthenticated && (
                      <Badge variant="success" glow>
                        <BadgeCheck className="w-3 h-3 inline mr-1" />
                        {t('profile.authenticated')}
                      </Badge>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </GlassCard>

        {/* Trading Stats Section */}
        <motion.div variants={itemVariants}>
          <SectionHeader
            title={t('profile.title')}
            subtitle={t('profile.subtitle')}
            icon={Activity}
          />
        </motion.div>

        {/* Loading state for stats */}
        {isLoading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {[...Array(4)].map((_, i) => (
              <div
                key={i}
                className="h-28 rounded-xl animate-pulse"
                style={{ background: colors.bgSecondary }}
              />
            ))}
          </div>
        ) : (
          <>
            {/* Main Stats Grid */}
            <motion.div
              className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4"
              variants={containerVariants}
            >
              <StatCard
                label={t('stats.totalTrades')}
                value={stats.totalTrades}
                icon={Activity}
                iconColor={colors.cyan}
                gradient
              />
              <StatCard
                label={t('stats.winRate')}
                value={`${stats.winRate.toFixed(1)}%`}
                icon={Target}
                iconColor={colors.emerald}
                valueColor={stats.winRate >= 50 ? colors.profit : colors.loss}
              />
              <StatCard
                label={t('stats.totalPnL')}
                value={`${stats.totalPnL >= 0 ? '+' : ''}$${stats.totalPnL.toLocaleString('en-US', { minimumFractionDigits: 2 })}`}
                icon={stats.totalPnL >= 0 ? TrendingUp : TrendingDown}
                iconColor={stats.totalPnL >= 0 ? colors.emerald : colors.rose}
                valueColor={stats.totalPnL >= 0 ? colors.profit : colors.loss}
              />
              <StatCard
                label={t('stats.bestTrade')}
                value={`+$${stats.bestTrade.toLocaleString('en-US', { minimumFractionDigits: 2 })}`}
                icon={TrendingUp}
                iconColor={colors.emerald}
                valueColor={colors.profit}
              />
            </motion.div>

            {/* Advanced Metrics */}
            <GlassCard>
              <h4 className="text-sm font-bold uppercase tracking-wider mb-4" style={{ color: colors.textMuted }}>
                {t('profile.advancedMetrics')}
              </h4>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div>
                  <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: colors.textMuted }}>
                    {t('stats.avgProfit')}
                  </p>
                  <p className="text-2xl font-black" style={{ color: colors.profit }}>
                    +${stats.avgProfit.toFixed(2)}
                  </p>
                </div>
                <div>
                  <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: colors.textMuted }}>
                    {t('stats.avgLoss')}
                  </p>
                  <p className="text-2xl font-black" style={{ color: colors.loss }}>
                    -${Math.abs(stats.avgLoss).toFixed(2)}
                  </p>
                </div>
                <div>
                  <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: colors.textMuted }}>
                    {t('stats.profitFactor')}
                  </p>
                  <p className="text-2xl font-black" style={{ color: stats.profitFactor >= 1 ? colors.profit : colors.loss }}>
                    {stats.profitFactor.toFixed(2)}
                  </p>
                </div>
                <div>
                  <p className="text-[10px] uppercase tracking-wider mb-1" style={{ color: colors.textMuted }}>
                    {t('stats.sharpeRatio')}
                  </p>
                  <p className="text-2xl font-black" style={{ color: stats.sharpeRatio >= 1 ? colors.profit : colors.textSecondary }}>
                    {stats.sharpeRatio.toFixed(2)}
                  </p>
                </div>
              </div>
            </GlassCard>
          </>
        )}

        {/* Achievements Section */}
        <motion.div variants={itemVariants}>
          <SectionHeader
            title={t('profile.achievements.title')}
            subtitle={t('profile.achievements.subtitle')}
            icon={Trophy}
          />
        </motion.div>

        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4"
          variants={containerVariants}
        >
          {achievements.map((achievement, index) => (
            <GlassCard key={index} hoverable>
              <div className="flex items-center gap-3">
                <GlowIcon
                  icon={achievement.icon}
                  color={achievement.color}
                  size="lg"
                  style={{ opacity: achievement.unlocked ? 1 : 0.4 }}
                />
                <div>
                  <p
                    className="font-bold text-sm"
                    style={{ color: achievement.unlocked ? colors.textPrimary : colors.textMuted }}
                  >
                    {achievement.label}
                  </p>
                  <p className="text-[10px]" style={{ color: colors.textMuted }}>
                    {achievement.unlocked && achievement.date !== 'Achieved' && achievement.date !== 'Active'
                      ? new Date(achievement.date).toLocaleDateString('en-US', {
                          month: 'short',
                          day: 'numeric',
                          year: 'numeric',
                        })
                      : achievement.date}
                  </p>
                </div>
              </div>
            </GlassCard>
          ))}
        </motion.div>

        {/* Activity Timeline Section */}
        <motion.div variants={itemVariants}>
          <SectionHeader
            title={t('profile.activity.title')}
            subtitle={t('profile.activity.subtitle')}
            icon={Clock}
          />
        </motion.div>

        <GlassCard>
          {activities.length === 0 ? (
            <div className="text-center py-8">
              <Activity className="w-12 h-12 mx-auto mb-3" style={{ color: colors.textMuted }} />
              <p style={{ color: colors.textMuted }}>{t('profile.activity.noActivity')}</p>
              <p className="text-sm mt-1" style={{ color: colors.textMuted }}>
                {t('profile.activity.startTrading')}
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {activities.map((activity, index) => (
                <div key={index}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <GlowIcon
                        icon={activity.type === 'achievement' ? Star : Activity}
                        color={
                          activity.type === 'achievement'
                            ? colors.amber
                            : activity.profit
                              ? colors.emerald
                              : colors.rose
                        }
                        size="sm"
                      />
                      <div>
                        <p className="text-sm font-medium" style={{ color: colors.textPrimary }}>
                          {activity.action}
                        </p>
                        <p className="text-[10px]" style={{ color: colors.textMuted }}>
                          {activity.time}
                        </p>
                      </div>
                    </div>
                    {activity.result && (
                      <span
                        className="text-sm font-bold"
                        style={{
                          color: activity.profit ? colors.profit : colors.loss,
                        }}
                      >
                        {activity.result}
                      </span>
                    )}
                  </div>
                  {index < activities.length - 1 && <Divider className="mt-4" />}
                </div>
              ))}
            </div>
          )}
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
}

export default Profile;
