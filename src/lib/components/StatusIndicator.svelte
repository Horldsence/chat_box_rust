<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let status: "loading" | "success" | "error" | "warning" | "info" | "idle" = "idle";
  export let message: string = "";
  export let details: string = "";
  export let showIcon: boolean = true;
  export let showMessage: boolean = true;
  export let showDetails: boolean = false;
  export let dismissible: boolean = false;
  export let size: "sm" | "md" | "lg" = "md";
  export let variant: "inline" | "toast" | "badge" = "inline";
  export let animateEntry: boolean = true;
  export let autoHide: boolean = false;
  export let autoHideDelay: number = 5000;

  const dispatch = createEventDispatcher<{
    dismiss: void;
    click: void;
    actionClick: { action: string };
  }>();

  let visible = true;
  let autoHideTimer: number | null = null;

  // 状态图标映射
  const statusIcons = {
    loading: "🔄",
    success: "✅",
    error: "❌",
    warning: "⚠️",
    info: "ℹ️",
    idle: "⚪",
  };

  // 状态颜色类映射
  const statusClasses = {
    loading:
      "bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800",
    success:
      "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800",
    error:
      "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 border-red-200 dark:border-red-800",
    warning:
      "bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-300 border-yellow-200 dark:border-yellow-800",
    info: "bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800",
    idle: "bg-gray-50 dark:bg-gray-800 text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-700",
  };

  // SVG 图标组件
  const statusSvgIcons = {
    loading: `<svg class="animate-spin" width="16" height="16" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-dasharray="9 3"/>
    </svg>`,
    success: `<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="8" fill="currentColor"/>
      <path d="M6 8l2 2 4-4" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>`,
    error: `<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="8" fill="currentColor"/>
      <path d="M6 6l4 4M10 6l-4 4" stroke="white" stroke-width="2" stroke-linecap="round"/>
    </svg>`,
    warning: `<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path d="M8 1l7 14H1L8 1z" fill="currentColor"/>
      <path d="M8 6v4M8 12h.01" stroke="white" stroke-width="2" stroke-linecap="round"/>
    </svg>`,
    info: `<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="8" fill="currentColor"/>
      <path d="M8 6v4M8 10h.01" stroke="white" stroke-width="2" stroke-linecap="round"/>
    </svg>`,
    idle: `<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="2"/>
    </svg>`,
  };

  $: sizeClasses = {
    sm: "text-xs p-2",
    md: "text-sm p-3",
    lg: "text-base p-4",
  };

  $: iconSizeClasses = {
    sm: "w-3 h-3",
    md: "w-4 h-4",
    lg: "w-5 h-5",
  };

  $: variantClasses = {
    inline: "rounded-lg border",
    toast: "rounded-lg border shadow-lg",
    badge: "rounded-full border px-3 py-1",
  };

  // 设置自动隐藏
  $: if (autoHide && visible && status !== "loading") {
    if (autoHideTimer) {
      clearTimeout(autoHideTimer);
    }
    autoHideTimer = setTimeout(() => {
      handleDismiss();
    }, autoHideDelay) as unknown as number;
  }

  function handleDismiss() {
    if (dismissible) {
      visible = false;
      dispatch("dismiss");
    }
  }

  function handleClick() {
    dispatch("click");
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleClick();
    }
  }

  // 清理定时器
  function cleanup() {
    if (autoHideTimer) {
      clearTimeout(autoHideTimer);
      autoHideTimer = null;
    }
  }

  // 组件销毁时清理
  import { onDestroy } from "svelte";
  onDestroy(cleanup);
</script>

