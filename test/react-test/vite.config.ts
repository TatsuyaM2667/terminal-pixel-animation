import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ['terminal-pixel-animation'],
  },
  server: {
    fs: {
      allow: [
        path.resolve(__dirname, '../..'),
      ],
    },
  },
})
