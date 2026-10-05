import { builtinModules } from 'node:module'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

// 扩展主机：打成一个 CommonJS 文件，vscode 和 Node 内置模块留给运行时
export default defineConfig({
  build: {
    target: 'node20',
    outDir: 'extension/out',
    emptyOutDir: true,
    minify: false,
    lib: { entry: fileURLToPath(new URL('./src/extension.ts', import.meta.url)), formats: ['cjs'], fileName: () => 'extension.cjs' },
    rollupOptions: { external: ['vscode', ...builtinModules, /^node:/] },
  },
})
