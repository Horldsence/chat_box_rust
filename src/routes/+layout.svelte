<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { errorService } from "$lib/services/ErrorService";
  import { themeStore, currentTheme } from "$lib/stores/themeStore";
  import Notification from "$lib/components/Notification.svelte";
  import "../app.css";

  let mounted = false;

  onMount(() => {
    mounted = true;

    // 初始化错误处理
    errorService.init();

    // 检查系统主题偏好
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (!localStorage.getItem("theme")) {
      themeStore.setMode(prefersDark ? "dark" : "light");
    }

    // 监听系统主题变化
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleThemeChange = (e: MediaQueryListEvent) => {
      if (!localStorage.getItem("theme")) {
        themeStore.setMode(e.matches ? "dark" : "light");
      }
    };

    mediaQuery.addEventListener("change", handleThemeChange);

    return () => {
      mediaQuery.removeEventListener("change", handleThemeChange);
    };
  });

  // 响应主题变化
  $: if (mounted) {
    document.documentElement.setAttribute("data-theme", $currentTheme);
    localStorage.setItem("theme", $currentTheme);
  }
</script>

<svelte:head>
  <title>Chat Box - AI 聊天助手</title>
  <meta name="description" content="基于 Rust + SvelteKit 的现代化 AI 聊天应用" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <link rel="icon" href="/favicon.ico" />
</svelte:head>

