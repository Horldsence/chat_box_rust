<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tick } from "svelte";
  import { fade, fly, slide } from "svelte/transition";
  import {
    chatStore,
    conversations,
    messages,
    currentConversation,
    isLoading,
    isTyping,
  } from "$lib/stores/chatStore";
  import { themeStore, isDark } from "$lib/stores/themeStore";
  import { tauriService } from "$lib/services/TauriService";
  import { errorService } from "$lib/services/ErrorService";
  import Button from "$lib/components/ui/Button.svelte";
  import Message from "$lib/components/ui/Message.svelte";
  import type { Conversation } from "$lib/types";

  // State
  let messageInput = "";
  let messagesContainer: HTMLElement;
  let inputTextarea: HTMLTextAreaElement;
  let showSidebar = true;
  let isInitialized = false;

  // Quick actions
  const quickActions = [
    { text: "你好，我想了解一下AI技术", label: "AI介绍", icon: "🤖" },
    { text: "帮我写一个简单的代码示例", label: "代码帮助", icon: "💻" },
    { text: "解释一下最新的技术趋势", label: "技术趋势", icon: "🚀" },
    { text: "推荐一些学习资源", label: "学习资源", icon: "📚" },
  ];

  // Lifecycle
  onMount(async () => {
    try {
      await tauriService.init();
      await chatStore.init();
      isInitialized = true;
      await tick();
      scrollToBottom();
    } catch (error) {
      console.error("Failed to initialize:", error);
      errorService.handleError(error, "initialization");
    }
  });

  onDestroy(() => {
    chatStore.destroy();
  });

  // Reactive updates
  $: if ($messages.length > 0) {
    tick().then(() => scrollToBottom());
  }

  // Methods
  async function sendMessage() {
    const content = messageInput.trim();
    if (!content || $isLoading) return;

    try {
      messageInput = "";
      adjustTextareaHeight();
      await chatStore.sendMessage(content);
    } catch (error) {
      errorService.handleError(error, "send-message");
    }
  }

  async function createNewConversation() {
    try {
      await chatStore.createConversation("新对话");
    } catch (error) {
      errorService.handleError(error, "create-conversation");
    }
  }

  async function selectConversation(conversationId: number) {
    try {
      await chatStore.selectConversation(conversationId);
    } catch (error) {
      errorService.handleError(error, "select-conversation");
    }
  }

  async function deleteConversation(conversationId: number, event: Event) {
    event.stopPropagation();
    if (!confirm("确定要删除这个对话吗？")) return;

    try {
      await chatStore.deleteConversation(conversationId);
    } catch (error) {
      errorService.handleError(error, "delete-conversation");
    }
  }

  function scrollToBottom() {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  function adjustTextareaHeight() {
    if (inputTextarea) {
      inputTextarea.style.height = "auto";
      inputTextarea.style.height = Math.min(inputTextarea.scrollHeight, 120) + "px";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }

  function handleQuickAction(text: string) {
    messageInput = text;
    tick().then(() => {
      adjustTextareaHeight();
      sendMessage();
    });
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString("zh-CN", {
      month: "short",
      day: "numeric",
    });
  }

  async function closeWindow() {
    try {
      await tauriService.closeWindow();
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  }
</script>

<svelte:head>
  <title>Chat Box - AI 聊天助手</title>
</svelte:head>

{#if !isInitialized}
  <div class="loading-screen">
    <div class="loading-content">
      <div class="loading-spinner"></div>
      <p>正在启动聊天助手...</p>
    </div>
  </div>
{:else}
  <div class="app" class:sidebar-collapsed={!showSidebar}>
    <!-- Sidebar -->
    <aside class="sidebar" class:collapsed={!showSidebar}>
      <div class="sidebar-header">
        <h2>对话</h2>
        <Button variant="primary" size="sm" on:click={createNewConversation} title="新建对话">
          <span>+</span>
        </Button>
      </div>

      <div class="conversations">
        {#each $conversations as conversation (conversation.id)}
          <div
            class="conversation"
            class:active={$currentConversation?.id === conversation.id}
            on:click={() => selectConversation(conversation.id)}
            on:keydown={(e) => e.key === "Enter" && selectConversation(conversation.id)}
            role="button"
            tabindex="0"
            transition:slide={{ duration: 200 }}
          >
            <div class="conversation-content">
              <div class="conversation-title">
                {conversation.title}
              </div>
              <div class="conversation-date">
                {formatDate(conversation.timestamp)}
              </div>
            </div>
            <button
              class="conversation-delete"
              on:click={(e) => deleteConversation(conversation.id, e)}
              title="删除对话"
            >
              ×
            </button>
          </div>
        {/each}
      </div>

      <div class="sidebar-footer">
        <Button variant="ghost" size="sm" on:click={() => themeStore.toggle()} title="切换主题">
          {$isDark ? "🌙" : "☀️"}
        </Button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main">
      <!-- Header -->
      <header class="header">
        <div class="header-left">
          {#if !showSidebar}
            <Button
              variant="ghost"
              size="sm"
              on:click={() => (showSidebar = true)}
              title="显示侧边栏"
            >
              ☰
            </Button>
          {/if}
          <h1>
            {$currentConversation?.title || "AI 聊天助手"}
          </h1>
        </div>

        <div class="header-right">
          {#if showSidebar}
            <Button
              variant="ghost"
              size="sm"
              on:click={() => (showSidebar = false)}
              title="隐藏侧边栏"
            >
              ←
            </Button>
          {/if}
          <Button variant="ghost" size="sm" on:click={closeWindow} title="关闭">×</Button>
        </div>
      </header>

      <!-- Messages -->
      <div class="messages" bind:this={messagesContainer}>
        {#if $messages.length === 0}
          <div class="welcome" transition:fade>
            <div class="welcome-content">
              <h2>👋 欢迎使用 Chat Box</h2>
              <p>选择一个话题开始对话，或者直接输入您的问题</p>

              <div class="quick-actions">
                {#each quickActions as action}
                  <button
                    class="quick-action"
                    on:click={() => handleQuickAction(action.text)}
                    transition:slide={{ delay: 100 }}
                  >
                    <span class="action-icon">{action.icon}</span>
                    <span class="action-label">{action.label}</span>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {:else}
          {#each $messages as message (message.id)}
            <Message {message} />
          {/each}

          {#if $isTyping}
            <Message
              message={{
                id: 0,
                conversation_id: $currentConversation?.id || 0,
                content: "",
                sender: "assistant",
                timestamp: Date.now(),
              }}
              isTyping={true}
            />
          {/if}
        {/if}
      </div>

      <!-- Input -->
      <div class="input-area">
        <div class="input-wrapper">
          <textarea
            bind:this={inputTextarea}
            bind:value={messageInput}
            placeholder="输入您的消息... (Enter发送，Shift+Enter换行)"
            rows="1"
            disabled={$isLoading}
            on:keydown={handleKeydown}
            on:input={adjustTextareaHeight}
          ></textarea>

          <Button
            variant="primary"
            size="sm"
            disabled={!messageInput.trim() || $isLoading}
            loading={$isLoading}
            on:click={sendMessage}
            title="发送"
          >
            {#if $isLoading}
              发送中...
            {:else}
              发送
            {/if}
          </Button>
        </div>
      </div>
    </main>
  </div>
{/if}

<style>
  .loading-screen {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
  }

  .loading-content {
    text-align: center;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top: 3px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  .app {
    height: 100vh;
    display: flex;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .sidebar {
    width: 280px;
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border-right: 1px solid rgba(255, 255, 255, 0.2);
    display: flex;
    flex-direction: column;
    transition: all 0.3s ease;
  }

  .sidebar.collapsed {
    width: 0;
    min-width: 0;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 1.5rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sidebar-header h2 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .conversations {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .conversation {
    display: flex;
    align-items: center;
    padding: 0.75rem;
    margin-bottom: 0.25rem;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    color: rgba(255, 255, 255, 0.8);
  }

  .conversation:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .conversation.active {
    background: rgba(255, 255, 255, 0.2);
    color: white;
  }

  .conversation-content {
    flex: 1;
    min-width: 0;
  }

  .conversation-title {
    font-weight: 500;
    margin-bottom: 0.25rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .conversation-date {
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .conversation-delete {
    opacity: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: all 0.2s;
  }

  .conversation:hover .conversation-delete {
    opacity: 1;
  }

  .conversation-delete:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .header {
    padding: 1rem 1.5rem;
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: white;
  }

  .header-left,
  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .welcome {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: white;
  }

  .welcome-content h2 {
    margin: 0 0 1rem;
    font-size: 2rem;
    font-weight: 600;
  }

  .welcome-content p {
    margin: 0 0 2rem;
    font-size: 1.1rem;
    opacity: 0.9;
  }

  .quick-actions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    max-width: 600px;
  }

  .quick-action {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.75rem;
    color: white;
    cursor: pointer;
    transition: all 0.3s;
    backdrop-filter: blur(10px);
  }

  .quick-action:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: translateY(-2px);
  }

  .action-icon {
    font-size: 1.5rem;
  }

  .action-label {
    font-weight: 500;
  }

  .input-area {
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border-top: 1px solid rgba(255, 255, 255, 0.2);
  }

  .input-wrapper {
    display: flex;
    gap: 0.75rem;
    align-items: flex-end;
    background: rgba(255, 255, 255, 0.9);
    border-radius: 1rem;
    padding: 0.75rem;
  }

  textarea {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    resize: none;
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.5;
    min-height: 1.5rem;
    max-height: 120px;
  }

  textarea::placeholder {
    color: #6b7280;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Responsive */
  @media (max-width: 768px) {
    .sidebar {
      position: absolute;
      top: 0;
      left: 0;
      height: 100%;
      z-index: 100;
    }

    .app.sidebar-collapsed .main {
      margin-left: 0;
    }

    .quick-actions {
      grid-template-columns: 1fr;
    }
  }

  /* Dark theme */
  :global([data-theme="dark"]) .input-wrapper {
    background: rgba(0, 0, 0, 0.3);
    color: white;
  }

  :global([data-theme="dark"]) textarea {
    color: white;
  }

  :global([data-theme="dark"]) textarea::placeholder {
    color: #9ca3af;
  }

  /* Scrollbar */
  .conversations::-webkit-scrollbar,
  .messages::-webkit-scrollbar {
    width: 6px;
  }

  .conversations::-webkit-scrollbar-track,
  .messages::-webkit-scrollbar-track {
    background: transparent;
  }

  .conversations::-webkit-scrollbar-thumb,
  .messages::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
  }

  .conversations::-webkit-scrollbar-thumb:hover,
  .messages::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
