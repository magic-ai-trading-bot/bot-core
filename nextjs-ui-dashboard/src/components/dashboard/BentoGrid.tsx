/**
 * BentoGrid Component
 *
 * CSS Grid layout with responsive breakpoints for dashboard widgets.
 * Named grid areas for flexible widget placement.
 */

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';
import { spacing } from '@/styles';

interface BentoGridProps {
  children: ReactNode;
  className?: string;
}

export function BentoGrid({ children, className }: BentoGridProps) {
  return (
    <div
      className={cn(
        'grid gap-4 md:gap-6',
        // Mobile: 1 column
        'grid-cols-1',
        // Tablet: 2 columns
        'md:grid-cols-2',
        // Desktop: 4 columns with named areas
        'lg:grid-cols-4',
        'lg:grid-rows-[repeat(2,minmax(300px,1fr))]',
        className
      )}
      style={{
        padding: `${spacing.md}px`,
      }}
    >
      {children}
    </div>
  );
}

// Widget grid area variants
interface WidgetProps {
  children: ReactNode;
  className?: string;
  size?: 'small' | 'medium' | 'large';
}

export function BentoWidget({ children, className, size = 'medium' }: WidgetProps) {
  const sizeClasses = {
    small: 'lg:col-span-1 lg:row-span-1',
    medium: 'lg:col-span-1 lg:row-span-1 md:col-span-1',
    large: 'lg:col-span-2 lg:row-span-2 md:col-span-2',
  };

  return (
    <div
      className={cn(
        'col-span-1',
        sizeClasses[size],
        className
      )}
    >
      {children}
    </div>
  );
}
