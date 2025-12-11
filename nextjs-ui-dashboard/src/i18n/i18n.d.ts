// TypeScript declarations for i18n
import 'i18next';
import type enCommon from './locales/en/common.json';
import type enLanding from './locales/en/landing.json';
import type enTrading from './locales/en/trading.json';
import type enDashboard from './locales/en/dashboard.json';
import type enErrors from './locales/en/errors.json';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: typeof enCommon;
      landing: typeof enLanding;
      trading: typeof enTrading;
      dashboard: typeof enDashboard;
      errors: typeof enErrors;
    };
  }
}
