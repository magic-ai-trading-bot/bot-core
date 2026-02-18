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
      <img
        src="/brand/botcore-avatar-512.png"
        alt="BotCore"
        width={iconSize}
        height={iconSize}
        className="rounded-lg object-cover"
        style={{
          filter: `drop-shadow(0 2px 8px ${colors.cyan}40)`,
        }}
      />

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
    <img
      src="/brand/botcore-avatar-512.png"
      alt="BotCore"
      width={size}
      height={size}
      className={`rounded-lg object-cover ${className}`}
      style={{
        filter: `drop-shadow(0 2px 8px ${colors.cyan}40)`,
      }}
    />
  );
};

export default BotCoreLogo;
