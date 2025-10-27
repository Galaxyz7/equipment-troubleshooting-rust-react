module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react-hooks/recommended',
  ],
  ignorePatterns: ['dist', '.eslintrc.cjs', 'src/types/*.ts'],
  parser: '@typescript-eslint/parser',
  plugins: ['react-refresh'],
  rules: {
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true },
    ],
    '@typescript-eslint/no-explicit-any': 'off',
    // Enforce using the logger utility instead of console.*
    'no-console': 'error',
  },
  overrides: [
    {
      // Allow console usage in the logger utility itself
      files: ['src/lib/logger.ts'],
      rules: {
        'no-console': 'off',
      },
    },
  ],
}
