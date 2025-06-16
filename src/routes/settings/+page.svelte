<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { themeStore, themeOptions, type ThemeMode } from "$lib/stores/themeStore";
  import { chatStore } from "$lib/stores/chatStore";
  import type { UserPreferences, AppSettings, ChatSettings } from "$lib/types";
  import SettingsSection from "$lib/components/SettingsSection.svelte";
  import SettingsItem from "$lib/components/SettingsItem.svelte";

  // Toast 通知函数（简化版）
  function showToast(message: string, type: "success" | "error" | "warning" = "success") {
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

  // 设置状态
  let settings: UserPreferences = {
    app: {
      theme: "auto",
      language: "zh-CN",
      fontSize: "medium",
      autoSave: true,
      soundEnabled: true,
      notificationsEnabled: true,
    },
    chat: {
      maxMessages: 100,
      showTimestamps: true,
      enableMarkdown: true,
      autoScroll: true,
      typingIndicator: true,
    },
    voice: {
      enabled: false,
      language: "zh-CN",
      continuous: false,
      interimResults: true,
      maxAlternatives: 1,
    },
    shortcuts: [],
    quickActions: [],
    plugins: [],
  };

  let isLoading = false;
  let hasChanges = false;
  let activeSection = "general";

  // 语言选项
  const languageOptions = [
    { value: "zh-CN", label: "简体中文", flag: "🇨🇳" },
    { value: "zh-TW", label: "繁體中文", flag: "🇹🇼" },
    { value: "en-US", label: "English (US)", flag: "🇺🇸" },
    { value: "ja-JP", label: "日本語", flag: "🇯🇵" },
    { value: "ko-KR", label: "한국어", flag: "🇰🇷" },
  ];

  // 字体大小选项
  const fontSizeOptions = [
    { value: "small", label: "小", description: "14px" },
    { value: "medium", label: "中", description: "16px" },
    { value: "large", label: "大", description: "18px" },
  ];

  // 导航项
  const navigationItems = [
    { id: "general", label: "通用", icon: "⚙️" },
    { id: "appearance", label: "外观", icon: "🎨" },
    { id: "chat", label: "聊天", icon: "💬" },
    { id: "voice", label: "语音", icon: "🎤" },
    { id: "shortcuts", label: "快捷键", icon: "⌨️" },
    { id: "advanced", label: "高级", icon: "🔧" },
    { id: "about", label: "关于", icon: "ℹ️" },
  ];

  onMount(async () => {
    await loadSettings();
  });

  async function loadSettings() {
    isLoading = true;
    try {
      // 从本地存储加载设置
      const savedSettings = localStorage.getItem("user-preferences");
      if (savedSettings) {
        settings = { ...settings, ...JSON.parse(savedSettings) };
      }

      // 同步主题设置
      themeStore.subscribe(($theme) => {
        settings.app.theme = $theme.mode;
      });
    } catch (error) {
      console.error("加载设置失败:", error);
      showToast("加载设置失败", "error");
    } finally {
      isLoading = false;
    }
  }

  async function saveSettings() {
    isLoading = true;
    try {
      // 保存到本地存储
      localStorage.setItem("user-preferences", JSON.stringify(settings));

      // 应用主题设置
      themeStore.setMode(settings.app.theme);

      // 保存到 Tauri 后端（如果可用）
      if (typeof window !== "undefined" && (window as any).__TAURI__) {
        // await invoke('save_settings', { settings });
      }

      hasChanges = false;
      showToast("设置已保存", "success");
    } catch (error) {
      console.error("保存设置失败:", error);
      showToast("保存设置失败", "error");
    } finally {
      isLoading = false;
    }
  }

  async function resetSettings() {
    if (confirm("确定要重置所有设置吗？此操作无法撤销。")) {
      try {
        localStorage.removeItem("user-preferences");
        themeStore.reset();
        await loadSettings();
        showToast("设置已重置", "success");
      } catch (error) {
        console.error("重置设置失败:", error);
        showToast("重置设置失败", "error");
      }
    }
  }

  function markChanged() {
    hasChanges = true;
  }

  function handleThemeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    settings.app.theme = target.value as ThemeMode;
    themeStore.setMode(settings.app.theme);
    markChanged();
  }

  function handleFontSizeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settings.app.fontSize = target.value as "small" | "medium" | "large";

    // 应用字体大小
    const root = document.documentElement;
    const sizeMap = { small: "14px", medium: "16px", large: "18px" };
    root.style.fontSize = sizeMap[settings.app.fontSize];

    markChanged();
  }

  // 键盘快捷键
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case "s":
          event.preventDefault();
          if (hasChanges) saveSettings();
          break;
        case "r":
          event.preventDefault();
          resetSettings();
          break;
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<svelte:head>
  <title>设置 - Chat Box</title>
