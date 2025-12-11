// @spec:FR-I18N-001 - Language Selector Component
// @ref:plans/20251209-2030-ui-theme-i18n/phase-03-i18n-restructure.md

import { useState, useRef, useEffect } from "react";
import { Globe, Check, ChevronDown } from "lucide-react";
import { useLanguage, SUPPORTED_LANGUAGES, SupportedLanguage } from "@/contexts/LanguageContext";
import { useThemeColors } from "@/hooks/useThemeColors";
import { motion, AnimatePresence } from "framer-motion";

export function LanguageSelector() {
  const { language, setLanguage } = useLanguage();
  const colors = useThemeColors();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const currentLanguage = SUPPORTED_LANGUAGES[language];
  const languageEntries = Object.entries(SUPPORTED_LANGUAGES) as [SupportedLanguage, typeof SUPPORTED_LANGUAGES[SupportedLanguage]][];

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

  const handleSelect = (code: SupportedLanguage) => {
    setLanguage(code);
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Trigger Button - 44px touch target (WCAG 2.5.5) */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 h-11 min-w-[44px] rounded-xl transition-all duration-200"
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
      >
        <Globe className="w-4 h-4" />
        <span className="hidden sm:inline">{currentLanguage.flag}</span>
        <span className="text-sm font-medium">{language.toUpperCase()}</span>
        <ChevronDown
          className="w-3 h-3 transition-transform duration-200"
          style={{ transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)' }}
        />
      </button>

      {/* Dropdown Menu */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: -8, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -8, scale: 0.95 }}
            transition={{ duration: 0.15 }}
            className="absolute right-0 mt-2 min-w-[180px] rounded-xl shadow-xl z-50 overflow-hidden"
            style={{
              backgroundColor: colors.bgPrimary,
              border: `1px solid ${colors.borderSubtle}`,
              boxShadow: `0 10px 40px ${colors.shadowColor}`,
            }}
          >
            <div className="py-1.5">
              {languageEntries.map(([code, lang]) => {
                const isActive = language === code;
                return (
                  <button
                    key={code}
                    onClick={() => handleSelect(code)}
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
                    <span className="text-base">{lang.flag}</span>
                    <span className="flex-1 text-left">{lang.nativeName}</span>
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
