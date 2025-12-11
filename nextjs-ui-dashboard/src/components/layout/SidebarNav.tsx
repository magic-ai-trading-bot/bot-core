/**
 * SidebarNav Component
 *
 * Navigation items configuration with icons and active state detection.
 */

import { NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
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
import { useThemeColors } from '@/hooks/useThemeColors';

interface NavItem {
  labelKey: string;
  icon: LucideIcon;
  path: string;
}

interface SidebarNavProps {
  isExpanded: boolean;
  onNavigate?: () => void; // Callback for mobile to close drawer
}

const navItems: NavItem[] = [
  { labelKey: 'nav.dashboard', icon: LayoutDashboard, path: '/dashboard' },
  { labelKey: 'nav.paperTrading', icon: TestTube, path: '/trading/paper' },
  { labelKey: 'nav.realTrading', icon: CircleDollarSign, path: '/trading/real' },
  { labelKey: 'nav.portfolio', icon: PieChart, path: '/portfolio' },
  { labelKey: 'nav.signals', icon: Brain, path: '/signals' },
  { labelKey: 'nav.tradeAnalyses', icon: BarChart3, path: '/trade-analyses' },
  { labelKey: 'nav.howItWorks', icon: HelpCircle, path: '/how-it-works' },
  { labelKey: 'nav.settings', icon: Settings, path: '/settings' },
];

export function SidebarNav({ isExpanded, onNavigate }: SidebarNavProps) {
  const { t } = useTranslation('common');
  const colors = useThemeColors();

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
            backgroundColor: isActive ? colors.bgTertiary : 'transparent',
            color: isActive ? colors.textPrimary : colors.textMuted,
            border: isActive ? `1px solid ${colors.borderSubtle}` : '1px solid transparent',
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
                    color: isActive ? colors.cyan : colors.textMuted,
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
                {t(item.labelKey)}
              </motion.span>

              {/* Active indicator dot */}
              {isActive && isExpanded && (
                <motion.div
                  layoutId="activeIndicator"
                  className="ml-auto h-2 w-2 rounded-full"
                  style={{
                    backgroundColor: colors.cyan,
                    boxShadow: colors.glowCyan,
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
