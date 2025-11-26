import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Link, useNavigate } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { MobileNav } from "./MobileNav";
import { WifiOff, LayoutDashboard, TrendingUp, BookOpen, Settings as SettingsIcon } from "lucide-react";

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
            <span>Bạn đang offline. Một số tính năng có thể không khả dụng.</span>
          </div>
        </div>
      )}
      <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between p-4 lg:p-6 border-b border-border gap-4">
        <div className="flex items-center gap-4">
        {/* Mobile Hamburger Menu - Only visible on mobile/tablet */}
        <MobileNav />
        <Link
          to="/dashboard"
          className="flex items-center gap-2 hover:opacity-80 transition-opacity"
          aria-label="Go to Dashboard home"
        >
          <div
            className="w-8 h-8 bg-gradient-to-br from-primary to-accent rounded-lg flex items-center justify-center"
            role="img"
            aria-label="Bot Core Logo"
          >
            <span className="text-primary-foreground font-bold text-sm" aria-hidden="true">
              BT
            </span>
          </div>
          <div>
            <h1 className="text-xl lg:text-2xl font-bold">
              Crypto Trading Bot
            </h1>
            <p className="text-muted-foreground text-xs lg:text-sm">
              AI-Powered Futures Trading
            </p>
          </div>
        </Link>
      </div>

      {/* Navigation Menu - Mobile friendly */}
      <div className="hidden lg:flex items-center gap-2">
        <Link to="/dashboard">
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <LayoutDashboard className="h-4 w-4" />
            Dashboard
          </Button>
        </Link>
        <Link to="/trading-paper">
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <TrendingUp className="h-4 w-4" />
            Trading Paper
          </Button>
        </Link>
        <Link to="/settings">
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <SettingsIcon className="h-4 w-4" />
            Settings
          </Button>
        </Link>
        <Link to="/how-it-works">
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-foreground text-sm flex items-center gap-2"
          >
            <BookOpen className="h-4 w-4" />
            Hướng Dẫn
          </Button>
        </Link>
      </div>

      <div className="flex flex-row items-center gap-2 lg:gap-4 ml-auto lg:ml-0">
        <Badge
          variant="outline"
          className="bg-profit/10 text-profit border-profit/20 text-xs lg:text-sm w-fit"
        >
          <div className="w-2 h-2 bg-profit rounded-full mr-2 animate-pulse"></div>
          Bot Active
        </Badge>

        <div className="text-left lg:text-right">
          <p className="text-xs text-muted-foreground">Connected to</p>
          <p className="font-semibold text-xs lg:text-sm">Binance Futures</p>
        </div>

        <div className="text-left lg:text-right">
          <p className="text-xs text-muted-foreground">Logged in as</p>
          <p className="font-semibold text-xs truncate max-w-32 lg:max-w-none">
            {user?.full_name || user?.email}
          </p>
        </div>

        <Button
          variant="outline"
          size="sm"
          onClick={handleLogout}
          className="text-xs lg:text-sm w-fit"
        >
          Đăng xuất
        </Button>
      </div>
    </div>
    </>
  );
}
