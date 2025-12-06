/**
 * SidebarNav Component
 *
 * Navigation items configuration with icons and active state detection.
 */

import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  TestTube,
  CircleDollarSign,
  PieChart,
  Brain,
  BarChart3,
  Settings,
  HelpCircle,
  type LucideIcon,
} from 'lucide-react';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { duration, easing } from '@/styles/tokens/animations';
import { luxuryColors } from '@/styles/luxury-design-system';

interface NavItem {
  label: string;
  icon: LucideIcon;
  path: string;
}

interface SidebarNavProps {
  isExpanded: boolean;
  onNavigate?: () => void; // Callback for mobile to close drawer
}

const navItems: NavItem[] = [
  { label: 'Dashboard', icon: LayoutDashboard, path: '/dashboard' },
  { label: 'Paper Trading', icon: TestTube, path: '/trading/paper' },
  { label: 'Real Trading', icon: CircleDollarSign, path: '/trading/real' },
  { label: 'Portfolio', icon: PieChart, path: '/portfolio' },
  { label: 'AI Signals', icon: Brain, path: '/signals' },
  { label: 'Trade Analyses', icon: BarChart3, path: '/trade-analyses' },
  { label: 'How It Works', icon: HelpCircle, path: '/how-it-works' },
  { label: 'Settings', icon: Settings, path: '/settings' },
];

export function SidebarNav({ isExpanded, onNavigate }: SidebarNavProps) {
  return (
    <nav className="flex-1 space-y-1 px-3 py-4" aria-label="Main navigation">
      {navItems.map((item) => (
        <NavLink
          key={item.path}
          to={item.path}
          onClick={onNavigate}
          className={({ isActive }) =>
            cn(
              'flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-200',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-black'
            )
          }
          style={({ isActive }) => ({
            backgroundColor: isActive ? luxuryColors.bgTertiary : 'transparent',
            color: isActive ? luxuryColors.textPrimary : luxuryColors.textMuted,
            border: isActive ? `1px solid ${luxuryColors.borderSubtle}` : '1px solid transparent',
          })}
        >
          {({ isActive }) => (
            <>
              <motion.div
                initial={false}
                animate={{
                  scale: isActive ? 1.1 : 1,
                }}
                transition={{
                  duration: duration.normal,
                  ease: easing.easeOut,
                }}
              >
                <item.icon
                  className="h-5 w-5 flex-shrink-0"
                  style={{
                    color: isActive ? luxuryColors.cyan : luxuryColors.textMuted,
                  }}
                />
              </motion.div>

              {/* Label with fade animation when collapsing */}
              <motion.span
                initial={false}
                animate={{
                  opacity: isExpanded ? 1 : 0,
                  width: isExpanded ? 'auto' : 0,
                }}
                transition={{
                  duration: duration.fast,
                  ease: easing.easeOut,
                }}
                className="overflow-hidden whitespace-nowrap"
              >
                {item.label}
              </motion.span>

              {/* Active indicator dot */}
              {isActive && isExpanded && (
                <motion.div
                  layoutId="activeIndicator"
                  className="ml-auto h-2 w-2 rounded-full"
                  style={{
                    backgroundColor: luxuryColors.cyan,
                    boxShadow: luxuryColors.glowCyan,
                  }}
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{
                    type: 'spring',
                    stiffness: 300,
                    damping: 25,
                  }}
                />
              )}
            </>
          )}
        </NavLink>
      ))}
    </nav>
  );
}
