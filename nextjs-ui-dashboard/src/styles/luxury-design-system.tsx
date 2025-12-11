/**
 * Luxury Design System
 *
 * Premium Dark OLED Design System for Bot Core Trading Platform
 * Consistent glassmorphism, gradients, and animations across all pages.
 *
 * Usage:
 * import { luxuryColors, GlassCard, GradientText, ... } from '@/styles/luxury-design-system';
 */

import { motion, Variants } from 'framer-motion';
import { ReactNode } from 'react';

// ============================================================================
// DESIGN TOKENS - Premium Dark OLED Luxury
// ============================================================================

export const luxuryColors = {
  // Backgrounds - Pure black for OLED
  bgPrimary: '#000000',
  bgSecondary: 'rgba(255, 255, 255, 0.03)',
  bgTertiary: 'rgba(255, 255, 255, 0.05)',
  bgHover: 'rgba(255, 255, 255, 0.08)',
  bgCard: 'rgba(255, 255, 255, 0.03)',

  // Primary Accents (flat for backward compat)
  emerald: '#22c55e',
  cyan: '#00D9FF',
  purple: '#8b5cf6',
  amber: '#f59e0b',
  rose: '#f43f5e',

  // Semantic Colors (flat for backward compat)
  profit: '#22c55e',
  loss: '#ef4444',
  warning: '#f59e0b',
  info: '#00D9FF',
  success: '#22c55e',

  // Text (flat for backward compat)
  textPrimary: '#ffffff',
  textSecondary: 'rgba(255, 255, 255, 0.7)',
  textMuted: 'rgba(255, 255, 255, 0.4)',
  textDisabled: 'rgba(255, 255, 255, 0.25)',

  // Borders (flat for backward compat)
  borderSubtle: 'rgba(255, 255, 255, 0.08)',
  borderLight: 'rgba(255, 255, 255, 0.12)',
  borderActive: '#00D9FF',
  borderHover: 'rgba(255, 255, 255, 0.15)',

  // Gradients
  gradientPremium: 'linear-gradient(135deg, #00D9FF, #22c55e)',
  gradientProfit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
  gradientLoss: 'linear-gradient(135deg, #ef4444, #f97316)',
  gradientPurple: 'linear-gradient(135deg, #8b5cf6, #ec4899)',
  gradientGold: 'linear-gradient(135deg, #f59e0b, #fbbf24)',
  gradientCyan: 'linear-gradient(135deg, #00D9FF, #06b6d4)',

  // Glow Effects
  glowCyan: '0 0 20px rgba(0, 217, 255, 0.3)',
  glowEmerald: '0 0 20px rgba(34, 197, 94, 0.3)',
  glowPurple: '0 0 20px rgba(139, 92, 246, 0.3)',
  glowRed: '0 0 20px rgba(239, 68, 68, 0.3)',

  // =========================================================================
  // NESTED STRUCTURES (for pages using dot notation like luxuryColors.text.primary)
  // =========================================================================

  // Text colors (nested)
  text: {
    primary: '#ffffff',
    secondary: 'rgba(255, 255, 255, 0.7)',
    muted: 'rgba(255, 255, 255, 0.4)',
    disabled: 'rgba(255, 255, 255, 0.25)',
  },

  // Status colors (nested)
  status: {
    success: '#22c55e',
    error: '#ef4444',
    warning: '#f59e0b',
    info: '#00D9FF',
  },

  // Accent colors (nested)
  accent: {
    cyan: '#00D9FF',
    emerald: '#22c55e',
    purple: '#8b5cf6',
    amber: '#f59e0b',
    gold: '#f59e0b',
    rose: '#f43f5e',
  },

  // Glass effect (nested)
  glass: {
    background: 'rgba(255, 255, 255, 0.03)',
    blur: 'blur(20px)',
    border: 'rgba(255, 255, 255, 0.08)',
  },

  // Border colors (nested)
  border: {
    subtle: 'rgba(255, 255, 255, 0.08)',
    light: 'rgba(255, 255, 255, 0.12)',
    active: '#00D9FF',
    hover: 'rgba(255, 255, 255, 0.15)',
  },

  // Gradient presets (nested)
  gradient: {
    premium: 'linear-gradient(135deg, #00D9FF, #22c55e)',
    profit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
    loss: 'linear-gradient(135deg, #ef4444, #f97316)',
    purple: 'linear-gradient(135deg, #8b5cf6, #ec4899)',
    gold: 'linear-gradient(135deg, #f59e0b, #fbbf24)',
    cyan: 'linear-gradient(135deg, #00D9FF, #06b6d4)',
  },

  // Glow presets (nested)
  glow: {
    cyan: '0 0 20px rgba(0, 217, 255, 0.3)',
    emerald: '0 0 20px rgba(34, 197, 94, 0.3)',
    purple: '0 0 20px rgba(139, 92, 246, 0.3)',
    red: '0 0 20px rgba(239, 68, 68, 0.3)',
  },
} as const;

