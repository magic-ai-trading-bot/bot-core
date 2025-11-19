import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist"] },
  {
    extends: [...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      // globals: globals.browser, // Commented out - missing dependency
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-unused-vars": "off",
      // Prevent console.logs in production code
      "no-console":
        process.env.NODE_ENV === "production"
          ? "error"
          : "warn",
      // Disable purity rule for mock/visualization data (Math.random, Date.now in components)
      // These are used for demo charts and don't affect actual business logic
      "react-hooks/purity": "off",
    },
  },
  // Allow 'any' type in test files
  {
    files: ["**/__tests__/**/*.{ts,tsx}", "**/*.test.{ts,tsx}", "**/*.spec.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  // Allow console in logger utility
  {
    files: ["**/utils/logger.ts"],
    rules: {
      "no-console": "off",
    },
  },
  // Allow component exports with utilities in UI components, contexts, and test utils
  {
    files: ["**/components/ui/**/*.{ts,tsx}", "**/contexts/**/*.{ts,tsx}", "**/test/utils.tsx"],
    rules: {
      "react-refresh/only-export-components": "off",
    },
  }
);
