<script lang="ts">
  import { onMount } from "svelte";
  import { writable } from "svelte/store";
  import { themeStore, currentTheme } from "$lib/stores/themeStore";
  import {
    chatStore,
    conversations,
    currentConversation,
    messages,
    isLoading,
    isTyping,
    isConnected,
  } from "$lib/stores/chatStore";
  import { settingsStore, config, healthStatus } from "$lib/stores/settingsStore";
  import CollapsePanel from "$lib/components/CollapsePanel.svelte";
  import type { Conversation, Message } from "$lib/types";

  // 应用状态
  let currentView = "chat";
  let sidebarWidth = "280px";
  let messageInput = "";
  let messagesContainer: HTMLElement;
  let showInitialization = false;

  // 快速操作建议
  const quickActions = [
    {
      icon: "🤖",
      title: "AI 介绍",
      description: "了解 AI 技术和应用",
      prompt: "请介绍一下人工智能技术的发展现状和应用领域",
    },
    {
      icon: "💻",
      title: "编程帮助",
      description: "代码示例和技术指导",
      prompt: "我想学习编程，可以给我一些建议和入门指导吗？",
    },
    {
      icon: "📚",
      title: "学习资源",
      description: "推荐优质学习材料",
      prompt: "可以推荐一些优质的在线学习资源和平台吗？",
    },
    {
      icon: "🔧",
      title: "技术问答",
      description: "解答技术疑问",
      prompt: "我在开发中遇到了问题，可以帮我分析一下吗？",
    },
  ];

  function showToast(message: string, type: "success" | "error" | "info" = "success") {
    const toast = document.createElement("div");
    toast.className = `fixed top-4 right-4 px-4 py-2 rounded-lg shadow-lg text-white z-50 animate-slide-down ${
      type === "success" ? "bg-green-500" : type === "error" ? "bg-red-500" : "bg-blue-500"
    }`;
    toast.textContent = message;
    document.body.appendChild(toast);

    setTimeout(() => {
      toast.remove();
    }, 3000);
  }

  async function initializeApp() {
    try {
      // 初始化聊天存储
      await chatStore.init();

      // 初始化设置存储
      await settingsStore.init();

      // 检查连接状态
      const connected = await chatStore.checkConnection();
      if (!connected) {
        showToast("后端连接失败", "error");
      }

      showToast("应用初始化完成", "success");
    } catch (error) {
      console.error("应用初始化失败:", error);
      showToast("应用初始化失败", "error");
    }
  }

  // 菜单选择处理
  function handleMenuSelect(event: CustomEvent) {
    const menuItem = event.detail;
    console.log("选中菜单项:", menuItem);
    currentView = menuItem;

    // 如果是切换到聊天视图，确保有一个被选中的对话
    if (menuItem === "chat" && !$currentConversation && $conversations.length > 0) {
      selectConversation($conversations[0].id);
    }
  }

  // 选择对话
  async function selectConversation(id: number) {
    try {
      await chatStore.selectConversation(id);
      showToast(`切换到对话`, "info");
    } catch (error) {
      console.error("选择对话失败:", error);
      showToast("切换对话失败", "error");
    }
  }

  // 创建新对话
  async function createNewConversation() {
    try {
      const title = `新对话 ${new Date().toLocaleString("zh-CN")}`;
      const newConv = await chatStore.createConversation(title);
      currentView = "chat";
      showToast("创建新对话成功", "success");
    } catch (error) {
      console.error("创建对话失败:", error);
      showToast("创建对话失败", "error");
    }
  }

  // 发送消息
  async function sendMessage() {
    if (!messageInput.trim() || $isLoading) return;

    const content = messageInput.trim();
    messageInput = "";

    try {
      await chatStore.sendMessage(content);
      scrollToBottom();
    } catch (error) {
      console.error("发送消息失败:", error);
      showToast("发送消息失败，请重试", "error");
    }
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (messagesContainer) {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      }
    }, 100);
  }

  function useQuickAction(action: any) {
    messageInput = action.prompt;
    sendMessage();
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  onMount(async () => {
    // 初始化应用
    await initializeApp();
    scrollToBottom();
  });
</script>

<svelte:head>
  <title>Chat Box - AI 聊天助手</title>
</svelte:head>

<div class="app-container">
  <!-- 主菜单 -->
  <CollapsePanel on:select={handleMenuSelect} />

  <div class="main-container">
    <!-- 对话列表侧边栏 (仅在聊天视图显示) -->
    {#if currentView === "chat"}
      <aside class="sidebar" style="width: {sidebarWidth}">
        <div class="sidebar-header">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-50">对话列表</h2>
            <div class="flex items-center space-x-2">
              <!-- 连接状态指示器 -->
              <div class="flex items-center space-x-1">
                <div
                  class="w-2 h-2 rounded-full {$isConnected ? 'bg-green-500' : 'bg-red-500'}"
                ></div>
                <span class="text-xs text-gray-500 dark:text-gray-400">
                  {$isConnected ? "已连接" : "未连接"}
                </span>
              </div>
              <button
                on:click={createNewConversation}
                class="p-2 text-gray-500 hover:text-blue-500 dark:text-gray-400 dark:hover:text-blue-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200"
                title="新建对话"
                aria-label="新建对话"
                disabled={!$isConnected}
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 4v16m8-8H4"
                  />
                </svg>
              </button>
            </div>
          </div>

          <!-- 搜索 -->
          <div class="relative">
            <input
              type="text"
              placeholder="搜索对话..."
              class="w-full pl-10 pr-4 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-50 placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg
                class="w-4 h-4 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
          </div>
        </div>

        <!-- 对话列表 -->
        <div class="conversation-list">
          {#if $isLoading}
            <div class="flex items-center justify-center py-8">
              <div
                class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"
              ></div>
            </div>
          {:else if $conversations.length === 0}
            <div class="text-center py-8">
              <p class="text-sm text-gray-500 dark:text-gray-400">还没有对话记录</p>
              <button
                on:click={createNewConversation}
                class="mt-2 text-sm text-blue-500 hover:text-blue-600"
                disabled={!$isConnected}
                aria-label="创建第一个对话"
              >
                创建第一个对话
              </button>
            </div>
          {:else}
            {#each $conversations as conversation}
              <button
                class="conversation-item"
                class:active={$currentConversation?.id === conversation.id}
                on:click={() => selectConversation(conversation.id)}
                disabled={!$isConnected}
                aria-label="选择对话 {conversation.title}"
              >
                <div class="flex items-start justify-between">
                  <div class="flex-1 min-w-0">
                    <h3 class="conversation-title">
                      {conversation.title}
                    </h3>
                    <p class="conversation-preview">
                      {conversation.last_message}
                    </p>
                  </div>
                  <time class="conversation-time">
                    {formatTime(conversation.timestamp)}
                  </time>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </aside>
    {/if}

    <!-- 主内容区域 -->
    <main class="main-content">
      <!-- 聊天视图 -->
      {#if currentView === "chat"}
        {#if !$currentConversation}
          <!-- 欢迎界面 -->
          <div class="welcome-section">
            <div class="text-center max-w-2xl mx-auto">
              <div class="mb-8">
                <div
                  class="w-20 h-20 mx-auto bg-blue-500 rounded-full flex items-center justify-center mb-6"
                >
                  <svg
                    class="w-10 h-10 text-white"
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
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-50 mb-4">
                  欢迎使用 Chat Box
                </h1>
                <p class="text-lg text-gray-600 dark:text-gray-300 mb-8">
                  基于 Rust + SvelteKit 的现代化 AI 聊天助手
                </p>
              </div>

              <!-- 快速操作 -->
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
                {#each quickActions as action}
                  <button on:click={() => useQuickAction(action)} class="quick-action-card">
                    <div class="flex items-start space-x-3">
                      <span class="text-2xl">{action.icon}</span>
                      <div class="text-left">
                        <h3
                          class="font-medium text-gray-900 dark:text-gray-50 group-hover:text-blue-600 dark:group-hover:text-blue-400"
                        >
                          {action.title}
                        </h3>
                        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                          {action.description}
                        </p>
                      </div>
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {:else}
          <!-- 聊天消息 -->
          <div class="chat-messages" bind:this={messagesContainer}>
            <div class="max-w-4xl mx-auto space-y-4">
              {#each $messages as message (message.id)}
                <div
                  class="message {message.sender === 'user'
                    ? 'message-user'
                    : message.sender === 'bot'
                      ? 'message-assistant'
                      : 'message-assistant'}"
                >
                  <div class="message-bubble">
                    <div class="message-content">{message.content}</div>
                  </div>
                  <div class="message-time">
                    {formatTime(message.timestamp)}
                  </div>
                </div>
              {/each}

              {#if $isTyping}
                <div class="message message-assistant">
                  <div class="message-bubble">
                    <div class="flex items-center space-x-2">
                      <div class="flex space-x-1">
                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
                        <div
                          class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                          style="animation-delay: 0.1s"
                        ></div>
                        <div
                          class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                          style="animation-delay: 0.2s"
                        ></div>
                      </div>
                      <span class="text-sm text-gray-500 dark:text-gray-400">正在思考...</span>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- 输入区域 -->
        <div class="input-area">
          <div class="max-w-4xl mx-auto">
            <div class="flex items-end space-x-4">
              <div class="flex-1">
                <textarea
                  bind:value={messageInput}
                  on:keypress={handleKeyPress}
                  placeholder="输入您的消息... (Enter 发送，Shift+Enter 换行)"
                  class="input-textarea"
                  rows="1"
                ></textarea>
              </div>
              <button
                on:click={sendMessage}
                disabled={!messageInput.trim() ||
                  $isLoading ||
                  !$isConnected ||
                  !$currentConversation}
                class="send-button"
                aria-label="发送消息"
              >
                {#if $isLoading}
                  <div
                    class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                  ></div>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                    />
                  </svg>
                {/if}
                <span class="hidden sm:inline">发送</span>
              </button>
            </div>
          </div>
        </div>
      {:else if currentView === "conversations"}
        <!-- 对话列表视图 -->
        <div class="view-container">
          <h1 class="view-title">对话管理</h1>
          <p class="view-description">管理您的所有对话记录</p>

          <div class="space-y-4">
            {#if $conversations.length === 0}
              <div class="text-center py-12">
                <p class="text-gray-500 dark:text-gray-400">还没有对话记录</p>
                <button
                  on:click={createNewConversation}
                  class="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                  disabled={!$isConnected}
                  aria-label="创建第一个对话"
                >
                  创建第一个对话
                </button>
              </div>
            {:else}
              {#each $conversations as conversation}
                <div class="conversation-card">
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <h3 class="font-medium text-gray-900 dark:text-gray-50">
                        {conversation.title}
                      </h3>
                      <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                        {conversation.last_message}
                      </p>
                      <p class="text-xs text-gray-500 dark:text-gray-500 mt-2">
                        {formatTime(conversation.timestamp)}
                      </p>
                    </div>
                    <div class="flex space-x-2">
                      <button
                        on:click={() => selectConversation(conversation.id)}
                        class="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
                        disabled={!$isConnected}
                        aria-label="打开对话 {conversation.title}"
                      >
                        打开
                      </button>
                      <button
                        on:click={() => chatStore.deleteConversation(conversation.id)}
                        class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
                        disabled={!$isConnected}
                        aria-label="删除对话 {conversation.title}"
                      >
                        删除
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {:else if ["ai-model", "ai-behavior", "voice", "ui", "database"].includes(currentView)}
        <!-- 设置视图 -->
        <div class="view-container">
          <h1 class="view-title">设置</h1>
          <p class="view-description">配置应用参数 - {currentView}</p>

          {#if $config}
            <div class="space-y-6">
              <!-- 显示配置信息 -->
              <div class="bg-white dark:bg-gray-800 p-4 rounded-lg border">
                <h3 class="font-medium mb-2">当前配置</h3>
                <pre class="text-sm text-gray-600 dark:text-gray-400 overflow-auto">{JSON.stringify(
                    $config,
                    null,
                    2
                  )}</pre>
              </div>

              <!-- 健康状态 -->
              {#if $healthStatus}
                <div class="bg-white dark:bg-gray-800 p-4 rounded-lg border">
                  <h3 class="font-medium mb-2">系统状态</h3>
                  <div class="grid grid-cols-2 gap-4 text-sm">
                    <div class="flex items-center space-x-2">
                      <div
                        class="w-2 h-2 rounded-full {$healthStatus.database
                          ? 'bg-green-500'
                          : 'bg-red-500'}"
                      ></div>
                      <span>数据库: {$healthStatus.database ? "正常" : "异常"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                      <div
                        class="w-2 h-2 rounded-full {$healthStatus.llm
                          ? 'bg-green-500'
                          : 'bg-red-500'}"
                      ></div>
                      <span>AI模型: {$healthStatus.llm ? "正常" : "异常"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                      <div
                        class="w-2 h-2 rounded-full {$healthStatus.voice
                          ? 'bg-green-500'
                          : 'bg-red-500'}"
                      ></div>
                      <span>语音: {$healthStatus.voice ? "正常" : "异常"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                      <div
                        class="w-2 h-2 rounded-full {$healthStatus.overall
                          ? 'bg-green-500'
                          : 'bg-red-500'}"
                      ></div>
                      <span>整体: {$healthStatus.overall ? "正常" : "异常"}</span>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          {:else}
            <div class="text-center py-8">
              <div
                class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"
              ></div>
              <p class="text-gray-500 dark:text-gray-400">正在加载配置...</p>
            </div>
          {/if}
        </div>
      {:else if currentView === "guide"}
        <!-- 帮助视图 -->
        <div class="view-container">
          <h1 class="view-title">使用指南</h1>
          <div class="help-card">
            <h3>基础功能</h3>
            <p>本应用是一个基于Tauri和大语言模型的聊天应用，支持AI对话、语音交互等功能。</p>
            <h4>主要特性：</h4>
            <ul>
              <li>创建并管理多个会话</li>
              <li>与AI进行文本对话</li>
              <li>语音输入功能</li>
              <li>丰富的配置选项</li>
            </ul>
          </div>
        </div>
      {:else if currentView === "about"}
        <!-- 关于视图 -->
        <div class="view-container">
          <h1 class="view-title">关于</h1>
          <div class="about-card">
            <h3>聊天应用</h3>
            <p>版本: 0.1.0</p>
            <p>基于Tauri、SvelteKit和AI模型实现的桌面聊天应用</p>
          </div>
        </div>
      {/if}
    </main>
  </div>
</div>

<style>
  .app-container {
    height: 100vh;
    width: 100vw;
    display: flex;
    overflow: hidden;
    background: #f9fafb;
  }

  :global(.dark) .app-container {
    background: #111827;
  }

  .main-container {
    flex: 1;
    display: flex;
    overflow: hidden;
    height: 100vh;
  }

  .sidebar {
    height: 100%;
    background: #ffffff;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  :global(.dark) .sidebar {
    background: #1f2937;
    border-right: 1px solid #374151;
  }

  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid #e5e7eb;
  }

  :global(.dark) .sidebar-header {
    border-bottom: 1px solid #374151;
  }

  .conversation-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .conversation-item {
    width: 100%;
    padding: 12px;
    margin: 2px 0;
    border: none;
    background: transparent;
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .conversation-item:hover {
    background: #f3f4f6;
  }

  .conversation-item.active {
    background: #3b82f6;
  }

  :global(.dark) .conversation-item:hover {
    background: #374151;
  }

  :global(.dark) .conversation-item.active {
    background: #3b82f6;
  }

  .conversation-title {
    font-size: 14px;
    font-weight: 500;
    color: #111827;
    margin-bottom: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conversation-item.active .conversation-title {
    color: #ffffff;
  }

  :global(.dark) .conversation-title {
    color: #f9fafb;
  }

  .conversation-preview {
    font-size: 12px;
    color: #6b7280;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conversation-item.active .conversation-preview {
    color: rgba(255, 255, 255, 0.8);
  }

  :global(.dark) .conversation-preview {
    color: #9ca3af;
  }

  .conversation-time {
    font-size: 11px;
    color: #9ca3af;
    flex-shrink: 0;
    margin-left: 8px;
  }

  .conversation-item.active .conversation-time {
    color: rgba(255, 255, 255, 0.7);
  }

  .main-content {
    flex: 1;
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: #ffffff;
  }

  :global(.dark) .main-content {
    background: #111827;
  }

  .welcome-section {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 32px;
  }

  .quick-action-card {
    padding: 16px;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    transition: all 0.2s ease;
    text-align: left;
    cursor: pointer;
    group: true;
  }

  .quick-action-card:hover {
    border-color: #3b82f6;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07);
  }

  :global(.dark) .quick-action-card {
    background: #1f2937;
    border: 1px solid #374151;
  }

  :global(.dark) .quick-action-card:hover {
    border-color: #3b82f6;
  }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .message {
    display: flex;
    margin-bottom: 16px;
  }

  .message-user {
    justify-content: flex-end;
  }

  .message-assistant {
    justify-content: flex-start;
  }

  .message-bubble {
    max-width: 70%;
    padding: 12px 16px;
    border-radius: 12px;
    word-wrap: break-word;
  }

  .message-user .message-bubble {
    background: #3b82f6;
    color: #ffffff;
  }

  .message-assistant .message-bubble {
    background: #f3f4f6;
    color: #111827;
    border: 1px solid #e5e7eb;
  }

  :global(.dark) .message-assistant .message-bubble {
    background: #1f2937;
    color: #f9fafb;
    border: 1px solid #374151;
  }

  .message-content {
    white-space: pre-wrap;
  }

  .message-time {
    font-size: 11px;
    color: #9ca3af;
    margin-top: 4px;
    align-self: flex-end;
  }

  .message-user .message-time {
    text-align: right;
  }

  .input-area {
    border-top: 1px solid #e5e7eb;
    background: #ffffff;
    padding: 16px;
  }

  :global(.dark) .input-area {
    border-top: 1px solid #374151;
    background: #1f2937;
  }

  .input-textarea {
    width: 100%;
    padding: 12px 16px;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    resize: none;
    background: #ffffff;
    color: #111827;
    min-height: 48px;
    max-height: 120px;
    transition: all 0.2s ease;
  }

  .input-textarea:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
  }

  :global(.dark) .input-textarea {
    background: #374151;
    border: 1px solid #4b5563;
    color: #f9fafb;
  }

  .input-textarea::placeholder {
    color: #6b7280;
  }

  :global(.dark) .input-textarea::placeholder {
    color: #9ca3af;
  }

  .send-button {
    padding: 12px 24px;
    background: #3b82f6;
    color: #ffffff;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .send-button:hover:not(:disabled) {
    background: #2563eb;
  }

  .send-button:disabled {
    background: #d1d5db;
    cursor: not-allowed;
  }

  :global(.dark) .send-button:disabled {
    background: #4b5563;
  }

  .view-container {
    padding: 32px;
    flex: 1;
    overflow-y: auto;
  }

  .view-title {
    font-size: 28px;
    font-weight: 700;
    color: #111827;
    margin-bottom: 8px;
  }

  :global(.dark) .view-title {
    color: #f9fafb;
  }

  .view-description {
    font-size: 16px;
    color: #6b7280;
    margin-bottom: 24px;
  }

  :global(.dark) .view-description {
    color: #9ca3af;
  }

  .help-card,
  .about-card {
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 24px;
    margin-bottom: 16px;
  }

  :global(.dark) .help-card,
  :global(.dark) .about-card {
    background: #1f2937;
    border: 1px solid #374151;
  }

  .help-card h3,
  .about-card h3 {
    color: #111827;
    margin-bottom: 16px;
  }

  :global(.dark) .help-card h3,
  :global(.dark) .about-card h3 {
    color: #f9fafb;
  }

  .help-card h4 {
    color: #374151;
    margin: 16px 0 8px 0;
  }

  :global(.dark) .help-card h4 {
    color: #d1d5db;
  }

  .help-card p,
  .about-card p {
    color: #6b7280;
    line-height: 1.6;
  }

  :global(.dark) .help-card p,
  :global(.dark) .about-card p {
    color: #9ca3af;
  }

  .help-card ul {
    color: #6b7280;
    padding-left: 20px;
  }

  :global(.dark) .help-card ul {
    color: #9ca3af;
  }

  /* 响应式设计 */
  @media (max-width: 768px) {
    .sidebar {
      display: none;
    }

    .welcome-section {
      padding: 16px;
    }

    .quick-action-card {
      padding: 12px;
    }

    .view-container {
      padding: 16px;
    }
  }

  /* 动画 */
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

  /* 滚动条样式 */
  .conversation-list::-webkit-scrollbar,
  .chat-messages::-webkit-scrollbar {
    width: 6px;
  }

  .conversation-list::-webkit-scrollbar-track,
  .chat-messages::-webkit-scrollbar-track {
    background: transparent;
  }

  .conversation-list::-webkit-scrollbar-thumb,
  .chat-messages::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  :global(.dark) .conversation-list::-webkit-scrollbar-thumb,
  :global(.dark) .chat-messages::-webkit-scrollbar-thumb {
    background: #4b5563;
  }
</style>
