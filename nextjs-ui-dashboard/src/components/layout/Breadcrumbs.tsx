/**
 * Breadcrumbs Component
 *
 * Dynamic breadcrumbs from react-router with chevron separators.
 * Automatically generates breadcrumbs from current route path.
 */

import { Link, useLocation } from 'react-router-dom';
import { ChevronRight, Home } from 'lucide-react';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { duration } from '@/styles/tokens/animations';
import { colors } from '@/styles/tokens/colors';

interface BreadcrumbItem {
  label: string;
  path: string;
  isClickable: boolean;
}

// Valid routes that exist in the app
const VALID_ROUTES = new Set([
  '/dashboard',
  '/trading/paper',
  '/trading/real',
  '/portfolio',
  '/signals',
  '/settings',
  '/profile',
  '/trade-analyses',
]);

export function Breadcrumbs() {
  const location = useLocation();

  // Generate breadcrumbs from pathname
  const breadcrumbs: BreadcrumbItem[] = [];

  // Always add home
  breadcrumbs.push({ label: 'Home', path: '/dashboard', isClickable: true });

  // Parse pathname segments
  const segments = location.pathname.split('/').filter(Boolean);

  let currentPath = '';
  segments.forEach((segment) => {
    currentPath += `/${segment}`;

    // Skip dashboard as it's already added as Home
    if (segment === 'dashboard') return;

    // Format label (capitalize, replace dashes with spaces)
    const label = segment
      .split('-')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');

    // Only make the path clickable if it's a valid route
    const isClickable = VALID_ROUTES.has(currentPath);
    breadcrumbs.push({ label, path: currentPath, isClickable });
  });

  // Don't show breadcrumbs on home page
  if (breadcrumbs.length <= 1) {
    return null;
  }

  return (
    <nav aria-label="Breadcrumb" className="flex items-center gap-1">
      {breadcrumbs.map((crumb, index) => {
        const isLast = index === breadcrumbs.length - 1;
        const isFirst = index === 0;

        return (
          <motion.div
            key={crumb.path}
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{
              duration: duration.fast,
              delay: index * 0.05,
            }}
            className="flex items-center gap-1"
          >
            {/* Breadcrumb item */}
            {isLast ? (
              <span
                className="text-sm font-medium"
                style={{ color: colors.text.primary }}
                aria-current="page"
              >
                {crumb.label}
              </span>
            ) : crumb.isClickable ? (
              <Link
                to={crumb.path}
                className={cn(
                  'flex items-center gap-1.5 text-sm font-medium transition-colors',
                  'hover:text-white focus-visible:outline-none focus-visible:ring-2',
                  'focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900 rounded px-1'
                )}
                style={{ color: colors.text.secondary }}
              >
                {isFirst && <Home className="h-3.5 w-3.5" />}
                {crumb.label}
              </Link>
            ) : (
              <span
                className="flex items-center gap-1.5 text-sm font-medium"
                style={{ color: colors.text.muted }}
              >
                {isFirst && <Home className="h-3.5 w-3.5" />}
                {crumb.label}
              </span>
            )}

            {/* Separator */}
            {!isLast && (
              <ChevronRight
                className="h-4 w-4"
                style={{ color: colors.text.muted }}
              />
            )}
          </motion.div>
        );
      })}
    </nav>
  );
}
