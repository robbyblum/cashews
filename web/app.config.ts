import { defineConfig } from '@tanstack/react-start/config'
import tsConfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
  vite: {
    plugins: [
      tsConfigPaths({
        projects: ['./tsconfig.json'],
      }),
    ],
  },
  server: {
    preset: "node-server",
    routeRules: {
      "/api/**": {
        proxy: {
          to: "http://localhost:3001/**"
        }
      }
    }
  }
})