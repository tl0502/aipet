import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import path from 'node:path'

// 显式锁定绝对路径,绕过 Vite 在含中文目录(如 D:\Project\ai桌宠)下相对路径解析问题。
const projectRoot = fileURLToPath(new URL('.', import.meta.url))
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue()],
  root: projectRoot,
  publicDir: path.resolve(projectRoot, 'public'),
  resolve: {
    alias: {
      '@': path.resolve(projectRoot, 'src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1431,
        }
      : undefined,
    fs: {
      allow: [projectRoot],
    },
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    outDir: path.resolve(projectRoot, 'dist'),
    target: ['es2021', 'chrome105'],
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
})