</svelte:head>

<div class="settings-page h-full flex">
  <!-- 侧边导航 -->
  <aside
    class="settings-sidebar w-64 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900"
  >
    <div class="p-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-50 mb-2">设置</h1>
      <p class="text-sm text-gray-600 dark:text-gray-400">自定义您的聊天体验</p>
    </div>

    <nav class="px-3">
      {#each navigationItems as item}
        <button
          class="nav-item w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-left transition-all duration-200 mb-1"
          class:active={activeSection === item.id}
          on:click={() => (activeSection = item.id)}
        >
          <span class="text-lg">{item.icon}</span>
          <span class="font-medium">{item.label}</span>
        </button>
      {/each}
    </nav>

    <!-- 底部操作 -->
    <div class="absolute bottom-4 left-4 right-4 space-y-2">
      <button
        on:click={saveSettings}
        disabled={!hasChanges || isLoading}
        class="w-full button-apple text-sm"
      >
        {isLoading ? "保存中..." : "保存设置"}
      </button>
      <button
        on:click={resetSettings}
        disabled={isLoading}
        class="w-full button-apple-secondary text-sm"
      >
        重置设置
      </button>
    </div>
  </aside>

  <!-- 主内容区 -->
  <main class="settings-content flex-1 overflow-y-auto">
    <div class="max-w-4xl mx-auto p-8">
      <!-- 通用设置 -->
      {#if activeSection === "general"}
        <SettingsSection title="通用设置" description="基本应用设置和偏好">
          <SettingsItem title="语言" description="选择应用界面语言">
            <select
              bind:value={settings.app.language}
              on:change={markChanged}
              class="input-apple w-48"
            >
              {#each languageOptions as option}
                <option value={option.value}>
                  {option.flag}
                  {option.label}
                </option>
              {/each}
            </select>
          </SettingsItem>

          <SettingsItem title="自动保存" description="自动保存聊天记录和设置">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.app.autoSave}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.app.autoSave}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="声音效果" description="启用消息提示音和其他音效">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.app.soundEnabled}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.app.soundEnabled}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="桌面通知" description="接收新消息的桌面通知">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.app.notificationsEnabled}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.app.notificationsEnabled}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>
        </SettingsSection>
      {/if}

      <!-- 外观设置 -->
      {#if activeSection === "appearance"}
        <SettingsSection title="外观设置" description="自定义应用的视觉效果">
          <SettingsItem title="主题模式" description="选择浅色、深色或跟随系统">
            <select
              value={settings.app.theme}
              on:change={handleThemeChange}
              class="input-apple w-48"
            >
              {#each themeOptions as option}
                <option value={option.value}>
                  {option.icon}
                  {option.label}
                </option>
              {/each}
            </select>
          </SettingsItem>

          <SettingsItem title="字体大小" description="调整界面文字大小">
            <div class="space-y-3">
              {#each fontSizeOptions as option}
                <label class="flex items-center space-x-3 cursor-pointer">
                  <input
                    type="radio"
                    bind:group={settings.app.fontSize}
                    value={option.value}
                    on:change={handleFontSizeChange}
                    class="radio-apple"
                  />
                  <div>
                    <div class="font-medium">{option.label}</div>
                    <div class="text-sm text-gray-600 dark:text-gray-400">{option.description}</div>
                  </div>
                </label>
              {/each}
            </div>
          </SettingsItem>
        </SettingsSection>
      {/if}

      <!-- 聊天设置 -->
      {#if activeSection === "chat"}
        <SettingsSection title="聊天设置" description="配置聊天相关功能">
          <SettingsItem title="最大消息数" description="单个对话中保留的最大消息数量">
            <div class="flex items-center space-x-4">
              <input
                type="range"
                min="50"
                max="500"
                step="50"
                bind:value={settings.chat.maxMessages}
                on:input={markChanged}
                class="flex-1 accent-blue-500"
              />
              <span class="w-16 text-center bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
                {settings.chat.maxMessages}
              </span>
            </div>
          </SettingsItem>

          <SettingsItem title="显示时间戳" description="在消息旁显示发送时间">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.chat.showTimestamps}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.chat.showTimestamps}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="Markdown 支持" description="启用消息中的 Markdown 格式">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.chat.enableMarkdown}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.chat.enableMarkdown}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="自动滚动" description="收到新消息时自动滚动到底部">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.chat.autoScroll}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.chat.autoScroll}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="输入指示器" description="显示对方正在输入的提示">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.chat.typingIndicator}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.chat.typingIndicator}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>
        </SettingsSection>
      {/if}

      <!-- 语音设置 -->
      {#if activeSection === "voice"}
        <SettingsSection title="语音设置" description="配置语音识别和语音合成">
          <SettingsItem title="启用语音功能" description="开启语音输入和语音播放功能">
            <label class="flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={settings.voice.enabled}
                on:change={markChanged}
                class="sr-only"
              />
              <div class="toggle-switch" class:active={settings.voice.enabled}>
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          {#if settings.voice.enabled}
            <SettingsItem title="语音语言" description="选择语音识别的语言">
              <select
                bind:value={settings.voice.language}
                on:change={markChanged}
                class="input-apple w-48"
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English (US)</option>
                <option value="ja-JP">日本語</option>
              </select>
            </SettingsItem>

            <SettingsItem title="连续识别" description="保持语音识别处于活跃状态">
              <label class="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={settings.voice.continuous}
                  on:change={markChanged}
                  class="sr-only"
                />
                <div class="toggle-switch" class:active={settings.voice.continuous}>
                  <div class="toggle-thumb"></div>
                </div>
              </label>
            </SettingsItem>

            <SettingsItem title="实时结果" description="显示语音识别的实时结果">
              <label class="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={settings.voice.interimResults}
                  on:change={markChanged}
                  class="sr-only"
                />
                <div class="toggle-switch" class:active={settings.voice.interimResults}>
                  <div class="toggle-thumb"></div>
                </div>
              </label>
            </SettingsItem>
          {/if}
        </SettingsSection>
      {/if}

      <!-- 快捷键设置 -->
      {#if activeSection === "shortcuts"}
        <SettingsSection title="快捷键" description="自定义键盘快捷键">
          <div class="space-y-4">
            <div
              class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4"
            >
              <h4 class="font-medium text-blue-900 dark:text-blue-100 mb-2">默认快捷键</h4>
              <div class="space-y-2 text-sm text-blue-800 dark:text-blue-200">
                <div class="flex justify-between">
                  <span>保存设置</span>
                  <kbd class="kbd">Ctrl/Cmd + S</kbd>
                </div>
                <div class="flex justify-between">
                  <span>重置设置</span>
                  <kbd class="kbd">Ctrl/Cmd + R</kbd>
                </div>
                <div class="flex justify-between">
                  <span>切换主题</span>
                  <kbd class="kbd">Ctrl/Cmd + T</kbd>
                </div>
                <div class="flex justify-between">
                  <span>新建对话</span>
                  <kbd class="kbd">Ctrl/Cmd + N</kbd>
                </div>
              </div>
            </div>

            <p class="text-gray-600 dark:text-gray-400">自定义快捷键功能正在开发中...</p>
          </div>
        </SettingsSection>
      {/if}

      <!-- 高级设置 -->
      {#if activeSection === "advanced"}
        <SettingsSection title="高级设置" description="开发者和高级用户选项">
          <SettingsItem title="开发者模式" description="启用调试功能和开发者工具">
            <label class="flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only" />
              <div class="toggle-switch">
                <div class="toggle-thumb"></div>
              </div>
            </label>
          </SettingsItem>

          <SettingsItem title="清除缓存" description="清除应用缓存和临时文件">
            <button class="button-apple-secondary"> 清除缓存 </button>
          </SettingsItem>

          <SettingsItem title="导出设置" description="将当前设置导出为文件">
            <button class="button-apple-secondary"> 导出设置 </button>
          </SettingsItem>

          <SettingsItem title="导入设置" description="从文件导入设置配置">
            <button class="button-apple-secondary"> 导入设置 </button>
          </SettingsItem>
        </SettingsSection>
      {/if}

      <!-- 关于页面 -->
      {#if activeSection === "about"}
        <SettingsSection title="关于 Chat Box" description="应用信息和版本详情">
          <div class="space-y-6">
            <!-- 应用信息 -->
            <div class="text-center">
              <div
                class="w-24 h-24 mx-auto bg-gradient-to-br from-blue-500 to-purple-600 rounded-2xl flex items-center justify-center mb-4"
              >
                <svg
                  class="w-12 h-12 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                  />
                </svg>
              </div>
              <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-50">Chat Box</h2>
              <p class="text-gray-600 dark:text-gray-400">现代化 AI 聊天应用</p>
              <p class="text-sm text-gray-500 dark:text-gray-500 mt-2">版本 0.1.0</p>
            </div>

            <!-- 技术信息 -->
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
                <h4 class="font-medium text-gray-900 dark:text-gray-50 mb-2">前端技术</h4>
                <ul class="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                  <li>SvelteKit</li>
                  <li>TypeScript</li>
                  <li>Tailwind CSS</li>
                  <li>Skeleton UI</li>
                </ul>
              </div>
              <div class="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
                <h4 class="font-medium text-gray-900 dark:text-gray-50 mb-2">后端技术</h4>
                <ul class="text-sm text-gray-600 dark:text-gray-400 space-y-1">
                  <li>Rust</li>
                  <li>Tauri</li>
                  <li>SQLite</li>
                  <li>Tokio</li>
                </ul>
              </div>
            </div>

            <!-- 链接 -->
            <div class="flex justify-center space-x-4">
              <button class="button-apple-ghost text-sm"> 📖 用户手册 </button>
              <button class="button-apple-ghost text-sm"> 🐛 反馈问题 </button>
              <button class="button-apple-ghost text-sm"> ⭐ GitHub </button>
            </div>

            <!-- 版权信息 -->
            <div
              class="text-center text-sm text-gray-500 dark:text-gray-500 border-t border-gray-200 dark:border-gray-700 pt-4"
            >
              <p>© 2024 Chat Box Team. All rights reserved.</p>
              <p>基于 MIT 许可证开源</p>
            </div>
          </div>
        </SettingsSection>
      {/if}

      <!-- 底部提示 -->
      {#if hasChanges}
        <div
          class="fixed bottom-4 right-4 bg-orange-100 dark:bg-orange-900 border border-orange-300 dark:border-orange-700 text-orange-800 dark:text-orange-200 px-4 py-2 rounded-lg shadow-lg animate-slide-up"
        >
          <div class="flex items-center space-x-2">
            <span>⚠️</span>
            <span class="text-sm">您有未保存的更改</span>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  .settings-page {
    height: calc(100vh - 120px); /* 减去头部和底部的高度 */
  }

  .settings-sidebar {
    position: relative;
    min-height: 100%;
  }

  .nav-item {
    color: rgb(107 114 128);
    transition: all 0.2s ease;
  }

  .nav-item:hover {
    background: rgb(243 244 246);
    color: rgb(17 24 39);
  }

  :global(.dark) .nav-item:hover {
    background: rgb(55 65 81);
    color: rgb(243 244 246);
  }

  .nav-item.active {
    background: rgb(59 130 246);
    color: white;
  }

  .nav-item.active:hover {
    background: rgb(37 99 235);
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    background: rgb(209 213 219);
    border-radius: 12px;
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .toggle-switch.active {
    background: rgb(59 130 246);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    transition: all 0.2s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(20px);
  }

  .radio-apple {
    appearance: none;
    width: 20px;
    height: 20px;
    border: 2px solid rgb(209 213 219);
    border-radius: 50%;
    position: relative;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .radio-apple:checked {
    border-color: rgb(59 130 246);
    background: rgb(59 130 246);
  }

  .radio-apple:checked::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 8px;
    height: 8px;
    background: white;
    border-radius: 50%;
  }

  .kbd {
    background: rgb(243 244 246);
    border: 1px solid rgb(209 213 219);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 0.75rem;
    font-family: monospace;
    color: rgb(75 85 99);
  }

  :global(.dark) .kbd {
    background: rgb(55 65 81);
    border: 1px solid rgb(75 85 99);
    color: rgb(209 213 219);
  }

  /* 响应式调整 */
  @media (max-width: 768px) {
    .settings-page {
      flex-direction: column;
    }

    .settings-sidebar {
      width: 100%;
      position: relative;
      min-height: auto;
    }

    .settings-sidebar nav {
      display: flex;
      overflow-x: auto;
      padding: 0 1rem;
    }

    .nav-item {
      flex-shrink: 0;
      white-space: nowrap;
    }
  }
</style>
