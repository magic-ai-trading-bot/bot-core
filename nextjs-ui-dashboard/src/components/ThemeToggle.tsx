// @spec:FR-THEME-001 - Theme Toggle Component
// @ref:plans/20251209-2030-ui-theme-i18n/phase-05-theme-toggle-ui.md

import { useState, useRef, useEffect } from "react";
import { Moon, Sun, Monitor, Check } from "lucide-react";
import { useTheme, type Theme } from "@/contexts/ThemeContext";
import { useThemeColors } from "@/hooks/useThemeColors";
import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "framer-motion";

const themeOptions: { value: Theme; icon: typeof Sun; labelKey: string }[] = [
  { value: "light", icon: Sun, labelKey: "theme.light" },
  { value: "dark", icon: Moon, labelKey: "theme.dark" },
  { value: "system", icon: Monitor, labelKey: "theme.system" },
];

export function ThemeToggle() {
  const { theme, resolvedTheme, setTheme } = useTheme();
  const { t } = useTranslation();
  const colors = useThemeColors();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const CurrentIcon = resolvedTheme === "dark" ? Moon : Sun;

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleSelect = (value: Theme) => {
    setTheme(value);
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Trigger Button - 44px touch target (WCAG 2.5.5) */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center justify-center h-11 w-11 rounded-xl transition-all duration-200"
        style={{
          backgroundColor: isOpen ? colors.bgSecondary : 'transparent',
          color: colors.textSecondary,
        }}
        onMouseEnter={(e) => {
          if (!isOpen) e.currentTarget.style.backgroundColor = colors.bgSecondary;
        }}
        onMouseLeave={(e) => {
          if (!isOpen) e.currentTarget.style.backgroundColor = 'transparent';
        }}
        aria-label={t("theme.toggle", "Toggle theme")}
      >
        <CurrentIcon className="h-4 w-4" />
      </button>

      {/* Dropdown Menu */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: -8, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -8, scale: 0.95 }}
            transition={{ duration: 0.15 }}
            className="absolute right-0 mt-2 min-w-[160px] rounded-xl shadow-xl z-50 overflow-hidden"
            style={{
              backgroundColor: colors.bgPrimary,
              border: `1px solid ${colors.borderSubtle}`,
              boxShadow: `0 10px 40px ${colors.shadowColor}`,
            }}
          >
            <div className="py-1.5">
              {themeOptions.map((option) => {
                const Icon = option.icon;
                const isActive = theme === option.value;
                return (
                  <button
                    key={option.value}
                    onClick={() => handleSelect(option.value)}
                    className="w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-all duration-150"
                    style={{
                      backgroundColor: isActive ? colors.bgSecondary : 'transparent',
                      color: isActive ? colors.textPrimary : colors.textSecondary,
                    }}
                    onMouseEnter={(e) => {
                      if (!isActive) {
                        e.currentTarget.style.backgroundColor = colors.bgSecondary;
                        e.currentTarget.style.color = colors.textPrimary;
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (!isActive) {
                        e.currentTarget.style.backgroundColor = 'transparent';
                        e.currentTarget.style.color = colors.textSecondary;
                      }
                    }}
                  >
                    <Icon className="h-4 w-4" style={{ color: isActive ? colors.cyan : undefined }} />
                    <span className="flex-1 text-left">{t(option.labelKey, option.value)}</span>
                    {isActive && (
                      <Check className="h-4 w-4" style={{ color: colors.cyan }} />
                    )}
                  </button>
                );
              })}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