// ============================================================================
// ANIMATION VARIANTS
// ============================================================================

export const containerVariants: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.08, delayChildren: 0.02 },
  },
};

export const itemVariants: Variants = {
  hidden: { opacity: 0, y: 15 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { type: 'spring', stiffness: 100, damping: 15 },
  },
};

export const fadeInVariants: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.3 } },
};

export const slideUpVariants: Variants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.4 } },
};

export const scaleInVariants: Variants = {
  hidden: { opacity: 0, scale: 0.95 },
  visible: { opacity: 1, scale: 1, transition: { duration: 0.3 } },
};

// ============================================================================
// GLASS CARD COMPONENT
// ============================================================================

interface GlassCardProps {
  children: ReactNode;
  className?: string;
  noPadding?: boolean;
  hoverable?: boolean;
  glowColor?: string;
  onClick?: () => void;
}

export function GlassCard({
  children,
  className = '',
  noPadding = false,
  hoverable = false,
  glowColor,
  onClick,
}: GlassCardProps) {
  return (
    <motion.div
      variants={itemVariants}
      whileHover={
        hoverable
          ? {
              y: -2,
              boxShadow: glowColor || '0 8px 32px rgba(0, 217, 255, 0.15)',
            }
          : undefined
      }
      onClick={onClick}
      className={`
        relative overflow-hidden rounded-2xl
        bg-white/80 dark:bg-white/[0.03] backdrop-blur-xl
        border border-black/[0.08] dark:border-white/[0.08]
        shadow-sm dark:shadow-none
        transition-all duration-300
        ${hoverable ? 'cursor-pointer hover:border-black/[0.15] dark:hover:border-white/[0.15]' : ''}
        ${className}
      `}
    >
      <div className={noPadding ? '' : 'p-4 md:p-6'}>{children}</div>
    </motion.div>
  );
}

// ============================================================================
// GRADIENT TEXT COMPONENT
// ============================================================================

interface GradientTextProps {
  children: ReactNode;
  className?: string;
  gradient?: string;
}

export function GradientText({
  children,
  className = '',
  gradient = luxuryColors.gradientPremium,
}: GradientTextProps) {
  return (
    <span
      className={`bg-clip-text text-transparent ${className}`}
      style={{ backgroundImage: gradient }}
    >
      {children}
    </span>
  );
}

// ============================================================================
// GLOW ICON CONTAINER
// ============================================================================