<div class="app" data-theme={$currentTheme}>
  <main class="main-content">
    <slot />
  </main>

  <!-- 全局通知组件 -->
  <Notification />

  <!-- 全局加载指示器 -->
  {#if !mounted}
    <div class="loading-overlay">
      <div class="loading-spinner">
        <div class="spinner"></div>
        <p>正在加载...</p>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(:root) {
    /* 光明主题色彩 */
    --color-primary: #667eea;
    --color-primary-hover: #5a67d8;
    --color-secondary: #4fd1c7;
    --color-secondary-hover: #38b2ac;
    --color-accent: #f56565;
    --color-success: #48bb78;
    --color-warning: #ed8936;
    --color-error: #f56565;

    /* 背景色 */
    --color-bg-primary: #ffffff;
    --color-bg-secondary: #f7fafc;
    --color-bg-tertiary: #edf2f7;
    --color-bg-overlay: rgba(255, 255, 255, 0.95);
    --color-bg-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);

    /* 文本色 */
    --color-text-primary: #2d3748;
    --color-text-secondary: #4a5568;
    --color-text-muted: #718096;
    --color-text-inverse: #ffffff;

    /* 边框色 */
    --color-border-light: #e2e8f0;
    --color-border-medium: #cbd5e0;
    --color-border-dark: #a0aec0;

    /* 阴影 */
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
    --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);
    --shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.15);

    /* 圆角 */
    --radius-sm: 4px;
    --radius-md: 8px;
    --radius-lg: 12px;
    --radius-xl: 16px;
    --radius-full: 9999px;

    /* 间距 */
    --spacing-xs: 0.25rem;
    --spacing-sm: 0.5rem;
    --spacing-md: 1rem;
    --spacing-lg: 1.5rem;
    --spacing-xl: 2rem;
    --spacing-2xl: 3rem;

    /* 字体 */
    --font-family-base:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    --font-family-mono:
      "SF Mono", Monaco, "Cascadia Code", "Roboto Mono", Consolas, "Courier New", monospace;

    /* 动画 */
    --transition-fast: 0.15s ease-in-out;
    --transition-base: 0.3s ease-in-out;
    --transition-slow: 0.5s ease-in-out;

    /* 模糊效果 */
    --blur-sm: blur(4px);
    --blur-md: blur(8px);
    --blur-lg: blur(12px);
  }

  :global([data-theme="dark"]) {
    /* 深色主题覆盖 */
    --color-bg-primary: #1a202c;
    --color-bg-secondary: #2d3748;
    --color-bg-tertiary: #4a5568;
    --color-bg-overlay: rgba(26, 32, 44, 0.95);
    --color-bg-gradient: linear-gradient(135deg, #2d3748 0%, #4a5568 100%);

    --color-text-primary: #f7fafc;
    --color-text-secondary: #e2e8f0;
    --color-text-muted: #a0aec0;
    --color-text-inverse: #1a202c;

    --color-border-light: #4a5568;
    --color-border-medium: #718096;
    --color-border-dark: #a0aec0;

    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);
    --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.3);
    --shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.4);
  }

  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: var(--font-family-base);
    line-height: 1.6;
    color: var(--color-text-primary);
    background: var(--color-bg-primary);
    overflow-x: hidden;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  :global(a) {
    color: var(--color-primary);
    text-decoration: none;
    transition: color var(--transition-fast);
  }

  :global(a:hover) {
    color: var(--color-primary-hover);
  }

  :global(button) {
    border: none;
    background: none;
    cursor: pointer;
    font-family: inherit;
    transition: all var(--transition-fast);
  }

  :global(input, textarea, select) {
    font-family: inherit;
    border: 1px solid var(--color-border-medium);
    border-radius: var(--radius-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    transition: all var(--transition-fast);
  }

  :global(input:focus, textarea:focus, select:focus) {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  :global(.btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    font-weight: 500;
    text-decoration: none;
    transition: all var(--transition-fast);
    cursor: pointer;
    border: none;
    font-family: inherit;
  }

  :global(.btn-primary) {
    background: var(--color-primary);
    color: var(--color-text-inverse);
  }

  :global(.btn-primary:hover) {
    background: var(--color-primary-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  :global(.btn-secondary) {
    background: var(--color-secondary);
    color: var(--color-text-inverse);
  }

  :global(.btn-secondary:hover) {
    background: var(--color-secondary-hover);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  :global(.btn-outline) {
    background: transparent;
    color: var(--color-primary);
    border: 1px solid var(--color-primary);
  }

  :global(.btn-outline:hover) {
    background: var(--color-primary);
    color: var(--color-text-inverse);
  }

  :global(.btn-ghost) {
    background: transparent;
    color: var(--color-text-secondary);
  }

  :global(.btn-ghost:hover) {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  :global(.btn-sm) {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 0.875rem;
  }

  :global(.btn-lg) {
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: 1.125rem;
  }

  :global(.btn:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none !important;
  }

  :global(.card) {
    background: var(--color-bg-overlay);
    backdrop-filter: var(--blur-md);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border-light);
    box-shadow: var(--shadow-md);
  }

  :global(.scrollbar-thin) {
    scrollbar-width: thin;
    scrollbar-color: var(--color-border-medium) transparent;
  }

  :global(.scrollbar-thin::-webkit-scrollbar) {
    width: 6px;
    height: 6px;
  }

  :global(.scrollbar-thin::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(.scrollbar-thin::-webkit-scrollbar-thumb) {
    background: var(--color-border-medium);
    border-radius: var(--radius-full);
  }

  :global(.scrollbar-thin::-webkit-scrollbar-thumb:hover) {
    background: var(--color-border-dark);
  }

  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--color-bg-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .loading-spinner {
    text-align: center;
    color: var(--color-text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border-light);
    border-top: 3px solid var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto var(--spacing-md);
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  /* 响应式设计 */
  @media (max-width: 768px) {
    :global(.btn) {
      padding: var(--spacing-sm);
    }

    :global(.btn-sm) {
      padding: var(--spacing-xs);
    }
  }

  /* 动画类 */
  :global(.fade-in) {
    animation: fadeIn 0.3s ease-in-out;
  }

  :global(.slide-up) {
    animation: slideUp 0.3s ease-out;
  }

  :global(.scale-in) {
    animation: scaleIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.9);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  /* 聊天相关的全局样式 */
  :global(.message-bubble) {
    max-width: 80%;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-lg);
    line-height: 1.5;
    word-wrap: break-word;
    animation: slideUp 0.3s ease-out;
  }

  :global(.message-user) {
    background: var(--color-primary);
    color: var(--color-text-inverse);
    align-self: flex-end;
    border-bottom-right-radius: var(--radius-sm);
  }

  :global(.message-assistant) {
    background: var(--color-bg-overlay);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-light);
    align-self: flex-start;
    border-bottom-left-radius: var(--radius-sm);
  }

  /* 打印样式 */
  @media print {
    .loading-overlay {
      display: none;
    }

    :global(.btn) {
      display: none;
    }
  }
</style>
