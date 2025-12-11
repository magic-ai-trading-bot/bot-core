/**
 * FeatureCard - Individual Feature Card Component
 *
 * Features:
 * - Icon with hover animation
 * - Title + description
 * - GlassCard style
 * - Smooth hover effects
 */

import { motion } from 'framer-motion';
import { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';
import { hoverLift } from '@/styles/tokens/animations';

interface FeatureCardProps {
  icon: LucideIcon;
  title: string;
  description: string;
  badge?: string;
  className?: string;
  delay?: number;
}

export function FeatureCard({
  icon: Icon,
  title,
  description,
  badge,
  className,
  delay = 0,
}: FeatureCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-100px' }}
      transition={{ duration: 0.4, delay, ease: 'easeOut' }}
      {...hoverLift}
      className={cn(
        'group relative p-6 rounded-xl border bg-slate-900/70 backdrop-blur-md',
        'border-slate-700/50 shadow-xl hover:border-blue-500/30',
        'transition-all duration-300',
        className
      )}
    >
      {/* Icon container with glow effect */}
      <div className="mb-4 inline-flex items-center justify-center w-12 h-12 rounded-lg bg-blue-500/10 group-hover:bg-blue-500/20 transition-colors">
        <Icon
          className="w-6 h-6 text-blue-400 group-hover:text-blue-300 transition-colors"
          strokeWidth={1.5}
        />
      </div>

      {/* Title with badge */}
      <div className="flex items-center gap-2 mb-3">
        <h3 className="text-lg font-semibold text-gray-100 group-hover:text-blue-300 transition-colors">
          {title}
        </h3>
        {badge && (
          <span className="px-2 py-0.5 text-xs font-medium rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">
            {badge}
          </span>
        )}
      </div>

      {/* Description */}
      <p className="text-sm text-gray-400 leading-relaxed">
        {description}
      </p>

      {/* Hover gradient effect */}
      <div className="absolute inset-0 rounded-xl bg-gradient-to-br from-blue-500/5 via-transparent to-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
    </motion.div>
  );
}
