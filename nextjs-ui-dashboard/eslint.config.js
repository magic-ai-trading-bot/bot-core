import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import unusedImports from "eslint-plugin-unused-imports";

export default tseslint.config(
  { ignores: ["dist", "node_modules", "coverage", "*.config.js", "*.config.ts", "vitest.globalSetup.ts", "vitest-environment-*.ts", "e2e/**", "**/__tests__/**", "**/*.test.*", "**/*.spec.*", "**/test/**"] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
      "unused-imports": unusedImports,
    },
    rules: {
      // React hooks rules
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "off",

      // React refresh
      "react-refresh/only-export-components": "off",

      // Unused imports - auto-remove
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "unused-imports/no-unused-imports": "error",
      "unused-imports/no-unused-vars": "off", // Turn off - chỉ xóa imports

      // TypeScript - tắt hết warnings
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "@typescript-eslint/no-empty-function": "off",
      "@typescript-eslint/no-non-null-assertion": "off",

      // Console logs - off
      "no-console": "off",
    },
  },
  // Allow component exports with utilities
  {
    files: [
      "**/components/ui/**/*.{ts,tsx}",
      "**/contexts/**/*.{ts,tsx}",
      "**/test/utils.tsx",
      "**/styles/**/*.{ts,tsx}",
    ],
    rules: {
      "react-refresh/only-export-components": "off",
    },
  }
);
