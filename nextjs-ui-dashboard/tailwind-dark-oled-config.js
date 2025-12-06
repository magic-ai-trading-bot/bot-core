/**
 * Tailwind CSS Configuration - Dark OLED Luxury Theme
 *
 * Cryptocurrency Trading Dashboard
 * Based on TradingView, Binance, Bybit, and Coinbase Pro design research
 *
 * @spec:COMP-FRONTEND-DASHBOARD
 * @ref:docs/design-system-cryptocurrency-trading-dashboard.md
 */

module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
    './src/hooks/**/*.{js,ts,jsx,tsx,mdx}',
  ],

  theme: {
    extend: {
      // ============================================================
      // COLOR PALETTE - Dark OLED Luxury
      // ============================================================
      colors: {
        // Primary Background (True Black - OLED Optimized)
        'bg': {
          primary: '#000000',      // Pure black - OLED power saving
          secondary: '#0F0F1E',    // Deep navy
          tertiary: '#1A1A2E',     // Slate
          quaternary: '#16213E',   // Rich dark blue
          surface: '#0D0D14',      // Elevated surface
        },

        // Text Colors (High Contrast)
        'text': {
          primary: '#FFFFFF',      // Pure white
          secondary: '#B0B0C0',    // Light gray-blue
          tertiary: '#7A7A8E',     // Medium gray
          muted: '#4A4A5E',        // Muted gray-blue
        },

        // Financial Status Colors
        'financial': {
          profit: '#10B981',       // Bullish Green
          'profit-hover': '#059669',
          'profit-bg': 'rgba(16, 185, 129, 0.08)',
          'profit-border': 'rgba(16, 185, 129, 0.2)',

          loss: '#EF4444',         // Bearish Red
          'loss-hover': '#DC2626',
          'loss-bg': 'rgba(239, 68, 68, 0.08)',
          'loss-border': 'rgba(239, 68, 68, 0.2)',

          neutral: '#6B7280',      // Neutral Gray
          'neutral-hover': '#9CA3AF',

          warning: '#F59E0B',      // Caution Orange
          'warning-hover': '#D97706',
          'warning-bg': 'rgba(245, 158, 11, 0.08)',

          success: '#10B981',      // Confirmed Green
        },

        // Brand & Accent Colors
        'brand': {
          blue: '#2962FF',         // Premium Blue (TradingView)
          'blue-hover': '#2257E7',
          'blue-light': '#5B7FFF',
          'blue-bg': 'rgba(41, 98, 255, 0.08)',
        },

        'accent': {
          cyan: '#00D9FF',         // Neon Cyan
          'cyan-soft': 'rgba(0, 217, 255, 0.15)',

          purple: '#A855F7',       // Vibrant Purple
          'purple-light': '#D4A5FF',

          gold: '#F3BA2F',         // Bitcoin Gold
          'gold-dim': 'rgba(243, 186, 47, 0.15)',

          silver: '#C0C0D0',       // Ethereum Silver
        },

        // Borders
        'border': {
          primary: 'rgba(176, 176, 192, 0.1)',
          secondary: 'rgba(176, 176, 192, 0.2)',
          accent: 'rgba(41, 98, 255, 0.3)',
          danger: 'rgba(239, 68, 68, 0.3)',
          success: 'rgba(16, 185, 129, 0.3)',
        },

        // Legacy Support (backward compatibility)
        'gray': {
          50: 'rgba(255, 255, 255, 1)',
          100: 'rgba(245, 245, 245, 1)',
          200: '#B0B0C0',
          300: '#7A7A8E',
          400: '#6B7280',
          500: '#4A4A5E',
          600: '#3A3A4E',
          700: '#2A2A3E',
          800: '#1A1A2E',
          900: '#0F0F1E',
        },

        'blue': {
          50: 'rgba(41, 98, 255, 0.05)',
          100: 'rgba(41, 98, 255, 0.1)',
          500: '#2962FF',
          600: '#2257E7',
          700: '#1E4DC7',
        },

        'green': {
          50: 'rgba(16, 185, 129, 0.05)',
          100: 'rgba(16, 185, 129, 0.1)',
          500: '#10B981',
          600: '#059669',
        },

        'red': {
          50: 'rgba(239, 68, 68, 0.05)',
          100: 'rgba(239, 68, 68, 0.1)',
          500: '#EF4444',
          600: '#DC2626',
        },

        'amber': {
          50: 'rgba(245, 158, 11, 0.05)',
          100: 'rgba(245, 158, 11, 0.1)',
          500: '#F59E0B',
          600: '#D97706',
        },

        'cyan': {
          50: 'rgba(0, 217, 255, 0.05)',
          100: 'rgba(0, 217, 255, 0.1)',
          500: '#00D9FF',
          600: '#00B8CC',
        },

        'purple': {
          50: 'rgba(168, 85, 247, 0.05)',
          100: 'rgba(168, 85, 247, 0.1)',
          500: '#A855F7',
          600: '#9333EA',
        },
      },

      // ============================================================
      // TYPOGRAPHY
      // ============================================================
      fontFamily: {
        // Sans - Clean, Modern
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          '\'Segoe UI\'',
          '\'Roboto\'',
          '\'Oxygen\'',
          '\'Ubuntu\'',
          '\'Cantarell\'',
          '\'Fira Sans\'',
          '\'Droid Sans\'',
          '\'Helvetica Neue\'',
          'sans-serif',
        ],

        // Mono - Technical, Code-like
        mono: [
          '\'JetBrains Mono\'',
          '\'Fira Code\'',
          '\'Courier New\'',
          'monospace',
        ],

        // Display - Headlines, Premium
        display: [
          '\'Inter\'',
          '\'Poppins\'',
          '\'Space Grotesk\'',
          'sans-serif',
        ],
      },

      // Font Sizes - Modular Scale (1.125x)
      fontSize: {
        // Display
        'display-lg': ['3.5rem', { lineHeight: '1.2', letterSpacing: '-0.01em' }],
        'display-md': ['2.75rem', { lineHeight: '1.2', letterSpacing: '-0.01em' }],
        'display-sm': ['2.125rem', { lineHeight: '1.2', letterSpacing: '-0.01em' }],

        // Headings
        'heading-1': ['1.75rem', { lineHeight: '1.2', fontWeight: '700' }],
        'heading-2': ['1.406rem', { lineHeight: '1.375', fontWeight: '600' }],
        'heading-3': ['1.266rem', { lineHeight: '1.375', fontWeight: '600' }],
        'heading-4': ['1.125rem', { lineHeight: '1.4', fontWeight: '600' }],

        // Body
        'body-lg': ['1rem', { lineHeight: '1.5' }],
        'body-md': ['0.938rem', { lineHeight: '1.5' }],
        'body-sm': ['0.875rem', { lineHeight: '1.5' }],
        'body-xs': ['0.75rem', { lineHeight: '1.5' }],

        // UI
        'ui-lg': ['0.938rem', { lineHeight: '1.375', fontWeight: '600' }],
        'ui-md': ['0.875rem', { lineHeight: '1.375', fontWeight: '500' }],
        'ui-sm': ['0.75rem', { lineHeight: '1.2', fontWeight: '500' }],

        // Code/Data
        'code-lg': ['0.938rem', { lineHeight: '1.5', fontFamily: 'JetBrains Mono' }],
        'code-md': ['0.875rem', { lineHeight: '1.5', fontFamily: 'JetBrains Mono' }],
        'code-sm': ['0.75rem', { lineHeight: '1.4', fontFamily: 'JetBrains Mono' }],

        // Default
        xs: ['0.75rem', { lineHeight: '1rem' }],
        sm: ['0.875rem', { lineHeight: '1.25rem' }],
        base: ['1rem', { lineHeight: '1.5rem' }],
        lg: ['1.125rem', { lineHeight: '1.75rem' }],
        xl: ['1.25rem', { lineHeight: '1.75rem' }],
        '2xl': ['1.5rem', { lineHeight: '2rem' }],
      },

      fontWeight: {
        light: 300,
        normal: 400,
        medium: 500,
        semibold: 600,
        bold: 700,
        extrabold: 800,
      },

      lineHeight: {
        tight: 1.2,
        snug: 1.375,
        normal: 1.5,
        relaxed: 1.625,
        loose: 2,
      },

      letterSpacing: {
        tight: '-0.01em',
        normal: '0em',
        wide: '0.025em',
        wider: '0.05em',
      },

      // ============================================================
      // SPACING (8px Base Unit)
      // ============================================================
      spacing: {
        0: '0',
        0.5: '0.125rem',    // 2px
        1: '0.25rem',       // 4px
        1.5: '0.375rem',    // 6px
        2: '0.5rem',        // 8px
        2.5: '0.625rem',    // 10px
        3: '0.75rem',       // 12px
        3.5: '0.875rem',    // 14px
        4: '1rem',          // 16px
        5: '1.25rem',       // 20px
        6: '1.5rem',        // 24px
        7: '1.75rem',       // 28px
        8: '2rem',          // 32px
        10: '2.5rem',       // 40px
        12: '3rem',         // 48px
        14: '3.5rem',       // 56px
        16: '4rem',         // 64px
        20: '5rem',         // 80px
        24: '6rem',         // 96px
      },

      gap: {
        0: '0',
        1: '0.25rem',
        2: '0.5rem',
        3: '0.75rem',
        4: '1rem',
        6: '1.5rem',       // Standard gap
        8: '2rem',         // Spacious gap
      },

      // ============================================================
      // BORDER RADIUS
      // ============================================================
      borderRadius: {
        none: '0',
        xs: '0.25rem',      // 4px
        sm: '0.375rem',     // 6px
        base: '0.5rem',     // 8px - Standard button radius
        md: '0.75rem',      // 12px - Card radius
        lg: '1rem',         // 16px - Large components
        full: '9999px',     // Fully rounded (badges)
      },

      // ============================================================
      // BOX SHADOWS
      // ============================================================
      boxShadow: {
        none: 'none',

        // Elevation shadows
        'sm': '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
        'base': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)',
        'md': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
        'lg': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
        'xl': '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
        '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',

        // Card shadows (dark-optimized)
        'card': '0 4px 6px rgba(0, 0, 0, 0.1), 0 1px 3px rgba(0, 0, 0, 0.08)',
        'card-hover': '0 8px 12px rgba(0, 0, 0, 0.15), 0 2px 4px rgba(0, 0, 0, 0.1)',

        // Glow effects
        'glow-blue': '0 0 20px rgba(41, 98, 255, 0.3), 0 4px 6px rgba(0, 0, 0, 0.1)',
        'glow-blue-intense': '0 0 30px rgba(41, 98, 255, 0.5), 0 0 60px rgba(41, 98, 255, 0.25)',
        'glow-cyan': '0 0 20px rgba(0, 217, 255, 0.3), 0 4px 6px rgba(0, 0, 0, 0.1)',
        'glow-green': '0 0 15px rgba(16, 185, 129, 0.3), 0 0 30px rgba(16, 185, 129, 0.15)',
        'glow-red': '0 0 15px rgba(239, 68, 68, 0.3), 0 0 30px rgba(239, 68, 68, 0.15)',

        // Glassmorphism
        'glass': '0 8px 32px rgba(0, 0, 0, 0.1)',
        'glass-inset': 'inset 1px 1px 0 rgba(255, 255, 255, 0.1), inset -1px -1px 0 rgba(0, 0, 0, 0.1)',

        // Inner shadows
        'inner': 'inset 0 2px 4px rgba(0, 0, 0, 0.05)',
        'inner-md': 'inset 0 4px 8px rgba(0, 0, 0, 0.1)',
      },

      // ============================================================
      // BACKDROP FILTER (Glassmorphism)
      // ============================================================
      backdropBlur: {
        none: '0',
        xs: '10px',
        sm: '12px',
        md: '20px',
        lg: '30px',
        xl: '40px',
      },

      backdropOpacity: {
        0: '0',
        5: '0.05',
        10: '0.1',
        20: '0.2',
        25: '0.25',
        30: '0.3',
        40: '0.4',
        50: '0.5',
        60: '0.6',
        70: '0.7',
        75: '0.75',
        80: '0.8',
        90: '0.9',
        95: '0.95',
        100: '1',
      },

      // ============================================================
      // GRADIENTS
      // ============================================================
      backgroundImage: {
        // Premium gradients
        'gradient-premium': 'linear-gradient(135deg, #0F0F1E 0%, #16213E 100%)',
        'gradient-cyan': 'linear-gradient(135deg, rgba(0, 217, 255, 0.2) 0%, rgba(0, 217, 255, 0.05) 100%)',
        'gradient-neon': 'linear-gradient(135deg, #2962FF 0%, #00D9FF 100%)',

        // Status gradients
        'gradient-profit': 'linear-gradient(135deg, rgba(16, 185, 129, 0.1) 0%, rgba(16, 185, 129, 0.05) 100%)',
        'gradient-loss': 'linear-gradient(135deg, rgba(239, 68, 68, 0.1) 0%, rgba(239, 68, 68, 0.05) 100%)',

        // Loading
        'gradient-shimmer': 'linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent)',
      },

      backgroundSize: {
        'shimmer': '1000px 100%',
      },

      // ============================================================
      // ANIMATION & TRANSITIONS
      // ============================================================
      animation: {
        none: 'none',
        spin: 'spin 1s linear infinite',
        ping: 'ping 1s cubic-bezier(0, 0, 0.2, 1) infinite',
        pulse: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        bounce: 'bounce 1s infinite',

        // Custom animations
        'fade-in': 'fade-in 0.3s ease-out',
        'slide-up': 'slide-up 0.3s ease-out',
        'shimmer': 'shimmer 2s infinite',
        'candle-draw': 'candle-draw 0.4s ease-out forwards',
        'price-pulse': 'price-pulse 0.6s ease-out',
      },

      keyframes: {
        // TailwindCSS defaults
        spin: {
          to: { transform: 'rotate(360deg)' },
        },
        ping: {
          '75%, 100%': { transform: 'scale(2)', opacity: '0' },
        },
        pulse: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '.5' },
        },
        bounce: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-25%)' },
        },

        // Custom keyframes
        'fade-in': {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        'slide-up': {
          from: { opacity: '0', transform: 'translateY(20px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        'shimmer': {
          '0%': { backgroundPosition: '-1000px 0' },
          '100%': { backgroundPosition: '1000px 0' },
        },
        'candle-draw': {
          from: { opacity: '0', transform: 'scaleY(0)' },
          to: { opacity: '1', transform: 'scaleY(1)' },
        },
        'price-pulse': {
          '0%': { backgroundColor: 'rgba(16, 185, 129, 0.2)' },
          '100%': { backgroundColor: 'transparent' },
        },
      },

      transitionDuration: {
        0: '0ms',
        75: '75ms',
        100: '100ms',
        150: '150ms',
        200: '200ms',
        300: '300ms',
        500: '500ms',
        700: '700ms',
        1000: '1000ms',
      },

      transitionTimingFunction: {
        'ease-out': 'cubic-bezier(0.0, 0, 0.2, 1)',
        'ease-in': 'cubic-bezier(0.4, 0, 1, 1)',
        'ease-in-out': 'cubic-bezier(0.4, 0, 0.2, 1)',
        'ease-bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      },

      // ============================================================
      // OTHER UTILITIES
      // ============================================================
      opacity: {
        0: '0',
        5: '0.05',
        10: '0.1',
        20: '0.2',
        25: '0.25',
        30: '0.3',
        40: '0.4',
        50: '0.5',
        60: '0.6',
        70: '0.7',
        75: '0.75',
        80: '0.8',
        90: '0.9',
        95: '0.95',
        100: '1',
      },

      scale: {
        0: '0',
        50: '.5',
        75: '.75',
        90: '.9',
        95: '.95',
        100: '1',
        105: '1.05',
        110: '1.1',
        125: '1.25',
        150: '1.5',
      },

      transform: [
        'responsive',
        'hover',
        'focus',
        'group-hover',
      ],

      screens: {
        xs: '480px',
        sm: '640px',
        md: '768px',
        lg: '1024px',
        xl: '1280px',
        '2xl': '1536px',
      },
    },
  },

  // ============================================================
  // PLUGINS
  // ============================================================
  plugins: [
    // Custom utilities for glassmorphism
    function ({ addComponents, theme }) {
      addComponents({
        '.glassmorphic': {
          background: 'rgba(26, 26, 46, 0.7)',
          backdropFilter: 'blur(20px)',
          WebkitBackdropFilter: 'blur(20px)',
          border: '1px solid rgba(176, 176, 192, 0.15)',
        },
        '.glassmorphic-tight': {
          background: 'rgba(26, 26, 46, 0.8)',
          backdropFilter: 'blur(12px)',
          WebkitBackdropFilter: 'blur(12px)',
          border: '1px solid rgba(176, 176, 192, 0.2)',
        },
        '.glassmorphic-soft': {
          background: 'rgba(26, 26, 46, 0.6)',
          backdropFilter: 'blur(30px)',
          WebkitBackdropFilter: 'blur(30px)',
          border: '1px solid rgba(176, 176, 192, 0.1)',
        },

        // Financial components
        '.financial-up': {
          color: theme('colors.financial.profit'),
          backgroundColor: theme('colors.financial.profit-bg'),
          borderColor: theme('colors.financial.profit-border'),
        },
        '.financial-down': {
          color: theme('colors.financial.loss'),
          backgroundColor: theme('colors.financial.loss-bg'),
          borderColor: theme('colors.financial.loss-border'),
        },

        // Trading buttons
        '.btn-buy': {
          backgroundColor: theme('colors.financial.profit'),
          '&:hover': {
            backgroundColor: theme('colors.financial.profit-hover'),
            boxShadow: '0 6px 20px rgba(16, 185, 129, 0.4)',
          },
        },
        '.btn-sell': {
          backgroundColor: theme('colors.financial.loss'),
          '&:hover': {
            backgroundColor: theme('colors.financial.loss-hover'),
            boxShadow: '0 6px 20px rgba(239, 68, 68, 0.4)',
          },
        },

        // Focus visible for accessibility
        '.focus-ring': {
          '&:focus-visible': {
            outline: '2px solid',
            outlineColor: theme('colors.brand.blue'),
            outlineOffset: '2px',
          },
        },
      });
    },

    // Reduce motion for accessibility
    function ({ addVariant }) {
      addVariant('no-motion', '@media (prefers-reduced-motion: reduce)');
      addVariant('motion-safe', '@media (prefers-reduced-motion: no-preference)');
    },
  ],

  // ============================================================
  // IMPORTANT SETTINGS
  // ============================================================
  important: false,
  corePlugins: {
    preflight: true,
  },
  darkMode: false, // Dark mode disabled - this IS the dark theme
};
