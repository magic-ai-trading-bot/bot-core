/**
 * Design Tokens - Animations
 *
 * Framer Motion animation variants for consistent, smooth transitions.
 * All animations respect prefers-reduced-motion for accessibility.
 */

import { Variants } from 'framer-motion';

// Animation duration constants
export const duration = {
  instant: 0.1,
  fast: 0.2,
  normal: 0.3,
  slow: 0.4,
  slower: 0.6,
} as const;

// Easing functions
export const easing = {
  easeOut: [0.0, 0.0, 0.2, 1],
  easeIn: [0.4, 0.0, 1, 1],
  easeInOut: [0.4, 0.0, 0.2, 1],
  spring: { type: 'spring', stiffness: 300, damping: 30 },
} as const;

// Fade in/out
export const fadeIn: Variants = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: duration.normal },
};

// Slide up from bottom
export const slideUp: Variants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
  transition: { duration: duration.normal, ease: easing.easeOut },
};

// Slide down from top
export const slideDown: Variants = {
  initial: { opacity: 0, y: -20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 20 },
  transition: { duration: duration.normal, ease: easing.easeOut },
};

// Scale in/out (for modals, popovers)
export const scaleIn: Variants = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.95 },
  transition: { duration: duration.fast, ease: easing.easeOut },
};

// Number change animation (for prices, P&L)
export const numberChange: Variants = {
  initial: { scale: 1 },
  animate: { scale: [1, 1.05, 1] },
  transition: { duration: duration.normal },
};

// Pulse animation (for real mode badge)
export const pulse: Variants = {
  animate: {
    scale: [1, 1.05, 1],
    opacity: [1, 0.8, 1],
  },
  transition: {
    duration: 2,
    repeat: Infinity,
    ease: 'easeInOut',
  },
};

// Flash animation (for price changes)
export const flash = {
  green: {
    animate: {
      backgroundColor: [
        'rgba(16, 185, 129, 0)',
        'rgba(16, 185, 129, 0.3)',
        'rgba(16, 185, 129, 0)',
      ],
    },
    transition: { duration: 0.5 },
  },
  red: {
    animate: {
      backgroundColor: [
        'rgba(239, 68, 68, 0)',
        'rgba(239, 68, 68, 0.3)',
        'rgba(239, 68, 68, 0)',
      ],
    },
    transition: { duration: 0.5 },
  },
} as const;

// Stagger children animation (for lists)
export const stagger = {
  container: {
    animate: {
      transition: {
        staggerChildren: 0.1,
      },
    },
  },
  item: {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: duration.normal },
  },
} as const;

// Hover effects
export const hoverScale = {
  whileHover: { scale: 1.02 },
  whileTap: { scale: 0.98 },
  transition: { duration: duration.fast },
};

export const hoverLift = {
  whileHover: { y: -2, boxShadow: '0 8px 24px rgba(0, 0, 0, 0.3)' },
  transition: { duration: duration.fast },
};

// Loading animations
export const spinner: Variants = {
  animate: {
    rotate: 360,
  },
  transition: {
    duration: 1,
    repeat: Infinity,
    ease: 'linear',
  },
};

export const shimmer: Variants = {
  animate: {
    x: ['-100%', '100%'],
  },
  transition: {
    duration: 1.5,
    repeat: Infinity,
    ease: 'easeInOut',
  },
};

// Notification slide-in from right
export const notificationSlideIn: Variants = {
  initial: { opacity: 0, x: 300 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: 300 },
  transition: { duration: duration.normal, ease: easing.easeOut },
};

// Type exports
export type AnimationVariant = Variants;
export type DurationKey = keyof typeof duration;
export type EasingKey = keyof typeof easing;

// Preset animation combinations
export const animationPresets = {
  // Card entrance
  card: slideUp,

  // Modal/Dialog
  modal: scaleIn,

  // Notification
  notification: notificationSlideIn,

  // Price update
  priceUpdate: numberChange,

  // Page transition
  page: fadeIn,

  // List items
  list: stagger,
} as const;

export type AnimationPresetKey = keyof typeof animationPresets;
