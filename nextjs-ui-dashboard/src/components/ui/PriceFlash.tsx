/**
 * PriceFlash Component
 *
 * Flashes green/red on price change for visual feedback.
 * Uses Framer Motion for smooth color transitions.
 */

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { colors } from '@/styles';
import { useEffect, useState } from 'react';

interface PriceFlashProps {
  value: number | string;
  className?: string;
  children?: React.ReactNode;
  flashDuration?: number;
  colorMode?: 'profit-loss' | 'up-down' | 'disabled';
}

export function PriceFlash({
  value,
  className,
  children,
  flashDuration = 500,
  colorMode = 'up-down',
}: PriceFlashProps) {
  const [previousValue, setPreviousValue] = useState<number | string>(value);
  const [direction, setDirection] = useState<'up' | 'down' | null>(null);

  useEffect(() => {
    if (colorMode === 'disabled') return;

    const numericValue = typeof value === 'string' ? parseFloat(value) : value;
    const prevNumericValue = typeof previousValue === 'string'
      ? parseFloat(previousValue)
      : previousValue;

    if (numericValue > prevNumericValue) {
      setDirection('up');
    } else if (numericValue < prevNumericValue) {
      setDirection('down');
    }

    setPreviousValue(value);

    // Reset direction after flash
    const timer = setTimeout(() => {
      setDirection(null);
    }, flashDuration);

    return () => clearTimeout(timer);
  }, [value, previousValue, flashDuration, colorMode]);

  // Get flash color based on direction
  const getFlashColor = () => {
    if (!direction) return 'rgba(0, 0, 0, 0)';

    if (colorMode === 'up-down') {
      return direction === 'up'
        ? 'rgba(16, 185, 129, 0.3)'  // Green
        : 'rgba(239, 68, 68, 0.3)';   // Red
    }

    // profit-loss mode (inverse of up-down for some contexts)
    return direction === 'up'
      ? 'rgba(16, 185, 129, 0.3)'
      : 'rgba(239, 68, 68, 0.3)';
  };

  // Get text color based on direction
  const getTextColor = () => {
    if (!direction || colorMode === 'disabled') return colors.text.primary;

    return direction === 'up' ? colors.profit : colors.loss;
  };

  return (
    <motion.div
      className={cn('inline-block', className)}
      animate={{
        backgroundColor: direction
          ? [
              'rgba(0, 0, 0, 0)',
              getFlashColor(),
              'rgba(0, 0, 0, 0)',
            ]
          : undefined,
      }}
      transition={{ duration: flashDuration / 1000 }}
      style={{ color: getTextColor() }}
    >
      {children}
    </motion.div>
  );
}

// Variant: Price with flash and formatting
interface FormattedPriceFlashProps {
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  className?: string;
  showDirection?: boolean;
}

export function FormattedPriceFlash({
  value,
  prefix = '$',
  suffix = '',
  decimals = 2,
  className,
  showDirection = false,
}: FormattedPriceFlashProps) {
  const [previousValue, setPreviousValue] = useState(value);
  const [direction, setDirection] = useState<'up' | 'down' | null>(null);

  useEffect(() => {
    if (value > previousValue) {
      setDirection('up');
    } else if (value < previousValue) {
      setDirection('down');
    }

    setPreviousValue(value);

    const timer = setTimeout(() => {
      setDirection(null);
    }, 1000);

    return () => clearTimeout(timer);
  }, [value, previousValue]);

  const directionIcon = direction === 'up' ? '↑' : direction === 'down' ? '↓' : '';

  return (
    <PriceFlash value={value} className={className}>
      <span className="font-mono font-semibold">
        {showDirection && direction && (
          <span className="mr-1" aria-label={direction === 'up' ? 'increased' : 'decreased'}>
            {directionIcon}
          </span>
        )}
        {prefix}
        {value.toFixed(decimals)}
        {suffix}
      </span>
    </PriceFlash>
  );
}

// Variant: Table cell with price flash (for data tables)
interface PriceFlashCellProps {
  value: number;
  format?: 'currency' | 'percentage' | 'number';
  className?: string;
}

export function PriceFlashCell({
  value,
  format = 'currency',
  className,
}: PriceFlashCellProps) {
  let formattedContent: React.ReactNode;

  switch (format) {
    case 'currency':
      formattedContent = (
        <FormattedPriceFlash
          value={value}
          prefix="$"
          decimals={2}
          showDirection
        />
      );
      break;
    case 'percentage':
      formattedContent = (
        <FormattedPriceFlash
          value={value}
          prefix={value > 0 ? '+' : ''}
          suffix="%"
          decimals={2}
          showDirection
        />
      );
      break;
    default:
      formattedContent = (
        <FormattedPriceFlash
          value={value}
          prefix=""
          decimals={2}
          showDirection
        />
      );
  }

  return (
    <div className={cn('py-2', className)}>
      {formattedContent}
    </div>
  );
}
