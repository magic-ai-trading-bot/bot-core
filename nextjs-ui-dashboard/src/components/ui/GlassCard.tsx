/**
 * GlassCard Component
 *
 * Glassmorphism card with backdrop blur for premium feel.
 * Mode-aware border colors for Paper/Real distinction.
 */

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { TradingMode, getTheme, withOpacity } from '@/styles';
import { ReactNode } from 'react';

interface GlassCardProps {
  children: ReactNode;
  mode?: TradingMode;
  className?: string;
  blur?: 'sm' | 'md' | 'lg';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  animate?: boolean;
  onClick?: () => void;
  hoverable?: boolean;
}

export function GlassCard({
  children,
  mode,
  className,
  blur = 'md',
  padding = 'md',
  animate = true,
  onClick,
  hoverable = false,
}: GlassCardProps) {
  const theme = mode ? getTheme(mode) : null;

  const blurClasses = {
    sm: 'backdrop-blur-sm',
    md: 'backdrop-blur-md',
    lg: 'backdrop-blur-lg',
  };

  const paddingClasses = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  };

  // Get border color based on mode
  const getBorderStyle = () => {
    if (!theme) {
      return 'border-slate-700/50';
    }

    return mode === 'paper'
      ? 'border-[#0EA5E9]/20'
      : 'border-[#EF4444]/30';
  };

  const Component = animate ? motion.div : 'div';

  const animationProps = animate
    ? {
        initial: { opacity: 0, y: 20 },
        animate: { opacity: 1, y: 0 },
        transition: { duration: 0.3, ease: 'easeOut' },
        whileHover: hoverable ? { y: -2, boxShadow: '0 8px 24px rgba(0, 0, 0, 0.3)' } : undefined,
      }
    : {};

  return (
    <Component
      className={cn(
        'rounded-xl border shadow-xl',
        'bg-slate-900/70',
        blurClasses[blur],
        getBorderStyle(),
        paddingClasses[padding],
        hoverable && 'cursor-pointer transition-shadow',
        className
      )}
      onClick={onClick}
      {...animationProps}
    >
      {children}
    </Component>
  );
}

// Variant: Glass card with header
interface GlassCardWithHeaderProps extends GlassCardProps {
  title: string;
  subtitle?: string;
  action?: ReactNode;
}

export function GlassCardWithHeader({
  title,
  subtitle,
  action,
  children,
  ...props
}: GlassCardWithHeaderProps) {
  return (
    <GlassCard {...props}>
      <div className="flex items-start justify-between mb-4">
        <div>
          <h3 className="text-xl font-semibold text-gray-100">{title}</h3>
          {subtitle && (
            <p className="text-sm text-gray-400 mt-1">{subtitle}</p>
          )}
        </div>
        {action && <div>{action}</div>}
      </div>
      {children}
    </GlassCard>
  );
}

// Variant: Glass card with highlight border (for active states)
interface HighlightGlassCardProps extends GlassCardProps {
  highlight?: boolean;
}

export function HighlightGlassCard({
  highlight = false,
  mode,
  ...props
}: HighlightGlassCardProps) {
  const theme = mode ? getTheme(mode) : null;

  // Add glow effect when highlighted
  const glowStyle = highlight && theme ? {
    boxShadow: `0 0 20px ${withOpacity(theme.primary, 0.3)}`,
  } : {};

  return (
    <GlassCard
      mode={mode}
      className={cn(
        highlight && 'ring-2',
        highlight && mode === 'paper' && 'ring-[#0EA5E9]/50',
        highlight && mode === 'real' && 'ring-[#EF4444]/50',
      )}
      style={glowStyle}
      {...props}
    />
  );
}
