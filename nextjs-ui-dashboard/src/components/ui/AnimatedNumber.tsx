/**
 * AnimatedNumber Component
 *
 * Smoothly animates number changes with Framer Motion.
 * Supports profit/loss coloring and custom formatting.
 */

import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/utils';
import { colors } from '@/styles';
import { useEffect, useState } from 'react';

interface AnimatedNumberProps {
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  colorMode?: 'profit-loss' | 'neutral' | 'custom';
  customColor?: string;
  className?: string;
  animate?: boolean;
  formatOptions?: Intl.NumberFormatOptions;
}

export function AnimatedNumber({
  value,
  prefix = '',
  suffix = '',
  decimals = 2,
  colorMode = 'neutral',
  customColor,
  className,
  animate = true,
  formatOptions,
}: AnimatedNumberProps) {
  const [previousValue, setPreviousValue] = useState(value);

  // Determine color based on mode
  const getColor = () => {
    if (customColor) return customColor;

    if (colorMode === 'profit-loss') {
      if (value > 0) return colors.profit;
      if (value < 0) return colors.loss;
      return colors.neutral;
    }

    return colors.text.primary;
  };

  // Format number
  const formatNumber = (num: number) => {
    if (formatOptions) {
      return new Intl.NumberFormat('en-US', formatOptions).format(num);
    }
    return num.toFixed(decimals);
  };

  // Track value changes for animation
  useEffect(() => {
    if (value !== previousValue) {
      setPreviousValue(value);
    }
  }, [value, previousValue]);

  // Animation variants
  const variants = {
    initial: { opacity: 0, y: -10 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: 10 },
  };

  const color = getColor();

  if (!animate) {
    return (
      <span
        className={cn('font-mono font-semibold', className)}
        style={{ color }}
      >
        {prefix}
        {formatNumber(value)}
        {suffix}
      </span>
    );
  }

  return (
    <AnimatePresence mode="wait">
      <motion.span
        key={value}
        initial="initial"
        animate="animate"
        exit="exit"
        variants={variants}
        transition={{ duration: 0.3, ease: 'easeOut' }}
        className={cn('inline-block font-mono font-semibold', className)}
        style={{ color }}
      >
        {prefix}
        {formatNumber(value)}
        {suffix}
      </motion.span>
    </AnimatePresence>
  );
}

// Variant: Animated percentage
interface AnimatedPercentageProps {
  value: number;
  decimals?: number;
  showSign?: boolean;
  className?: string;
}

export function AnimatedPercentage({
  value,
  decimals = 2,
  showSign = true,
  className,
}: AnimatedPercentageProps) {
  const sign = showSign && value > 0 ? '+' : '';

  return (
    <AnimatedNumber
      value={value}
      prefix={sign}
      suffix="%"
      decimals={decimals}
      colorMode="profit-loss"
      className={className}
    />
  );
}

// Variant: Animated currency
interface AnimatedCurrencyProps {
  value: number;
  currency?: string;
  locale?: string;
  showSign?: boolean;
  className?: string;
}

export function AnimatedCurrency({
  value,
  currency = 'USD',
  locale = 'en-US',
  showSign = false,
  className,
}: AnimatedCurrencyProps) {
  const sign = showSign && value > 0 ? '+' : '';

  return (
    <AnimatedNumber
      value={value}
      prefix={sign}
      colorMode="profit-loss"
      formatOptions={{
        style: 'currency',
        currency,
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      }}
      className={className}
    />
  );
}
