import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig(async () => ({
  plugins: [sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,

  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
    // Tauri expects localhost
    host: "localhost",
  },

  // 环境变量前缀
  envPrefix: ["VITE_", "TAURI_"],

  // 构建选项
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // 代码分割
    rollupOptions: {
      output: {
        manualChunks: {
          // 将第三方库分离到单独的chunk
          vendor: ["svelte"],
        },
      },
    },
    // 资源大小警告限制
    chunkSizeWarningLimit: 1000,
  },

  // 优化选项
  optimizeDeps: {
    // 预构建依赖
    include: ["@tauri-apps/api/core", "@tauri-apps/api/event", "@tauri-apps/api/webviewWindow"],
    // 排除服务端依赖
    exclude: ["@tauri-apps/cli"],
  },

  // 测试配置
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    environment: "jsdom",
  },

  // 开发工具
  define: {
    // 全局常量定义
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version || "0.1.0"),
    __TAURI_DEBUG__: JSON.stringify(!!process.env.TAURI_DEBUG),
  },

  // 预览服务器配置
  preview: {
    port: 4173,
    strictPort: true,
    host: "localhost",
  },
}));
