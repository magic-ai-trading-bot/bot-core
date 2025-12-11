// i18n configuration with namespace-based translations
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import English translations
import enCommon from './locales/en/common.json';
import enLanding from './locales/en/landing.json';
import enTrading from './locales/en/trading.json';
import enDashboard from './locales/en/dashboard.json';
import enErrors from './locales/en/errors.json';
import enAuth from './locales/en/auth.json';
import enSettings from './locales/en/settings.json';
import enPages from './locales/en/pages.json';

// Import Vietnamese translations
import viCommon from './locales/vi/common.json';
import viLanding from './locales/vi/landing.json';
import viTrading from './locales/vi/trading.json';
import viDashboard from './locales/vi/dashboard.json';
import viErrors from './locales/vi/errors.json';
import viAuth from './locales/vi/auth.json';
import viSettings from './locales/vi/settings.json';
import viPages from './locales/vi/pages.json';

// Import French translations
import frCommon from './locales/fr/common.json';
import frLanding from './locales/fr/landing.json';
import frTrading from './locales/fr/trading.json';
import frDashboard from './locales/fr/dashboard.json';
import frErrors from './locales/fr/errors.json';
import frAuth from './locales/fr/auth.json';
import frSettings from './locales/fr/settings.json';
import frPages from './locales/fr/pages.json';

// Import Chinese translations
import zhCommon from './locales/zh/common.json';
import zhLanding from './locales/zh/landing.json';
import zhTrading from './locales/zh/trading.json';
import zhDashboard from './locales/zh/dashboard.json';
import zhErrors from './locales/zh/errors.json';
import zhAuth from './locales/zh/auth.json';
import zhSettings from './locales/zh/settings.json';
import zhPages from './locales/zh/pages.json';

// Import Japanese translations
import jaCommon from './locales/ja/common.json';
import jaLanding from './locales/ja/landing.json';
import jaTrading from './locales/ja/trading.json';
import jaDashboard from './locales/ja/dashboard.json';
import jaErrors from './locales/ja/errors.json';
import jaAuth from './locales/ja/auth.json';
import jaSettings from './locales/ja/settings.json';
import jaPages from './locales/ja/pages.json';

export const SUPPORTED_LANGUAGES = {
  en: { name: 'English', nativeName: 'English', flag: '🇺🇸' },
  vi: { name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
  fr: { name: 'French', nativeName: 'Français', flag: '🇫🇷' },
  zh: { name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
  ja: { name: 'Japanese', nativeName: '日本語', flag: '🇯🇵' },
} as const;

export type SupportedLanguage = keyof typeof SUPPORTED_LANGUAGES;

export const NAMESPACES = ['common', 'landing', 'trading', 'dashboard', 'errors', 'auth', 'settings', 'pages'] as const;
export type Namespace = typeof NAMESPACES[number];

export const DEFAULT_NAMESPACE: Namespace = 'common';
export const DEFAULT_LANGUAGE: SupportedLanguage = 'en';

const resources = {
  en: {
    common: enCommon,
    landing: enLanding,
    trading: enTrading,
    dashboard: enDashboard,
    errors: enErrors,
    auth: enAuth,
    settings: enSettings,
    pages: enPages,
  },
  vi: {
    common: viCommon,
    landing: viLanding,
    trading: viTrading,
    dashboard: viDashboard,
    errors: viErrors,
    auth: viAuth,
    settings: viSettings,
    pages: viPages,
  },
  fr: {
    common: frCommon,
    landing: frLanding,
    trading: frTrading,
    dashboard: frDashboard,
    errors: frErrors,
    auth: frAuth,
    settings: frSettings,
    pages: frPages,
  },
  zh: {
    common: zhCommon,
    landing: zhLanding,
    trading: zhTrading,
    dashboard: zhDashboard,
    errors: zhErrors,
    auth: zhAuth,
    settings: zhSettings,
    pages: zhPages,
  },
  ja: {
    common: jaCommon,
    landing: jaLanding,
    trading: jaTrading,
    dashboard: jaDashboard,
    errors: jaErrors,
    auth: jaAuth,
    settings: jaSettings,
    pages: jaPages,
  },
};

// Get initial language from localStorage or browser
const getInitialLanguage = (): SupportedLanguage => {
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined' && localStorage?.getItem) {
    try {
      const stored = localStorage.getItem('language');
      if (stored && stored in SUPPORTED_LANGUAGES) {
        return stored as SupportedLanguage;
      }
      // Try browser language
      if (navigator?.language) {
        const browserLang = navigator.language.split('-')[0];
        if (browserLang in SUPPORTED_LANGUAGES) {
          return browserLang as SupportedLanguage;
        }
      }
    } catch {
      // localStorage might throw in some environments (e.g., incognito mode)
    }
  }
  return DEFAULT_LANGUAGE;
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    lng: getInitialLanguage(),
    fallbackLng: DEFAULT_LANGUAGE,
    defaultNS: DEFAULT_NAMESPACE,
    ns: NAMESPACES,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      lookupLocalStorage: 'language',
      caches: ['localStorage'],
    },
    react: {
      useSuspense: false,
    },
  });

// Helper to change language and persist
export const changeLanguage = async (lang: SupportedLanguage): Promise<void> => {
  await i18n.changeLanguage(lang);
  if (typeof window !== 'undefined' && typeof localStorage !== 'undefined' && localStorage?.setItem) {
    try {
      localStorage.setItem('language', lang);
    } catch {
      // localStorage might throw in some environments
    }
  }
};

export default i18n;
