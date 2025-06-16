<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { chatStore, conversations } from "$lib/stores/chatStore";
  import type { Conversation } from "$lib/types";

  let searchQuery = "";
  let isLoading = false;
  let selectedConversations: Set<number> = new Set();
  let showDeleteConfirm = false;

  // Mock conversations data
  let mockConversations: Conversation[] = [
    {
      id: 1,
      title: "AI 技术讨论",
      last_message: "人工智能的发展前景如何？",
      timestamp: Date.now() - 3600000, // 1 hour ago
    },
    {
      id: 2,
      title: "编程学习",
      last_message: "请推荐一些学习编程的资源",
      timestamp: Date.now() - 7200000, // 2 hours ago
    },
    {
      id: 3,
      title: "技术问答",
      last_message: "React 和 Vue 的区别是什么？",
      timestamp: Date.now() - 86400000, // 1 day ago
    },
    {
      id: 4,
      title: "项目开发",
      last_message: "如何优化网站性能？",
      timestamp: Date.now() - 172800000, // 2 days ago
    },
    {
      id: 5,
      title: "数据库设计",
      last_message: "MySQL 索引的最佳实践",
      timestamp: Date.now() - 259200000, // 3 days ago
    },
    {
      id: 6,
      title: "算法学习",
      last_message: "二分查找的实现原理",
      timestamp: Date.now() - 604800000, // 1 week ago
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

  onMount(async () => {
    await loadConversations();
  });

  async function loadConversations() {
    isLoading = true;
    try {
      // 这里应该从实际的数据源加载对话
      // await chatStore.loadConversations();
      // 目前使用模拟数据
      await new Promise((resolve) => setTimeout(resolve, 500));
    } catch (error) {
      console.error("加载对话失败:", error);
      showToast("加载对话失败", "error");
    } finally {
      isLoading = false;
    }
  }

  function formatTime(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;

    if (diff < 60000) return "刚刚";
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
    if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`;

    return new Date(timestamp).toLocaleDateString("zh-CN", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }

  function formatTimeDetailed(timestamp: number): string {
    return new Date(timestamp).toLocaleString("zh-CN", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  async function openConversation(conversation: Conversation) {
    try {
      // 这里应该选择对话并跳转到聊天页面
      // await chatStore.selectConversation(conversation.id);
      showToast(`打开对话: ${conversation.title}`, "info");
      goto("/");
    } catch (error) {
      console.error("打开对话失败:", error);
      showToast("打开对话失败", "error");
    }
  }

  async function createNewConversation() {
    try {
      const title = `新对话 ${new Date().toLocaleString("zh-CN")}`;
      // await chatStore.createConversation(title);
      showToast("创建新对话成功", "success");
      goto("/");
    } catch (error) {
      console.error("创建对话失败:", error);
      showToast("创建对话失败", "error");
    }
  }

  function toggleConversationSelection(conversationId: number) {
    if (selectedConversations.has(conversationId)) {
      selectedConversations.delete(conversationId);
    } else {
      selectedConversations.add(conversationId);
    }
    selectedConversations = selectedConversations; // 触发响应性更新
  }

  function selectAllConversations() {
    if (selectedConversations.size === filteredConversations.length) {
      selectedConversations.clear();
    } else {
      selectedConversations = new Set(filteredConversations.map((c) => c.id));
    }
    selectedConversations = selectedConversations;
  }

  async function deleteSelectedConversations() {
    if (selectedConversations.size === 0) return;

    try {
      for (const id of selectedConversations) {
        // await chatStore.deleteConversation(id);
        mockConversations = mockConversations.filter((c) => c.id !== id);
      }
      selectedConversations.clear();
      selectedConversations = selectedConversations;
      showDeleteConfirm = false;
      showToast(`删除了 ${selectedConversations.size} 个对话`, "success");
    } catch (error) {
      console.error("删除对话失败:", error);
      showToast("删除对话失败", "error");
    }
  }

  async function exportConversations() {
    try {
      const data = {
        exportTime: new Date().toISOString(),
        conversations: filteredConversations,
      };

      const blob = new Blob([JSON.stringify(data, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `conversations-export-${new Date().toISOString().split("T")[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);

      showToast("对话导出成功", "success");
    } catch (error) {
      console.error("导出失败:", error);
      showToast("导出失败", "error");
    }
  }

  // 过滤对话
  $: filteredConversations = mockConversations.filter((conversation) => {
    if (!searchQuery.trim()) return true;
    const query = searchQuery.toLowerCase();
    return (
      conversation.title.toLowerCase().includes(query) ||
      conversation.last_message.toLowerCase().includes(query)
    );
  });

  // 按时间排序
  $: sortedConversations = filteredConversations.sort((a, b) => b.timestamp - a.timestamp);
</script>

<svelte:head>
  <title>对话列表 - Chat Box</title>
</svelte:head>

<div class="conversations-page h-full flex flex-col bg-gray-50 dark:bg-gray-900">
  <!-- 页面头部 -->
  <div class="page-header bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
    <div class="max-w-6xl mx-auto">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-50">对话列表</h1>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            共 {sortedConversations.length} 个对话
            {#if selectedConversations.size > 0}
              · 已选择 {selectedConversations.size} 个
            {/if}
          </p>
        </div>

        <div class="flex items-center space-x-3">
          <button
            on:click={createNewConversation}
            class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors duration-200 flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            <span class="hidden sm:inline">新建对话</span>
          </button>

          {#if selectedConversations.size > 0}
            <button
              on:click={() => (showDeleteConfirm = true)}
              class="px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors duration-200 flex items-center space-x-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                />
              </svg>
              <span class="hidden sm:inline">删除</span>
            </button>
          {/if}

          <button
            on:click={exportConversations}
            class="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg transition-colors duration-200 flex items-center space-x-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            <span class="hidden sm:inline">导出</span>
          </button>
        </div>
      </div>

      <!-- 搜索和批量操作 -->
      <div class="flex items-center space-x-4">
        <div class="flex-1">
          <div class="relative">
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="搜索对话..."
              class="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-50 placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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

        {#if sortedConversations.length > 0}
          <button
            on:click={selectAllConversations}
            class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200"
          >
            {selectedConversations.size === filteredConversations.length ? "取消全选" : "全选"}
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- 对话列表 -->
  <div class="conversations-list flex-1 overflow-y-auto p-4">
    <div class="max-w-6xl mx-auto">
      {#if isLoading}
        <div class="flex items-center justify-center py-12">
          <div class="flex flex-col items-center space-y-4">
            <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
            <p class="text-gray-600 dark:text-gray-400">加载中...</p>
          </div>
        </div>
      {:else if sortedConversations.length === 0}
        <div class="text-center py-12">
          <div class="w-16 h-16 mx-auto bg-gray-200 dark:bg-gray-700 rounded-full flex items-center justify-center mb-4">
            <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
          </div>
          <h3 class="text-lg font-medium text-gray-900 dark:text-gray-50 mb-2">
            {searchQuery ? "未找到匹配的对话" : "还没有对话"}
          </h3>
          <p class="text-gray-600 dark:text-gray-400 mb-4">
            {searchQuery ? "尝试使用不同的关键词搜索" : "创建您的第一个对话开始聊天"}
          </p>
          {#if !searchQuery}
            <button
              on:click={createNewConversation}
              class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors duration-200"
            >
              创建新对话
            </button>
          {/if}
        </div>
      {:else}
        <div class="grid gap-4">
          {#each sortedConversations as conversation (conversation.id)}
            <div
              class="conversation-item bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 transition-all duration-200 cursor-pointer group"
              class:ring-2={selectedConversations.has(conversation.id)}
              class:ring-blue-500={selectedConversations.has(conversation.id)}
            >
              <div class="p-4">
                <div class="flex items-start space-x-3">
                  <div class="flex-shrink-0 pt-1">
                    <input
                      type="checkbox"
                      checked={selectedConversations.has(conversation.id)}
                      on:change={() => toggleConversationSelection(conversation.id)}
                      class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                    />
                  </div>

                  <div
                    class="flex-1 min-w-0"
                    role="button"
                    tabindex="0"
                    on:click={() => openConversation(conversation)}
                    on:keypress={(e) => e.key === "Enter" && openConversation(conversation)}
                  >
                    <div class="flex items-center justify-between mb-2">
                      <h3
                        class="text-base font-medium text-gray-900 dark:text-gray-50 truncate group-hover:text-blue-600 dark:group-hover:text-blue-400"
                      >
                        {conversation.title}
                      </h3>
                      <time
                        class="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0 ml-2"
                        title={formatTimeDetailed(conversation.timestamp)}
                      >
                        {formatTime(conversation.timestamp)}
                      </time>
                    </div>
                    <p class="text-sm text-gray-600 dark:text-gray-300 line-clamp-2">
                      {conversation.last_message}
                    </p>
                  </div>

                  <div class="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <div class="flex items-center space-x-1">
                      <button
                        on:click|stopPropagation={() => openConversation(conversation)}
                        class="p-1 text-gray-400 hover:text-blue-500 transition-colors duration-200"
                        title="打开对话"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                          />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- 删除确认对话框 -->
{#if showDeleteConfirm}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
      <h3 class="text-lg font-medium text-gray-900 dark:text-gray-50 mb-4">删除对话</h3>
      <p class="text-gray-600 dark:text-gray-400 mb-6">
        确定要删除选中的 {selectedConversations.size} 个对话吗？此操作无法撤销。
      </p>
      <div class="flex space-x-3">
        <button
          on:click={() => (showDeleteConfirm = false)}
          class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200"
        >
          取消
        </button>
        <button
          on:click={deleteSelectedConversations}
          class="flex-1 px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors duration-200"
        >
          删除
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .conversations-page {
    height: calc(100vh - 120px);
  }

  .conversations-list {
    scrollbar-width: thin;
    scrollbar-color: rgb(156 163 175) transparent;
  }

  .conversations-list::-webkit-scrollbar {
    width: 6px;
  }

  .conversations-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .conversations-list::-webkit-scrollbar-thumb {
    background-color: rgb(156 163 175);
    border-radius: 3px;
  }

  .conversations-list::-webkit-scrollbar-thumb:hover {
    background-color: rgb(107 114 128);
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
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

  .animate-slide-down {
    animation: slide-down 0.3s ease-out;
  }

  @media (max-width: 640px) {
    .conversations-page {
      height: calc(100vh - 100px);
    }

    .page-header {
      padding: 1rem;
    }

    .conversations-list {
      padding: 1rem;
    }
  }
</style>
