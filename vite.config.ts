import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import path from 'node:path'

// 显式锁定绝对路径,绕过 Vite 在含中文目录(如 D:\Project\ai桌宠)下相对路径解析问题。
const projectRoot = fileURLToPath(new URL('.', import.meta.url))
const host = process.env.TAURI_DEV_HOST

export default defineConfig(({ command }) => ({
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
  // production 构建时 strip console.log / debugger,避免 release 二进制污染控制台 + 暴露内部 metric 名
  // (详 progress/code-review-2026-05-03.md L-7)。dev 期保留 console 用于开发调试。
  esbuild: {
    drop: command === 'build' ? ['console', 'debugger'] : [],
  },
  build: {
    outDir: path.resolve(projectRoot, 'dist'),
    target: ['es2021', 'chrome105'],
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}))
