import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';

export default [
  js.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.es2021,
        __APP_VERSION__: 'readonly',
      },
      ecmaVersion: 2022,
      sourceType: 'module',
    },
    rules: {
      'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: '@typescript-eslint/parser',
      },
    },
  },
  {
    ignores: [
      'dist/',
      'build/',
      '.svelte-kit/',
      'node_modules/',
      'src-tauri/',
      '*.config.js',
    ],
  },
];
