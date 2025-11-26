import { Bot, Zap } from "lucide-react";
import { cn } from "@/lib/utils";

interface LogoProps {
  size?: "sm" | "md" | "lg" | "xl";
  showText?: boolean;
  className?: string;
}

const sizeConfig = {
  sm: {
    container: "w-8 h-8",
    innerContainer: "w-8 h-8",
    icon: "w-4 h-4",
    badge: "w-3 h-3 -top-0.5 -right-0.5",
    badgeIcon: "w-2 h-2",
    title: "text-base",
    subtitle: "text-[9px]",
  },
  md: {
    container: "w-10 h-10",
    innerContainer: "w-10 h-10",
    icon: "w-5 h-5",
    badge: "w-3.5 h-3.5 -top-0.5 -right-0.5",
    badgeIcon: "w-2 h-2",
    title: "text-lg",
    subtitle: "text-[10px]",
  },
  lg: {
    container: "w-11 h-11",
    innerContainer: "w-11 h-11",
    icon: "w-6 h-6",
    badge: "w-4 h-4 -top-1 -right-1",
    badgeIcon: "w-2.5 h-2.5",
    title: "text-lg lg:text-xl",
    subtitle: "text-[10px] lg:text-xs",
  },
  xl: {
    container: "w-14 h-14 md:w-16 md:h-16",
    innerContainer: "w-14 h-14 md:w-16 md:h-16",
    icon: "w-7 h-7 md:w-8 md:h-8",
    badge: "w-5 h-5 -top-1 -right-1",
    badgeIcon: "w-3 h-3",
    title: "text-xl md:text-2xl",
    subtitle: "text-xs",
  },
};

export function Logo({ size = "md", showText = true, className }: LogoProps) {
  const config = sizeConfig[size];

  return (
    <div className={cn("flex items-center gap-3", className)}>
      {/* 3D Logo with Depth Effect */}
      <div className="relative" role="img" aria-label="BotCore Logo">
        {/* Back layer - shadow/depth */}
        <div
          className={cn(
            "absolute inset-0 bg-gradient-to-br from-green-700 to-green-800 rounded-2xl translate-x-1 translate-y-1 opacity-60",
            config.innerContainer
          )}
        />
        {/* Middle layer - glow */}
        <div
          className={cn(
            "absolute inset-0 bg-gradient-to-br from-green-400 to-green-500 rounded-2xl blur-sm opacity-50 group-hover:opacity-70 transition-opacity duration-300",
            config.innerContainer
          )}
        />
        {/* Front layer - main logo */}
        <div
          className={cn(
            "relative bg-gradient-to-br from-green-400 via-green-500 to-green-600 rounded-2xl flex items-center justify-center border border-green-300/30 group-hover:scale-105 group-hover:-translate-y-0.5 transition-all duration-300",
            config.innerContainer
          )}
        >
          {/* Inner highlight */}
          <div className="absolute inset-0.5 bg-gradient-to-br from-white/20 to-transparent rounded-xl" />
          {/* Icon */}
          <Bot
            className={cn("text-white drop-shadow-md relative z-10", config.icon)}
            aria-hidden="true"
          />
        </div>
        {/* Floating particle */}
        <div
          className={cn(
            "absolute bg-gradient-to-br from-amber-300 to-orange-400 rounded-full flex items-center justify-center shadow-lg shadow-amber-400/40 border border-amber-200/50",
            config.badge
          )}
        >
          <Zap className={cn("text-amber-900", config.badgeIcon)} />
        </div>
      </div>

      {/* Brand Text */}
      {showText && (
        <div className="flex flex-col">
          <h1
            className={cn(
              "font-extrabold tracking-tight leading-none",
              config.title
            )}
          >
            <span className="text-foreground drop-shadow-sm">Bot</span>
            <span className="bg-gradient-to-r from-green-400 to-green-500 bg-clip-text text-transparent">
              Core
            </span>
          </h1>
          <p
            className={cn(
              "text-muted-foreground/80 font-semibold tracking-widest uppercase",
              config.subtitle
            )}
          >
            AI Trading
          </p>
        </div>
      )}
    </div>
  );
}

export function LogoIcon({ size = "md", className }: Omit<LogoProps, "showText">) {
  return <Logo size={size} showText={false} className={className} />;
}
