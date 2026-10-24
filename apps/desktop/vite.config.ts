import { defineConfig } from 'vite';

const DEV_PORT = 1420;

export default defineConfig({
  clearScreen: false,
  server: { host: '127.0.0.1', port: DEV_PORT, strictPort: true },
  build: { target: ['es2022', 'safari15'] },
});
