// Re-export BotCoreLogo as Logo for backward compatibility
import { BotCoreLogo, BotCoreIcon } from '@/components/BotCoreLogo';

interface LogoProps {
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showText?: boolean;
  className?: string;
}

export function Logo({ size = 'md', showText = true, className }: LogoProps) {
  return <BotCoreLogo size={size} showText={showText} className={className} />;
}

export function LogoIcon({ size = 'md', className }: Omit<LogoProps, 'showText'>) {
  return <BotCoreIcon size={size === 'sm' ? 24 : size === 'md' ? 32 : size === 'lg' ? 40 : 56} className={className} />;
}