{#if visible}
  {#if $$slots.default || message}
    <button
      type="button"
      class="status-indicator {statusClasses[status]} {sizeClasses[size]} {variantClasses[variant]}"
      class:animate-entry={animateEntry}
      class:clickable={$$slots.default || message}
      role={dismissible ? "alert" : "status"}
      aria-live="polite"
      aria-atomic="true"
      on:click={handleClick}
      on:keydown={handleKeydown}
    >
      <div class="flex items-start space-x-2">
        <!-- 状态图标 -->
        {#if showIcon}
          <div class="status-icon {iconSizeClasses[size]} flex-shrink-0 mt-0.5" aria-hidden="true">
            {@html statusSvgIcons[status]}
          </div>
        {/if}

        <!-- 内容区域 -->
        <div class="flex-1 min-w-0">
          {#if showMessage && message}
            <p class="status-message font-medium">{message}</p>
          {/if}

          {#if showDetails && details}
            <p class="status-details mt-1 text-xs opacity-75">{details}</p>
          {/if}

          <!-- 插槽内容 -->
          <slot />
        </div>

        <!-- 关闭按钮 -->
        {#if dismissible}
          <button
            type="button"
            class="dismiss-button flex-shrink-0 p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors duration-150"
            on:click|stopPropagation={handleDismiss}
            aria-label="关闭"
          >
            <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none">
              <path
                d="M4 4l8 8M12 4l-8 8"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          </button>
        {/if}
      </div>

      <!-- 进度条（仅在加载状态显示） -->
      {#if status === "loading" && variant !== "badge"}
        <div
          class="progress-bar mt-2 h-1 bg-black/10 dark:bg-white/10 rounded-full overflow-hidden"
        >
          <div class="progress-fill h-full bg-current rounded-full animate-pulse"></div>
        </div>
      {/if}
    </button>
  {:else}
    <div
      class="status-indicator {statusClasses[status]} {sizeClasses[size]} {variantClasses[variant]}"
      class:animate-entry={animateEntry}
      role={dismissible ? "alert" : "status"}
      aria-live="polite"
      aria-atomic="true"
    >
      <div class="flex items-start space-x-2">
        <!-- 状态图标 -->
        {#if showIcon}
          <div class="status-icon {iconSizeClasses[size]} flex-shrink-0 mt-0.5" aria-hidden="true">
            {@html statusSvgIcons[status]}
          </div>
        {/if}

        <!-- 内容区域 -->
        <div class="flex-1 min-w-0">
          {#if showMessage && message}
            <p class="status-message font-medium">{message}</p>
          {/if}

          {#if showDetails && details}
            <p class="status-details mt-1 text-xs opacity-75">{details}</p>
          {/if}

          <!-- 插槽内容 -->
          <slot />
        </div>

        <!-- 关闭按钮 -->
        {#if dismissible}
          <button
            type="button"
            class="dismiss-button flex-shrink-0 p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors duration-150"
            on:click|stopPropagation={handleDismiss}
            aria-label="关闭"
          >
            <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none">
              <path
                d="M4 4l8 8M12 4l-8 8"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          </button>
        {/if}
      </div>

      <!-- 进度条（仅在加载状态显示） -->
      {#if status === "loading" && variant !== "badge"}
        <div
          class="progress-bar mt-2 h-1 bg-black/10 dark:bg-white/10 rounded-full overflow-hidden"
        >
          <div class="progress-fill h-full bg-current rounded-full animate-pulse"></div>
        </div>
      {/if}
    </div>
  {/if}
{/if}

<style>
  .status-indicator {
    transition: all 0.2s;
  }

  .status-indicator.clickable {
    cursor: pointer;
    background: none;
    border: none;
    text-align: left;
  }

  .status-indicator.clickable:hover {
    transform: scale(1.02);
  }

  .status-indicator.clickable:focus {
    outline: none;
    box-shadow:
      0 0 0 2px currentColor,
      0 0 0 4px transparent;
    opacity: 0.5;
  }

  .status-indicator.animate-entry {
    animation: slideIn 0.3s ease-out;
  }

  .status-message {
    line-height: 1.25;
  }

  .status-details {
    line-height: 1.625;
  }

  .dismiss-button:focus {
    outline: none;
    box-shadow: 0 0 0 2px currentColor;
    opacity: 0.5;
  }

  .progress-fill {
    animation: loading-progress 2s ease-in-out infinite;
  }

  /* 动画定义 */
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes loading-progress {
    0%,
    100% {
      transform: translateX(-100%);
    }
    50% {
      transform: translateX(100%);
    }
  }

  /* 响应式调整 */
  @media (max-width: 640px) {
    .status-indicator {
      font-size: 0.75rem;
    }
  }

  /* 高对比度模式 */
  @media (prefers-contrast: high) {
    .status-indicator {
      border-width: 2px;
    }
  }

  /* 减少动画模式 */
  @media (prefers-reduced-motion: reduce) {
    .status-indicator,
    .status-icon svg,
    .progress-fill {
      transition: none;
    }

    .status-indicator.animate-entry {
      animation: none;
    }

    @keyframes slideIn {
      from,
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }

    @keyframes loading-progress {
      from,
      to {
        transform: translateX(0);
      }
    }
  }

  /* 暗色主题特殊处理 */
  @media (prefers-color-scheme: dark) {
    .status-indicator {
      box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    }
  }
</style>
