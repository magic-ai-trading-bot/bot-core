import { PremiumButton } from "@/styles/luxury-design-system";
import { Badge } from "@/components/ui/badge";
import { Link, useNavigate } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { MobileNav } from "./MobileNav";
import { WifiOff, LayoutDashboard, TrendingUp, BookOpen, Settings as SettingsIcon, LogOut, User, Wifi, Brain } from "lucide-react";
import { Logo } from "@/components/ui/Logo";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export function DashboardHeader() {
  const { logout, user } = useAuth();
  const navigate = useNavigate();
  const isOnline = useOnlineStatus();

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  return (
    <>
      {!isOnline && (
        <div className="bg-warning/10 border-b border-warning/20 px-4 py-2">
          <div className="flex items-center gap-2 text-warning text-sm">
            <WifiOff className="h-4 w-4" />
            <span>You are offline. Some features may be unavailable.</span>
          </div>
        </div>
      )}
      <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between px-4 lg:px-6 py-3 lg:py-4 border-b border-border gap-4 relative">
        {/* Left: Logo */}
        <div className="flex items-center gap-3 lg:flex-shrink-0">
          {/* Mobile Hamburger Menu - Only visible on mobile/tablet */}
          <MobileNav />
          <Link
            to="/"
            className="group flex items-center"
            aria-label="Go to Home"
          >
            <Logo size="md" />
          </Link>
        </div>

      {/* Center: Navigation Menu - Absolutely centered on desktop */}
      <div className="hidden lg:flex items-center gap-2 lg:absolute lg:left-1/2 lg:-translate-x-1/2">
        <Link to="/dashboard">
          <PremiumButton
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <LayoutDashboard className="h-4 w-4" />
            Dashboard
          </PremiumButton>
        </Link>
        <Link to="/trading-paper">
          <PremiumButton
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <TrendingUp className="h-4 w-4" />
            Trading Paper
          </PremiumButton>
        </Link>
        <Link to="/trade-analyses">
          <PremiumButton
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <Brain className="h-4 w-4" />
            AI Analyses
          </PremiumButton>
        </Link>
        <Link to="/settings">
          <PremiumButton
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <SettingsIcon className="h-4 w-4" />
            Settings
          </PremiumButton>
        </Link>
        <Link to="/how-it-works">
          <PremiumButton
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <BookOpen className="h-4 w-4" />
            How It Works
          </PremiumButton>
        </Link>
      </div>

      <TooltipProvider>
        <div className="flex flex-row items-center gap-2 lg:gap-3 ml-auto lg:ml-0">
          {/* Bot Status Badge */}
          <Badge
            variant="outline"
            className="bg-profit/10 text-profit border-profit/20 text-xs w-fit"
          >
            <div className="w-2 h-2 bg-profit rounded-full mr-1.5 animate-pulse"></div>
            <span className="hidden sm:inline">Active</span>
          </Badge>

          {/* Connection Status */}
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-muted/50 cursor-default">
                <Wifi className="h-3.5 w-3.5 text-profit" />
                <span className="text-xs font-medium hidden md:inline">Binance</span>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>Connected to Binance Futures</p>
            </TooltipContent>
          </Tooltip>

          {/* User Info */}
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-muted/50 cursor-default">
                <User className="h-3.5 w-3.5 text-muted-foreground" />
                <span className="text-xs font-medium truncate max-w-20 hidden md:inline">
                  {user?.full_name?.split(' ')[0] || user?.email?.split('@')[0] || 'User'}
                </span>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p>{user?.full_name || user?.email}</p>
            </TooltipContent>
          </Tooltip>

          {/* Logout Button */}
          <Tooltip>
            <TooltipTrigger asChild>
              <PremiumButton
                variant="ghost"
                size="sm"
                onClick={handleLogout}
                className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground"
              >
                <LogOut className="h-4 w-4" />
              </PremiumButton>
            </TooltipTrigger>
            <TooltipContent>
              <p>Sign Out</p>
            </TooltipContent>
          </Tooltip>
        </div>
      </TooltipProvider>
    </div>
    </>
  );
}
