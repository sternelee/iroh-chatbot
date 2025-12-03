import js from '@eslint/js'
import typescript from '@typescript-eslint/eslint-plugin'
import typescriptParser from '@typescript-eslint/parser'
import vue from 'eslint-plugin-vue'
import prettier from 'eslint-plugin-prettier'
import prettierConfig from 'eslint-config-prettier'
import vueParser from 'vue-eslint-parser'

export default [
  // 全局忽略配置
  {
    ignores: [
      'node_modules/**',
      'dist/**',
      'build/**',
      'coverage/**',
      '*.d.ts',
      'vite.config.js',
      'tailwind.config.js',
      'eslint.config.js',
    ],
  },

  // JavaScript 基础配置
  js.configs.recommended,

  // TypeScript 配置
  {
    files: ['**/*.{ts,tsx,vue}'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        extraFileExtensions: ['.vue'],
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
    },
    rules: {
      // TypeScript 规则
      ...typescript.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-non-null-assertion': 'warn',
      '@typescript-eslint/prefer-const': 'off',
      '@typescript-eslint/no-inferrable-types': 'off',
    },
  },

  // Vue 配置
  {
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: typescriptParser,
        ecmaVersion: 'latest',
        sourceType: 'module',
        extraFileExtensions: ['.vue'],
      },
      globals: {
        defineProps: 'readonly',
        defineEmits: 'readonly',
        defineExpose: 'readonly',
        withDefaults: 'readonly',
        document: 'readonly',
        window: 'readonly',
        navigator: 'readonly',
        KeyboardEvent: 'readonly',
        MouseEvent: 'readonly',
        Event: 'readonly',
        HTMLElement: 'readonly',
        HTMLFormElement: 'readonly',
        HTMLInputElement: 'readonly',
        File: 'readonly',
        FileList: 'readonly',
        FileReader: 'readonly',
        DragEvent: 'readonly',
        ClipboardEvent: 'readonly',
        EventTarget: 'readonly',
        URL: 'readonly',
        fetch: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        console: 'readonly',
        alert: 'readonly',
        prompt: 'readonly',
        confirm: 'readonly',
      },
    },
    plugins: {
      vue: vue,
    },
    rules: {
      // Vue 基础规则
      'vue/multi-word-component-names': 'off',
      'vue/html-self-closing': 'error',
      'vue/no-unused-components': 'warn',
      'vue/require-explicit-emits': 'error',
      'vue/component-name-in-template-casing': ['error', 'PascalCase'],
      'vue/component-definition-name-casing': ['error', 'PascalCase'],
      'vue/custom-event-name-casing': ['error', 'camelCase'],
    },
  },

  // JavaScript/TypeScript 文件的额外规则
  {
    files: ['**/*.{js,ts,tsx}'],
    languageOptions: {
      globals: {
        console: 'readonly',
        process: 'readonly',
        __dirname: 'readonly',
        document: 'readonly',
        window: 'readonly',
        navigator: 'readonly',
        HTMLElement: 'readonly',
        HTMLFormElement: 'readonly',
        HTMLInputElement: 'readonly',
        File: 'readonly',
        FileList: 'readonly',
        FileReader: 'readonly',
        DragEvent: 'readonly',
        ClipboardEvent: 'readonly',
        EventTarget: 'readonly',
        URL: 'readonly',
        fetch: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        alert: 'readonly',
        prompt: 'readonly',
        confirm: 'readonly',
      },
    },
    rules: {
      // 通用规则
      'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
      'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
      'no-unused-vars': 'off', // 使用 TypeScript 规则
      'prefer-const': 'error',
      'no-var': 'error',
      'object-shorthand': 'error',
      'prefer-template': 'error',
      'template-curly-spacing': 'error',
      'arrow-spacing': 'error',
      'comma-dangle': ['error', 'es5'],
      'quotes': ['error', 'single', { avoidEscape: true }],
      'semi': ['error', 'never'],
      'indent': 'off', // 使用 Prettier
      'max-len': ['warn', { code: 100, ignoreUrls: true }],
      'eol-last': 'error',
      'no-trailing-spaces': 'error',
    },
  },

  // 特定文件配置
  {
    files: ['src/**/*.{ts,tsx,vue}'],
    rules: {
      // 源码更严格的规则
      'no-console': 'warn',
      'no-debugger': 'error',
    },
  },

  // 测试文件配置（如果有的话）
  {
    files: ['**/*.{test,spec}.{js,ts,tsx,vue}'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      'no-console': 'off',
    },
  },

  // Prettier 配置（必须放在最后）
  {
    plugins: {
      prettier: prettier,
    },
    rules: {
      ...prettierConfig.rules,
      'prettier/prettier': 'error',
    },
  },
]