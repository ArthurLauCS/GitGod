import { svelte } from '@sveltejs/vite-plugin-svelte'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

const shim = fileURLToPath(new URL('./webview/tauri.ts', import.meta.url))

// 提交图面板：与桌面版同一份界面，只把 Tauri 的调用换成发给扩展主机的消息
export default defineConfig({
  plugins: [svelte()],
  base: './',
  resolve: { alias: { '@tauri-apps/api/core': shim, '@tauri-apps/plugin-dialog': shim } },
  build: {
    outDir: 'extension/media',
    emptyOutDir: true,
    cssCodeSplit: false,
    // 文件名不带哈希，扩展主机按固定名字引用
    rollupOptions: {
      input: { index: 'index.html', sidebar: 'extension/webview/sidebar.ts' },
      output: { entryFileNames: '[name].js', chunkFileNames: '[name].js', assetFileNames: '[name][extname]' },
    },
  },
})
