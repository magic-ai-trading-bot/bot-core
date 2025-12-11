import React, { createContext, useContext, useCallback, useMemo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  SUPPORTED_LANGUAGES,
  SupportedLanguage,
  DEFAULT_LANGUAGE,
  changeLanguage as i18nChangeLanguage
} from '@/i18n/config';

interface LanguageContextType {
  language: SupportedLanguage;
  setLanguage: (lang: SupportedLanguage) => void;
  languages: typeof SUPPORTED_LANGUAGES;
  isRTL: boolean;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation();
  const [language, setLanguageState] = useState<SupportedLanguage>(
    (i18n.language as SupportedLanguage) || DEFAULT_LANGUAGE
  );

  // Sync with i18n changes
  useEffect(() => {
    const handleLanguageChanged = (lng: string) => {
      if (lng in SUPPORTED_LANGUAGES) {
        setLanguageState(lng as SupportedLanguage);
      }
    };

    i18n.on('languageChanged', handleLanguageChanged);
    return () => {
      i18n.off('languageChanged', handleLanguageChanged);
    };
  }, [i18n]);

  const setLanguage = useCallback(async (lang: SupportedLanguage) => {
    await i18nChangeLanguage(lang);
    setLanguageState(lang);
  }, []);

  // RTL support (Arabic, Hebrew, etc. - not currently supported but prepared)
  const isRTL = useMemo(() => {
    const rtlLanguages = ['ar', 'he', 'fa', 'ur'];
    return rtlLanguages.includes(language);
  }, [language]);

  // Update document direction for RTL support
  useEffect(() => {
    document.documentElement.dir = isRTL ? 'rtl' : 'ltr';
    document.documentElement.lang = language;
  }, [isRTL, language]);

  const value = useMemo(() => ({
    language,
    setLanguage,
    languages: SUPPORTED_LANGUAGES,
    isRTL,
  }), [language, setLanguage, isRTL]);

  return (
    <LanguageContext.Provider value={value}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage(): LanguageContextType {
  const context = useContext(LanguageContext);
  if (context === undefined) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
}

// Re-export for convenience
export { SUPPORTED_LANGUAGES, type SupportedLanguage };
