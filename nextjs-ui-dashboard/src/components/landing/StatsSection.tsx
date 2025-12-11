/**
 * StatsSection - Key Metrics Section with Animated Counters
 *
 * Features:
 * - Animated counters triggered by IntersectionObserver
 * - Stats: Total volume, Active users, Accuracy rate, Uptime
 * - Gradient background
 * - Responsive grid layout
 */

import { motion } from 'framer-motion';
import { TrendingUp, Users, Target, Zap } from 'lucide-react';
import { ShortAnimatedCounter } from './AnimatedCounter';

interface StatCardProps {
  icon: React.ReactNode;
  value: number;
  suffix?: string;
  prefix?: string;
  label: string;
  color: 'blue' | 'green' | 'purple' | 'yellow';
  delay?: number;
}

function StatCard({ icon, value, suffix, prefix, label, color, delay = 0 }: StatCardProps) {
  const colorClasses = {
    blue: {
      iconBg: 'bg-blue-500/10',
      iconText: 'text-blue-400',
      valueBg: 'from-blue-500/20 to-blue-600/10',
      border: 'border-blue-500/20',
    },
    green: {
      iconBg: 'bg-emerald-500/10',
      iconText: 'text-emerald-400',
      valueBg: 'from-emerald-500/20 to-emerald-600/10',
      border: 'border-emerald-500/20',
    },
    purple: {
      iconBg: 'bg-purple-500/10',
      iconText: 'text-purple-400',
      valueBg: 'from-purple-500/20 to-purple-600/10',
      border: 'border-purple-500/20',
    },
    yellow: {
      iconBg: 'bg-yellow-500/10',
      iconText: 'text-yellow-400',
      valueBg: 'from-yellow-500/20 to-yellow-600/10',
      border: 'border-yellow-500/20',
    },
  };

  const colors = colorClasses[color];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-100px' }}
      transition={{ duration: 0.5, delay, ease: 'easeOut' }}
      whileHover={{ y: -4, scale: 1.02 }}
      className={`relative p-8 rounded-xl bg-slate-900/70 backdrop-blur-md border ${colors.border} shadow-xl overflow-hidden group`}
    >
      {/* Background gradient effect */}
      <div className={`absolute inset-0 bg-gradient-to-br ${colors.valueBg} opacity-0 group-hover:opacity-100 transition-opacity duration-300`} />

      {/* Content */}
      <div className="relative z-10">
        {/* Icon */}
        <div className={`inline-flex items-center justify-center w-14 h-14 rounded-lg ${colors.iconBg} mb-4`}>
          <div className={colors.iconText}>{icon}</div>
        </div>

        {/* Value */}
        <div className="mb-2">
          <ShortAnimatedCounter
            end={value}
            suffix={suffix}
            prefix={prefix}
            decimals={suffix === '%' ? 0 : 1}
            className="text-4xl lg:text-5xl font-bold text-gray-100"
          />
        </div>

        {/* Label */}
        <p className="text-gray-400 text-sm font-medium">{label}</p>
      </div>

      {/* Decorative corner accent */}
      <div className={`absolute top-0 right-0 w-20 h-20 bg-gradient-to-br ${colors.valueBg} rounded-bl-full opacity-30`} />
    </motion.div>
  );
}

export function StatsSection() {
  return (
    <section className="py-24 bg-gradient-to-b from-slate-900 to-slate-950 relative overflow-hidden">
      {/* Background decorative elements */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-500/5 via-transparent to-transparent" />
      <div className="absolute top-0 left-1/4 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl" />
      <div className="absolute bottom-0 right-1/4 w-64 h-64 bg-blue-500/10 rounded-full blur-3xl" />

      <div className="container mx-auto px-4 relative z-10">
        {/* Section header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-blue-500/10 text-blue-400 text-sm font-medium mb-6 border border-blue-500/20">
            <TrendingUp className="w-4 h-4" />
            Platform Statistics
          </span>

          <h2 className="text-4xl lg:text-5xl font-bold text-gray-100 mb-4">
            Trusted by Traders
            <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-500">
              Worldwide
            </span>
          </h2>

          <p className="text-xl text-gray-400 max-w-2xl mx-auto">
            Join thousands of successful traders using our AI-powered platform
          </p>
        </motion.div>

        {/* Stats grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 lg:gap-8">
          <StatCard
            icon={<TrendingUp className="w-7 h-7" strokeWidth={1.5} />}
            value={52}
            suffix="M+"
            label="Trading Volume"
            color="blue"
            delay={0}
          />

          <StatCard
            icon={<Users className="w-7 h-7" strokeWidth={1.5} />}
            value={2847}
            label="Active Traders"
            color="green"
            delay={0.1}
          />

          <StatCard
            icon={<Target className="w-7 h-7" strokeWidth={1.5} />}
            value={72}
            suffix="%"
            label="Win Rate"
            color="purple"
            delay={0.2}
          />

          <StatCard
            icon={<Zap className="w-7 h-7" strokeWidth={1.5} />}
            value={99.9}
            suffix="%"
            label="Uptime"
            color="yellow"
            delay={0.3}
          />
        </div>

        {/* Additional info */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.4 }}
          className="mt-12 text-center text-gray-500 text-sm"
        >
          <p>
            Updated in real-time • Trusted by professional traders since 2023
          </p>
        </motion.div>
      </div>
    </section>
  );
}
