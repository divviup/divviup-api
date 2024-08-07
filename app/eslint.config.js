// @ts-check

import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import prettierConfig from "eslint-config-prettier";
import { fixupPluginRules } from "@eslint/compat";
import eslintPluginReact from "eslint-plugin-react";
import eslintPluginReactHooks from "eslint-plugin-react-hooks";
import globals from "globals";

export default tseslint.config(
  {
    // Global ignores
    ignores: ["build"],
  },
  {
    extends: [
      eslint.configs.recommended,
      ...tseslint.configs.recommended,
      prettierConfig,
    ],
    settings: {
      react: {
        version: "detect",
      },
    },
    languageOptions: {
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: {
          jsx: true,
        },
        projectService: {
          allowDefaultProject: ["eslint.config.js"],
        },
      },
      globals: {
        ...globals.browser,
      },
    },
    plugins: {
      react: eslintPluginReact,
      // @ts-expect-error: types of eslint-plugin-react-hooks do not match flat config types
      "react-hooks": fixupPluginRules(eslintPluginReactHooks),
    },
    // @ts-expect-error: types of eslint-plugin-react-hooks do not match flat config types
    rules: {
      // Recommended rules from plugins that don't yet support flat config:
      ...eslintPluginReact.configs.recommended.rules,
      ...eslintPluginReactHooks.configs.recommended.rules,

      // Rules from eslint-plugin-react's jsx-runtime config:
      "react/react-in-jsx-scope": 0,
      "react/jsx-uses-react": 0,

      // Custom rules configuration:
      "react/no-unescaped-entities": "off",
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
      "no-empty": [
        "error",
        {
          allowEmptyCatch: true,
        },
      ],
      "no-console": "warn",
    },
  },
);
