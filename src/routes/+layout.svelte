<script lang="ts">
  import { onMount } from "svelte";
  import { errorService } from "$lib/services/ErrorService";
  import { themeStore, currentTheme } from "$lib/stores/themeStore";
  import "../app.css";

  let mounted = false;

  onMount(() => {
    mounted = true;

    // Initialize error handling
    errorService.init();

    // Setup theme system
    initializeTheme();

    // Setup error notifications
    setupErrorNotifications();

    return () => {
      // Cleanup
    };
  });

  function initializeTheme() {
    // Initialize theme store
    themeStore.init();
  }

  function setupErrorNotifications() {
    // Setup global error handling with simple notifications
    window.addEventListener("error", (event) => {
      console.error("Global error:", event.error);
      showSimpleToast(`错误: ${event.error?.message || "未知错误"}`, "error");
    });

    window.addEventListener("unhandledrejection", (event) => {
      console.error("Unhandled promise rejection:", event.reason);
      showSimpleToast(`Promise 错误: ${event.reason?.message || "未知错误"}`, "error");
    });
  }

  function showSimpleToast(message: string, type: "success" | "error" | "warning" = "success") {
    const toast = document.createElement("div");
    toast.className = `fixed top-4 right-4 px-4 py-2 rounded-lg shadow-lg text-white z-50 animate-slide-down ${
      type === "success" ? "bg-green-500" : type === "error" ? "bg-red-500" : "bg-orange-500"
    }`;
    toast.textContent = message;
    document.body.appendChild(toast);

    setTimeout(() => {
      toast.remove();
    }, 3000);
  }

  // Reactive theme application
  $: if (mounted && $currentTheme) {
    // Apply theme to document
    document.documentElement.classList.remove("light", "dark");
    document.documentElement.classList.add($currentTheme);

    // Apply to body class for CSS styling
    if ($currentTheme === "dark") {
      document.body.classList.add("dark");
    } else {
      document.body.classList.remove("dark");
    }
  }
</script>

<svelte:head>
  <title>Chat Box - AI 聊天助手</title>
  <meta name="description" content="基于 Rust + SvelteKit 的现代化 AI 聊天应用" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta name="theme-color" content={$currentTheme === "dark" ? "#1f2937" : "#ffffff"} />
  <link rel="icon" href="/favicon.ico" />

  <!-- Apple-specific meta tags -->
  <meta name="apple-mobile-web-app-capable" content="yes" />
  <meta
    name="apple-mobile-web-app-status-bar-style"
    content={$currentTheme === "dark" ? "black-translucent" : "default"}
  />
  <meta name="apple-mobile-web-app-title" content="Chat Box" />

  <!-- Prevent FOUC (Flash of Unstyled Content) -->
  <script>
    // Apply theme immediately to prevent flash
    (function () {
      const theme =
        localStorage.getItem("theme-mode") ||
        (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
      document.documentElement.classList.add(theme);
      if (theme === "dark") {
        document.body.classList.add("dark");
      }
    })();
  </script>
</svelte:head>

<div class="app-shell min-h-screen">
  <!-- Main Content Area -->
  <div class="app-content h-screen">
    {#if mounted}
      <div class="animate-fade-in h-full">
        <slot />
      </div>
    {:else}
      <!-- Loading State -->
      <div class="flex items-center justify-center h-full bg-gray-50 dark:bg-gray-900">
        <div class="flex flex-col items-center space-y-4">
          <div class="animate-spin">
            <div class="w-8 h-8 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full">
              <div
                class="w-8 h-8 border-2 border-gray-100 border-t-transparent rounded-full animate-spin"
              ></div>
            </div>
          </div>
          <p class="text-gray-600 dark:text-gray-300 text-sm">正在加载...</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .app-shell {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .app-content {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  /* Smooth theme transitions */
  :global(html) {
    transition: background-color 0.3s ease;
  }

  /* Loading animation refinements */
  .animate-spin > div {
    animation-duration: 1s;
  }

  /* Focus improvements for accessibility */
  :global(button:focus-visible) {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }

  :global(a:focus-visible) {
    outline: 2px solid #3b82f6;
    outline-offset: 2px;
  }

  /* Fade in animation */
  .animate-fade-in {
    animation: fade-in 0.3s ease;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  /* Toast slide down animation */
  :global(.animate-slide-down) {
    animation: slide-down 0.3s ease-out;
  }

  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .animate-fade-in,
    .animate-spin,
    :global(.animate-slide-down) {
      animation: none;
    }

    :global(html) {
      transition: none;
    }
  }
</style>
