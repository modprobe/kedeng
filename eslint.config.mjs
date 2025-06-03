// @ts-check

import tseslint from "typescript-eslint";
import prettierRecommended from "eslint-plugin-prettier/recommended";
import jest from "eslint-plugin-jest";
import importPlugin from "eslint-plugin-import";

/** @type {import("typescript-eslint").ConfigArray} */
const config = tseslint.config(
  {
    ignores: ["jest.config.*", "tsconfig.json", "eslint.config.*", "**/dist/**"],
  },
  ...tseslint.configs.recommendedTypeChecked,
  { ignores: ['dist'] },
  {
    files: ["**/*.test.ts"],
    ...jest.configs["flat/recommended"],
    rules: {
      ...jest.configs["flat/recommended"].rules,
      "@typescript-eslint/unbound-method": "off",
      "jest/unbound-method": "error",
      "@typescript-eslint/no-floating-promises": "off",
    },
  },
  {
    plugins: {
      import: importPlugin,
    },
  },
  {
    languageOptions: {
      globals: {
        console: "readonly",
        process: "readonly",
        __dirname: "readonly",
      },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    rules: {
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/interface-name-prefix": "off",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "import/order": [
        "error",
        {
          "newlines-between": "always",
        },
      ],
      "import/no-duplicates": "error",
      "@typescript-eslint/no-unused-vars": ["error", { destructuredArrayIgnorePattern: "^_" }]
    },
  },
  prettierRecommended,
);

export default config;