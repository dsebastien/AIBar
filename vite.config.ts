import path, { resolve } from 'path'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
    plugins: [react(), tailwindcss()],
    root: 'src',
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                  protocol: 'ws',
                  host,
                  port: 1421
              }
            : undefined,
        watch: {
            ignored: ['**/src-tauri/**']
        }
    },
    envPrefix: ['VITE_', 'TAURI_ENV_*'],
    build: {
        outDir: '../dist',
        emptyOutDir: true,
        rollupOptions: {
            input: {
                main: resolve(__dirname, 'src/index.html')
            }
        },
        target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome107' : 'safari16',
        minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
        sourcemap: !!process.env.TAURI_ENV_DEBUG
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src')
        }
    },
    publicDir: '../public'
})