interface GlowIconProps {
  icon: React.ElementType;
  color?: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function GlowIcon({
  icon: Icon,
  color = luxuryColors.cyan,
  size = 'md',
  className = '',
}: GlowIconProps) {
  // WCAG 2.5.5: Touch targets must be at least 44x44px for interactive elements
  const sizes = {
    sm: { container: 'p-2.5 min-w-[44px] min-h-[44px] flex items-center justify-center', icon: 'w-4 h-4' },
    md: { container: 'p-3 min-w-[44px] min-h-[44px] flex items-center justify-center', icon: 'w-5 h-5' },
    lg: { container: 'p-3.5 min-w-[48px] min-h-[48px] flex items-center justify-center', icon: 'w-6 h-6' },
  };

  const rgb = color === luxuryColors.cyan ? '0, 217, 255' :
              color === luxuryColors.emerald ? '34, 197, 94' :
              color === luxuryColors.purple ? '139, 92, 246' :
              color === luxuryColors.amber ? '245, 158, 11' :
              color === luxuryColors.rose ? '244, 63, 94' : '255, 255, 255';

  return (
    <div
      className={`rounded-xl ${sizes[size].container} ${className}`}
      style={{
        background: `rgba(${rgb}, 0.1)`,
        border: `1px solid rgba(${rgb}, 0.2)`,
      }}
    >
      <Icon className={sizes[size].icon} style={{ color }} />
    </div>
  );
}

// ============================================================================
// PREMIUM BADGE COMPONENT
// ============================================================================

interface BadgeProps {
  children: ReactNode;
  variant?: 'default' | 'success' | 'error' | 'warning' | 'info' | 'purple';
  size?: 'sm' | 'md';
  glow?: boolean;
  className?: string;
}

export function Badge({
  children,
  variant = 'default',
  size = 'md',
  glow = false,
  className = '',
}: BadgeProps) {
  const variants = {
    default: {
      bg: 'rgba(255, 255, 255, 0.1)',
      color: luxuryColors.textSecondary,
      border: 'rgba(255, 255, 255, 0.15)',
    },
    success: {
      bg: 'rgba(34, 197, 94, 0.15)',
      color: luxuryColors.profit,
      border: 'rgba(34, 197, 94, 0.3)',
    },
    error: {
      bg: 'rgba(239, 68, 68, 0.15)',
      color: luxuryColors.loss,
      border: 'rgba(239, 68, 68, 0.3)',
    },
    warning: {
      bg: 'rgba(245, 158, 11, 0.15)',
      color: luxuryColors.warning,
      border: 'rgba(245, 158, 11, 0.3)',
    },
    info: {
      bg: 'rgba(0, 217, 255, 0.15)',
      color: luxuryColors.cyan,
      border: 'rgba(0, 217, 255, 0.3)',
    },
    purple: {
      bg: 'rgba(139, 92, 246, 0.15)',
      color: luxuryColors.purple,
      border: 'rgba(139, 92, 246, 0.3)',
    },
  };

  const sizes = {
    sm: 'px-2 py-0.5 text-[9px]',
    md: 'px-2.5 py-1 text-[10px]',
  };

  const style = variants[variant];

  return (
    <span
      className={`inline-flex items-center rounded-lg font-bold uppercase tracking-wider ${sizes[size]} ${className}`}
      style={{
        backgroundColor: style.bg,
        color: style.color,
        border: `1px solid ${style.border}`,
        boxShadow: glow ? `0 0 15px ${style.bg}` : undefined,
      }}
    >
      {children}
    </span>
  );
}

// ============================================================================
// PREMIUM BUTTON COMPONENT
// ============================================================================

interface PremiumButtonProps {
  children: ReactNode;
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  disabled?: boolean;
  loading?: boolean;
  onClick?: () => void;
  type?: 'button' | 'submit';
  fullWidth?: boolean;
}

export function PremiumButton({
  children,
  variant = 'primary',
  size = 'md',
  className = '',
  disabled = false,
  loading = false,
  onClick,
  type = 'button',
  fullWidth = false,
}: PremiumButtonProps) {
  const variants = {
    primary: {
      bg: luxuryColors.gradientPremium,
      shadow: '0 8px 32px rgba(0, 217, 255, 0.3)',
      hoverShadow: '0 12px 40px rgba(0, 217, 255, 0.4)',
    },
    secondary: {
      bg: 'transparent',
      shadow: 'none',
      hoverShadow: 'none',
      border: 'none',
      textClass: 'text-black/70 dark:text-white/70 hover:text-black dark:hover:text-white border border-black/20 dark:border-white/20 hover:border-black/40 dark:hover:border-white/40',
    },
    success: {
      bg: luxuryColors.gradientProfit,
      shadow: '0 8px 32px rgba(34, 197, 94, 0.3)',
      hoverShadow: '0 12px 40px rgba(34, 197, 94, 0.4)',
    },
    danger: {
      bg: luxuryColors.gradientLoss,
      shadow: '0 8px 32px rgba(239, 68, 68, 0.3)',
      hoverShadow: '0 12px 40px rgba(239, 68, 68, 0.4)',
    },
    ghost: {
      bg: 'transparent',
      shadow: 'none',
      hoverShadow: 'none',
      textClass: 'text-black/70 dark:text-white/70 hover:text-black dark:hover:text-white',
    },
  };

  // WCAG 2.5.5: Touch targets must be at least 44x44px
  const sizes = {
    sm: 'px-4 py-2.5 text-xs min-h-[44px]',      // 44px height
    md: 'px-5 py-3 text-sm min-h-[44px]',        // 44px height
    lg: 'px-6 py-4 text-base min-h-[48px]',      // 48px height
  };

  const style = variants[variant];

  return (
    <motion.button
      type={type}
      onClick={onClick}
      disabled={disabled || loading}
      whileHover={!disabled ? { scale: 1.02, y: -2 } : undefined}
      whileTap={!disabled ? { scale: 0.98 } : undefined}
      className={`
        rounded-xl font-bold
        transition-all duration-300
        flex items-center justify-center gap-2
        disabled:opacity-50 disabled:cursor-not-allowed
        ${(style as { textClass?: string }).textClass || 'text-white'}
        ${sizes[size]}
        ${fullWidth ? 'w-full' : ''}
        ${className}
      `}
      style={{
        background: style.bg,
        boxShadow: style.shadow,
        border: (style as { border?: string }).border || 'none',
      }}
    >
      {loading ? (
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
          className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full"
        />
      ) : (
        children
      )}
    </motion.button>
  );
}

// ============================================================================
// PREMIUM INPUT COMPONENT
// ============================================================================

interface PremiumInputProps {
  id?: string;
  label?: string;
  value: string | number;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: 'text' | 'email' | 'password' | 'number';
  suffix?: string;
  prefix?: ReactNode;
  error?: string;
  disabled?: boolean;
  className?: string;
}

export function PremiumInput({
  id,
  label,
  value,
  onChange,
  placeholder,
  type = 'text',
  suffix,
  prefix,
  error,
  disabled = false,
  className = '',
}: PremiumInputProps) {
  return (
    <div className={className}>
      {label && (
        <label
          className="block text-[10px] uppercase tracking-wider mb-1.5 font-medium text-black/40 dark:text-white/40"
        >
          {label}
        </label>
      )}
      <div
        className={`
          relative flex items-center rounded-xl border transition-all duration-300
          bg-black/[0.03] dark:bg-white/[0.03]
          border-black/[0.08] dark:border-white/[0.08]
          focus-within:border-cyan-500/50 focus-within:shadow-[0_0_20px_rgba(0,217,255,0.15)]
          ${error ? 'border-red-500/50' : ''}
          ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      >
        {prefix && (
          <span className="pl-3 text-black/40 dark:text-white/40">
            {prefix}
          </span>
        )}
        <input
          id={id}
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          className="w-full px-3 py-2.5 text-sm font-mono bg-transparent outline-none text-black dark:text-white placeholder:text-black/30 dark:placeholder:text-white/30 disabled:cursor-not-allowed"
        />
        {suffix && (
          <span className="px-3 text-xs font-medium text-black/40 dark:text-white/40">
            {suffix}
          </span>
        )}
      </div>
      {error && (
        <p className="mt-1.5 text-xs" style={{ color: luxuryColors.loss }}>
          {error}
        </p>
      )}
    </div>
  );
}

// ============================================================================
// STAT CARD COMPONENT
// ============================================================================

interface StatCardProps {
  label: string;
  value: string | number;
  icon?: React.ElementType;
  trend?: number;
  trendLabel?: string;
  iconColor?: string;
  valueColor?: string;
  gradient?: boolean;
}

export function StatCard({
  label,
  value,
  icon: Icon,
  trend,
  trendLabel,
  iconColor = luxuryColors.cyan,
  valueColor,
  gradient = false,
}: StatCardProps) {
  const isPositiveTrend = trend && trend > 0;
  const isNegativeTrend = trend && trend < 0;

  return (
    <GlassCard hoverable glowColor={`0 8px 32px ${iconColor}30`}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-[10px] uppercase tracking-wider mb-1 text-black/50 dark:text-white/50">
            {label}
          </p>
          {gradient ? (
            <GradientText className="text-2xl font-black">{value}</GradientText>
          ) : (
            <p
              className="text-2xl font-black"
              style={{ color: valueColor || luxuryColors.textPrimary }}
            >
              {value}
            </p>
          )}
          {trend !== undefined && (
            <div className="flex items-center gap-1 mt-1">
              <span
                className="text-xs font-medium"
                style={{
                  color: isPositiveTrend
                    ? luxuryColors.profit
                    : isNegativeTrend
                      ? luxuryColors.loss
                      : luxuryColors.textMuted,
                }}
              >
                {isPositiveTrend ? '+' : ''}
                {trend}%
              </span>
              {trendLabel && (
                <span className="text-[10px] text-black/40 dark:text-white/40">
                  {trendLabel}
                </span>
              )}
            </div>
          )}
        </div>
        {Icon && <GlowIcon icon={Icon} color={iconColor} size="lg" />}
      </div>
    </GlassCard>
  );
}

// ============================================================================
// SECTION HEADER COMPONENT
// ============================================================================

interface SectionHeaderProps {
  title: string;
  subtitle?: string;
  icon?: React.ElementType;
  action?: ReactNode;
  gradient?: boolean;
}

export function SectionHeader({
  title,
  subtitle,
  icon: Icon,
  action,
  gradient = true,
}: SectionHeaderProps) {
  return (
    <div className="flex items-center justify-between mb-6">
      <div className="flex items-center gap-3">
        {Icon && <GlowIcon icon={Icon} size="lg" />}
        <div>
          {gradient ? (
            <GradientText className="text-xl font-black">{title}</GradientText>
          ) : (
            <h2 className="text-xl font-black text-white">{title}</h2>
          )}
          {subtitle && (
            <p
              className="text-xs mt-0.5"
              style={{ color: luxuryColors.textMuted }}
            >
              {subtitle}
            </p>
          )}
        </div>
      </div>
      {action}
    </div>
  );
}

// ============================================================================
// PAGE WRAPPER COMPONENT
// ============================================================================

interface PageWrapperProps {
  children: ReactNode;
  className?: string;
  withPadding?: boolean;
}

export function PageWrapper({
  children,
  className = '',
  withPadding = true,
}: PageWrapperProps) {
  return (
    <motion.div
      className={`min-h-full ${withPadding ? 'p-4 md:p-6 lg:p-8' : ''} ${className}`}
      // Removed hardcoded background - parent container handles theme-aware bg
      initial="hidden"
      animate="visible"
      variants={containerVariants}
    >
      {children}
    </motion.div>
  );
}

// ============================================================================
// EMPTY STATE COMPONENT
// ============================================================================

interface EmptyStateProps {
  icon: React.ElementType;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
}: EmptyStateProps) {
  return (
    <motion.div
      className="flex flex-col items-center justify-center py-12 text-center"
      variants={itemVariants}
    >
      <div
        className="p-4 rounded-2xl mb-4"
        style={{
          background: 'rgba(255, 255, 255, 0.03)',
          border: '1px solid rgba(255, 255, 255, 0.08)',
        }}
      >
        <Icon className="w-8 h-8 opacity-50" style={{ color: luxuryColors.textMuted }} />
      </div>
      <h3 className="text-sm font-medium text-white mb-1">{title}</h3>
      {description && (
        <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
          {description}
        </p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </motion.div>
  );
}

// ============================================================================
// LOADING SPINNER COMPONENT
// ============================================================================

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  color?: string;
}

export function LoadingSpinner({
  size = 'md',
  color = luxuryColors.cyan,
}: LoadingSpinnerProps) {
  const sizes = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8',
  };

  return (
    <motion.div
      animate={{ rotate: 360 }}
      transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
      className={`${sizes[size]} border-2 rounded-full`}
      style={{
        borderColor: `${color}30`,
        borderTopColor: color,
      }}
    />
  );
}

// ============================================================================
// DIVIDER COMPONENT
// ============================================================================

interface DividerProps {
  className?: string;
  vertical?: boolean;
}

export function Divider({ className = '', vertical = false }: DividerProps) {
  if (vertical) {
    return (
      <div
        className={`w-px h-full ${className}`}
        style={{ backgroundColor: luxuryColors.borderSubtle }}
      />
    );
  }
  return (
    <div
      className={`h-px w-full ${className}`}
      style={{ backgroundColor: luxuryColors.borderSubtle }}
    />
  );
}
