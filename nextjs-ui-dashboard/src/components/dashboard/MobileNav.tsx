import { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { PremiumButton } from "@/styles/luxury-design-system";
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet";
import { Menu, LayoutDashboard, TrendingUp, Settings, LogOut, BookOpen, Brain } from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { useNavigate } from "react-router-dom";
import { Logo } from "@/components/ui/Logo";

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
      icon: LayoutDashboard,
    },
    {
      label: "Paper Trading",
      href: "/trading/paper",
      icon: TrendingUp,
    },
    {
      label: "AI Analyses",
      href: "/trade-analyses",
      icon: Brain,
    },
    {
      label: "Settings",
      href: "/settings",
      icon: Settings,
    },
    {
      label: "Hướng Dẫn",
      href: "/how-it-works",
      icon: BookOpen,
    },
  ];

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <PremiumButton
          variant="ghost"
          size="sm"
          className="lg:hidden focus-custom"
          aria-label="Open navigation menu"
        >
          <Menu className="h-5 w-5" />
        </PremiumButton>
      </SheetTrigger>
      <SheetContent side="left" className="w-[280px] sm:w-[320px]">
        <SheetHeader>
          <SheetTitle className="text-left">
            <Logo size="sm" />
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
            <PremiumButton
              variant="secondary"
              className="w-full justify-start gap-3 focus-danger"
              onClick={() => {
                handleLogout();
                setOpen(false);
              }}
            >
              <LogOut className="h-5 w-5" aria-hidden="true" />
              <span>Đăng xuất</span>
            </PremiumButton>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
