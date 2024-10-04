import { defineConfig } from 'vite';

export default defineConfig({
  test: {
    globalSetup: ['./tests/global-setup.mts'],
  },
});
