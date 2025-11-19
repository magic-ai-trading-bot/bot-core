import { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Menu, Home, TrendingUp, Settings, LogOut } from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { useNavigate } from "react-router-dom";

export function MobileNav() {
  const [open, setOpen] = useState(false);
  const location = useLocation();
  const { logout, user } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  const navItems = [
    {
      label: "Dashboard",
      href: "/dashboard",
      icon: Home,
    },
    {
      label: "Trading Paper",
      href: "/trading-paper",
      icon: TrendingUp,
    },
    {
      label: "Settings",
      href: "/settings",
      icon: Settings,
    },
  ];

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="lg:hidden focus-custom"
          aria-label="Open navigation menu"
        >
          <Menu className="h-5 w-5" />
        </Button>
      </SheetTrigger>
      <SheetContent side="left" className="w-[280px] sm:w-[320px]">
        <SheetHeader>
          <SheetTitle className="text-left flex items-center gap-2">
            <div
              className="w-8 h-8 bg-gradient-to-br from-primary to-accent rounded-lg flex items-center justify-center"
              role="img"
              aria-label="Bot Core Logo"
            >
              <span className="text-primary-foreground font-bold text-sm" aria-hidden="true">
                BT
              </span>
            </div>
            <span>Crypto Trading Bot</span>
          </SheetTitle>
        </SheetHeader>

        <div className="mt-8 space-y-4">
          {/* User Info */}
          <div className="p-4 rounded-lg bg-secondary/50 border border-border">
            <p className="text-xs text-muted-foreground mb-1">Logged in as</p>
            <p className="font-semibold text-sm truncate">
              {user?.full_name || user?.email}
            </p>
          </div>

          {/* Navigation Links */}
          <nav className="space-y-2" aria-label="Mobile navigation">
            {navItems.map((item) => {
              const isActive = location.pathname === item.href;
              const Icon = item.icon;

              return (
                <Link
                  key={item.href}
                  to={item.href}
                  onClick={() => setOpen(false)}
                  className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors focus-custom ${
                    isActive
                      ? "bg-profit/10 text-profit border border-profit/20"
                      : "hover:bg-secondary/50 text-foreground"
                  }`}
                  aria-current={isActive ? "page" : undefined}
                >
                  <Icon className="h-5 w-5" aria-hidden="true" />
                  <span className="font-medium">{item.label}</span>
                </Link>
              );
            })}
          </nav>

          {/* Logout Button */}
          <div className="pt-4 border-t border-border">
            <Button
              variant="outline"
              className="w-full justify-start gap-3 focus-danger"
              onClick={() => {
                handleLogout();
                setOpen(false);
              }}
            >
              <LogOut className="h-5 w-5" aria-hidden="true" />
              <span>Đăng xuất</span>
            </Button>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
