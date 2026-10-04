import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  server: {
    proxy: {
      '/events': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/result.mxl': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      '/tree': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      }
    }
  }
})
