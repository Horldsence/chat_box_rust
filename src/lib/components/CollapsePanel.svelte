<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let collapsed = false;

  // 菜单项定义
  const menuItems = [
    {
      id: "chat",
      label: "聊天",
      icon: "💬",
      description: "AI 对话聊天",
    },
    {
      id: "conversations",
      label: "对话列表",
      icon: "📋",
      description: "查看所有对话",
    },
    {
      id: "ai-model",
      label: "AI 模型",
      icon: "🤖",
      description: "AI 模型设置",
    },
    {
      id: "ai-behavior",
      label: "AI 行为",
      icon: "⚙️",
      description: "AI 行为配置",
    },
    {
      id: "voice",
      label: "语音设置",
      icon: "🎤",
      description: "语音输入输出",
    },
    {
      id: "ui",
      label: "界面设置",
      icon: "🎨",
      description: "界面主题配置",
    },
    {
      id: "database",
      label: "数据库",
      icon: "💾",
      description: "数据存储管理",
    },
    {
      id: "candle-test",
      label: "Candle 测试",
      icon: "🧪",
      description: "本地模型测试",
    },
    {
      id: "guide",
      label: "使用指南",
      icon: "📖",
      description: "帮助文档",
    },
    {
      id: "about",
      label: "关于",
      icon: "ℹ️",
      description: "应用信息",
    },
  ];

  let activeItem = "chat";

  function selectItem(item: any) {
    activeItem = item.id;
    dispatch("select", item.id);
  }

  function toggleCollapse() {
    collapsed = !collapsed;
  }
</script>

<div class="collapse-panel" class:collapsed>
  <!-- 顶部控制按钮 -->
  <div class="panel-header">
    <button
      class="collapse-toggle"
      on:click={toggleCollapse}
      title={collapsed ? "展开菜单" : "收起菜单"}
      aria-label={collapsed ? "展开菜单" : "收起菜单"}
    >
      <svg
        class="toggle-icon"
        class:rotated={collapsed}
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
        />
      </svg>
    </button>

    {#if !collapsed}
      <div class="panel-title">
        <h2>菜单</h2>
      </div>
    {/if}
  </div>

  <!-- 菜单项列表 -->
  <nav class="menu-nav">
    {#each menuItems as item}
      <button
        class="menu-item"
        class:active={activeItem === item.id}
        on:click={() => selectItem(item)}
        title={collapsed ? `${item.label} - ${item.description}` : item.description}
      >
        <span class="menu-icon">{item.icon}</span>
        {#if !collapsed}
          <div class="menu-content">
            <span class="menu-label">{item.label}</span>
            <span class="menu-desc">{item.description}</span>
          </div>
        {/if}
      </button>
    {/each}
  </nav>

  <!-- 底部信息 -->
  {#if !collapsed}
    <div class="panel-footer">
      <div class="app-info">
        <div class="app-name">Chat Box</div>
        <div class="app-version">v0.1.0</div>
      </div>
    </div>
  {/if}
</div>

<style>
  .collapse-panel {
    width: 280px;
    height: 100vh;
    background: #ffffff;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    transition: width 0.3s ease;
    position: relative;
    z-index: 100;
  }

  .collapse-panel.collapsed {
    width: 64px;
  }

  :global(.dark) .collapse-panel {
    background: #1f2937;
    border-right: 1px solid #374151;
  }

  .panel-header {
    padding: 16px;
    border-bottom: 1px solid #e5e7eb;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 60px;
  }

  :global(.dark) .panel-header {
    border-bottom: 1px solid #374151;
  }

  .collapse-toggle {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: #6b7280;
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .collapse-toggle:hover {
    background: #f3f4f6;
    color: #374151;
  }

  :global(.dark) .collapse-toggle:hover {
    background: #374151;
    color: #d1d5db;
  }

  .toggle-icon {
    width: 16px;
    height: 16px;
    transition: transform 0.3s ease;
  }

  .toggle-icon.rotated {
    transform: rotate(180deg);
  }

  .panel-title {
    flex: 1;
    min-width: 0;
  }

  .panel-title h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #111827;
    white-space: nowrap;
  }

  :global(.dark) .panel-title h2 {
    color: #f9fafb;
  }

  .menu-nav {
    flex: 1;
    padding: 8px 12px;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #d1d5db transparent;
  }

  .menu-nav::-webkit-scrollbar {
    width: 4px;
  }

  .menu-nav::-webkit-scrollbar-track {
    background: transparent;
  }

  .menu-nav::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 2px;
  }

  :global(.dark) .menu-nav {
    scrollbar-color: #4b5563 transparent;
  }

  :global(.dark) .menu-nav::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  .menu-item {
    width: 100%;
    padding: 12px 8px;
    margin: 2px 0;
    border: none;
    background: transparent;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    color: #6b7280;
    position: relative;
  }

  .menu-item:hover {
    background: #f3f4f6;
    color: #374151;
  }

  .menu-item.active {
    background: #3b82f6;
    color: #ffffff;
  }

  :global(.dark) .menu-item {
    color: #9ca3af;
  }

  :global(.dark) .menu-item:hover {
    background: #374151;
    color: #d1d5db;
  }

  :global(.dark) .menu-item.active {
    background: #3b82f6;
    color: #ffffff;
  }

  .menu-icon {
    font-size: 20px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .menu-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .menu-label {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .menu-desc {
    font-size: 12px;
    opacity: 0.8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .panel-footer {
    padding: 16px;
    border-top: 1px solid #e5e7eb;
    margin-top: auto;
  }

  :global(.dark) .panel-footer {
    border-top: 1px solid #374151;
  }

  .app-info {
    text-align: center;
  }

  .app-name {
    font-size: 14px;
    font-weight: 600;
    color: #374151;
    margin-bottom: 2px;
  }

  .app-version {
    font-size: 12px;
    color: #6b7280;
  }

  :global(.dark) .app-name {
    color: #d1d5db;
  }

  :global(.dark) .app-version {
    color: #9ca3af;
  }

  /* 响应式调整 */
  @media (max-width: 768px) {
    .collapse-panel {
      position: fixed;
      left: 0;
      top: 0;
      z-index: 1000;
      box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
    }

    .collapse-panel.collapsed {
      transform: translateX(-100%);
    }
  }

  /* 减少动画效果以提高性能 */
  @media (prefers-reduced-motion: reduce) {
    .collapse-panel,
    .toggle-icon,
    .menu-item {
      transition: none;
    }
  }
</style>
