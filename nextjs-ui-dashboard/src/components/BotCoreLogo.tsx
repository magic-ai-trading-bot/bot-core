import { useThemeColors } from '@/hooks/useThemeColors';

interface BotCoreLogoProps {
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showText?: boolean;
  className?: string;
}

const sizes = {
  sm: { icon: 32, text: 'text-base' },
  md: { icon: 40, text: 'text-xl' },
  lg: { icon: 48, text: 'text-2xl' },
  xl: { icon: 64, text: 'text-3xl' },
};

export const BotCoreLogo = ({ size = 'md', showText = true, className = '' }: BotCoreLogoProps) => {
  const colors = useThemeColors();
  const { icon: iconSize, text: textSize } = sizes[size];

  return (
    <div className={`flex items-center gap-3 ${className}`}>
      {/* Logo Icon - Modern AI Trading Bot */}
      <svg
        width={iconSize}
        height={iconSize}
        viewBox="0 0 48 48"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{
          filter: `drop-shadow(0 2px 8px ${colors.cyan}40)`,
        }}
      >
        <defs>
          <linearGradient id="logoGradientMain" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#22d3ee" />
            <stop offset="100%" stopColor="#0891b2" />
          </linearGradient>
          <linearGradient id="logoGradientAccent" x1="0%" y1="100%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#a855f7" />
            <stop offset="100%" stopColor="#22d3ee" />
          </linearGradient>
        </defs>

        {/* Background rounded square */}
        <rect x="2" y="2" width="44" height="44" rx="12" fill="url(#logoGradientMain)" />

        {/* Uptrend chart line */}
        <path
          d="M8 32 L16 26 L22 30 L30 18 L38 12"
          stroke="white"
          strokeWidth="3"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
          opacity="0.95"
        />

        {/* Arrow head for uptrend */}
        <path
          d="M34 10 L40 12 L38 18"
          stroke="white"
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
          opacity="0.95"
        />

        {/* AI Brain chip indicator */}
        <circle cx="30" cy="18" r="5" fill="white" opacity="0.95" />
        <circle cx="30" cy="18" r="2.5" fill="url(#logoGradientAccent)" />

        {/* Data points on chart */}
        <circle cx="16" cy="26" r="2" fill="white" opacity="0.8" />
        <circle cx="22" cy="30" r="2" fill="white" opacity="0.8" />

        {/* Bottom bar chart elements */}
        <rect x="10" y="36" width="4" height="6" rx="1" fill="white" opacity="0.6" />
        <rect x="16" y="34" width="4" height="8" rx="1" fill="white" opacity="0.7" />
        <rect x="22" y="32" width="4" height="10" rx="1" fill="white" opacity="0.8" />
        <rect x="28" y="30" width="4" height="12" rx="1" fill="white" opacity="0.9" />
        <rect x="34" y="28" width="4" height="14" rx="1" fill="white" opacity="1" />
      </svg>

      {/* Text */}
      {showText && (
        <span
          className={`font-black tracking-tight ${textSize}`}
          style={{
            background: `linear-gradient(135deg, ${colors.cyan} 0%, #a855f7 100%)`,
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            backgroundClip: 'text',
          }}
        >
          Bot Core
        </span>
      )}
    </div>
  );
};

// Icon-only variant for compact uses
export const BotCoreIcon = ({ size = 40, className = '' }: { size?: number; className?: string }) => {
  const colors = useThemeColors();

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      style={{
        filter: `drop-shadow(0 2px 8px ${colors.cyan}40)`,
      }}
    >
      <defs>
        <linearGradient id="iconGradientMain" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#22d3ee" />
          <stop offset="100%" stopColor="#0891b2" />
        </linearGradient>
        <linearGradient id="iconGradientAccent" x1="0%" y1="100%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#a855f7" />
          <stop offset="100%" stopColor="#22d3ee" />
        </linearGradient>
      </defs>

      <rect x="2" y="2" width="44" height="44" rx="12" fill="url(#iconGradientMain)" />

      {/* Uptrend chart line */}
      <path
        d="M8 32 L16 26 L22 30 L30 18 L38 12"
        stroke="white"
        strokeWidth="3"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
        opacity="0.95"
      />

      {/* Arrow head */}
      <path
        d="M34 10 L40 12 L38 18"
        stroke="white"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
        opacity="0.95"
      />

      {/* AI indicator */}
      <circle cx="30" cy="18" r="5" fill="white" opacity="0.95" />
      <circle cx="30" cy="18" r="2.5" fill="url(#iconGradientAccent)" />

      {/* Data points */}
      <circle cx="16" cy="26" r="2" fill="white" opacity="0.8" />
      <circle cx="22" cy="30" r="2" fill="white" opacity="0.8" />

      {/* Bottom bars */}
      <rect x="10" y="36" width="4" height="6" rx="1" fill="white" opacity="0.6" />
      <rect x="16" y="34" width="4" height="8" rx="1" fill="white" opacity="0.7" />
      <rect x="22" y="32" width="4" height="10" rx="1" fill="white" opacity="0.8" />
      <rect x="28" y="30" width="4" height="12" rx="1" fill="white" opacity="0.9" />
      <rect x="34" y="28" width="4" height="14" rx="1" fill="white" opacity="1" />
    </svg>
  );
};

export default BotCoreLogo;
