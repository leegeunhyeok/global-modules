const { resolve } = require('node:path');

const project = resolve(__dirname, 'tsconfig.json');

module.exports = {
  root: true,
  env: {
    node: true,
  },
  plugins: ['prettier'],
  extends: [
    require.resolve('@vercel/style-guide/eslint/node'),
    require.resolve('@vercel/style-guide/eslint/typescript'),
  ],
  parserOptions: {
    project,
  },
  settings: {
    'import/resolver': {
      typescript: {
        project,
      },
    },
  },
  overrides: [
    {
      files: ['*.js', '*.ts', '*.mts'],
      rules: {
        semi: ['error', 'always'],
        quotes: ['error', 'single'],
        eqeqeq: 'off',
        '@typescript-eslint/no-shadow': 'off',
        '@typescript-eslint/no-confusing-void-expression': 'off',
        '@typescript-eslint/prefer-promise-reject-errors': 'off',
        '@typescript-eslint/no-unnecessary-condition': 'off',
        '@typescript-eslint/no-unsafe-assignment': 'off',
        '@typescript-eslint/unbound-method': 'off',
      },
    },
    {
      files: ['vitest.config.mts'],
      rules: {
        'import/no-default-export': 'off',
      },
    },
    {
      files: ['**/swc-plugin/index.*'],
      rules: {
        'import/no-default-export': 'off',
      },
    },
  ],
};
