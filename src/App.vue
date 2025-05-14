<template>
  <el-container class="app-container">
    <!-- 主菜单 -->
    <CollapsePanel @select="handleMenuSelect" />

    <el-container>
      <!-- 对话列表侧边栏 -->
      <el-aside
        :width="sidebarWidth"
        class="sidebar"
        v-show="currentView === 'chat'"
      >
        <ConversationList
          :conversations="conversations"
          :current-conversation-id="
            currentConversation ? currentConversation.id : null
          "
          @select="selectConversation"
          @create="createNewConversation"
          @delete="deleteConversation"
        />
      </el-aside>

      <!-- 主内容区域 -->
      <el-main class="main-content">
        <!-- 聊天视图 -->
        <MessagePanel
          v-if="currentView === 'chat'"
          :messages="currentMessages"
          :conversation="currentConversation"
          :is-loading="isLoading"
          @send-message="sendMessage"
        />

        <!-- 设置视图 -->
        <SettingsView
          v-else-if="
            ['ai-model', 'ai-behavior', 'voice', 'ui'].includes(currentView)
          "
          :settingType="currentView"
        />

        <!-- 帮助视图 -->
        <div v-else-if="currentView === 'guide'" class="help-view">
          <h2>使用指南</h2>
          <el-card class="help-card">
            <template #header>
              <div class="card-header">
                <h3>基础功能</h3>
              </div>
            </template>
            <p>
              本应用是一个基于Tauri和大语言模型的聊天应用，支持AI对话、语音交互等功能。
            </p>
            <h4>主要特性：</h4>
            <ul>
              <li>创建并管理多个会话</li>
              <li>与AI进行文本对话</li>
              <li>语音输入功能</li>
              <li>丰富的配置选项</li>
            </ul>
          </el-card>
        </div>

        <!-- 关于视图 -->
        <div v-else-if="currentView === 'about'" class="about-view">
          <h2>关于</h2>
          <el-card>
            <h3>聊天应用</h3>
            <p>版本: 0.1.0</p>
            <p>基于Tauri、Vue3和Ollama实现的桌面AI聊天应用</p>
          </el-card>
        </div>
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ConversationList from "./components/chat/ConversationList.vue";
import MessagePanel from "./components/chat/MessagePanel.vue";
import CollapsePanel from "./components/menu/CollapsePanel.vue";
import SettingsView from "./components/settings/SettingsView.vue";
import type { Message, Conversation, MessageChunk } from "./types";

// 应用配置接口
interface AppConfig {
  ui: {
    sidebar_width: string;
  };
  // 其他配置属性
}

// 状态
const conversations = ref<Conversation[]>([]);
const currentConversation = ref<Conversation | null>(null);
const allMessages = ref<Message[]>([]);
const isLoading = ref(false);
const sidebarWidth = ref("280px");
const currentView = ref("chat");

// 菜单选择处理
const handleMenuSelect = (menuItem: string) => {
  console.log("选中菜单项:", menuItem);
  currentView.value = menuItem;

  // 如果是切换到聊天视图，确保有一个被选中的对话
  if (
    menuItem === "chat" &&
    !currentConversation.value &&
    conversations.value.length > 0
  ) {
    selectConversation(conversations.value[0].id);
  }
};

// 当前对话的消息
const currentMessages = computed(() => {
  if (!currentConversation.value) return [];
  return allMessages.value
    .filter((m) => m.conversation_id === currentConversation.value?.id)
    .sort((a, b) => a.timestamp - b.timestamp);
});

// 初始化
// 加载配置
const loadConfig = async () => {
  try {
    const config = await invoke<AppConfig>("get_app_config");
    sidebarWidth.value = config.ui.sidebar_width;
  } catch (error) {
    console.error("加载配置失败:", error);
  }
};

// 监听后端消息更新
const setupMessageListeners = async () => {
  await listen<MessageChunk>("message_chunk", (event) => {
    const { conversation_id, content, is_complete } = event.payload;

    // 如果不是结束信号并且有内容，将内容追加到当前消息
    if (!is_complete && content) {
      // 查找当前消息集中最后一条机器人消息
      const messages = allMessages.value.filter(
        (m) => m.conversation_id === conversation_id
      );
      const lastBotMessage = messages
        .filter((m) => m.sender === "bot")
        .sort((a, b) => b.timestamp - a.timestamp)[0];

      if (lastBotMessage) {
        lastBotMessage.content += content;
      }
    }

    // 更新对话最后消息时间
    if (is_complete) {
      updateConversationTimestamp(conversation_id);
    }
  });
};

// 加载对话列表
const loadConversations = async () => {
  try {
    const convs = await invoke<Conversation[]>("get_conversations");
    conversations.value = convs;

    // 如果有对话但没有选中的对话，自动选择第一个
    if (convs.length > 0 && !currentConversation.value) {
      await selectConversation(convs[0].id);
    }
  } catch (error) {
    console.error("加载对话失败:", error);
  }
};

// 选择对话
const selectConversation = async (id: number) => {
  try {
    const conv = conversations.value.find((c) => c.id === id);
    if (!conv) return;

    currentConversation.value = conv;
    await loadConversationMessages(id);
  } catch (error) {
    console.error("切换对话失败:", error);
  }
};

// 加载特定对话的所有消息
const loadConversationMessages = async (conversationId: number) => {
  try {
    const messages = await invoke<Message[]>("get_conversation_messages", {
      conversationId,
    });

    // 更新消息缓存
    allMessages.value = allMessages.value.filter(
      (m) => m.conversation_id !== conversationId
    );
    allMessages.value.push(...messages);
  } catch (error) {
    console.error("加载对话消息失败:", error);
  }
};

// 发送消息
const sendMessage = async (content: string) => {
  if (!currentConversation.value || !content.trim()) return;

  const conversationId = currentConversation.value.id;
  isLoading.value = true;

  try {
    // 1. 创建用户消息并添加到UI
    const userMsg = await invoke<Message>("send_user_message", {
      content,
      conversationId,
    });

    // 2. 立即将新消息添加到UI，不等待后端响应
    allMessages.value.push(userMsg);

    // 3. 启动AI响应生成
    await invoke("generate_ai_response", {
      userMessageContent: content,
      conversationId,
    });

    // 4. 更新对话列表（更新时间戳和最新消息）
    await loadConversations();
  } catch (error) {
    console.error("发送消息失败:", error);
  } finally {
    isLoading.value = false;
  }
};

// 更新对话时间戳
const updateConversationTimestamp = (conversationId: number) => {
  const conv = conversations.value.find((c) => c.id === conversationId);
  if (conv) {
    conv.timestamp = Date.now();
    conversations.value.sort((a, b) => b.timestamp - a.timestamp);
  }
};

// 创建新对话
const createNewConversation = async (title: string) => {
  try {
    const newConv = await invoke<Conversation>("create_conversation", {
      title,
    });
    conversations.value.unshift(newConv);
    await selectConversation(newConv.id);
    // 自动切换到聊天视图
    currentView.value = "chat";
  } catch (error) {
    console.error("创建对话失败:", error);
  }
};

// 删除对话
const deleteConversation = async (id: number) => {
  try {
    await invoke("delete_conversation", { conversationId: id });
    conversations.value = conversations.value.filter((c) => c.id !== id);

    // 如果删除的是当前对话，自动选择另一个
    if (currentConversation.value?.id === id) {
      currentConversation.value =
        conversations.value.length > 0 ? conversations.value[0] : null;

      if (currentConversation.value) {
        await loadConversationMessages(currentConversation.value.id);
      }
    }
  } catch (error) {
    console.error("删除对话失败:", error);
  }
};
</script>

<style>
html,
body {
  margin: 0;
  padding: 0;
  height: 100vh;
  overflow: hidden;
}

#app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-container {
  height: 100vh;
  display: flex;
  overflow: hidden;
}

.sidebar {
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.main-content {
  height: 100%;
  overflow: hidden;
  padding: 0;
}

.help-view,
.about-view {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.help-card {
  margin-bottom: 20px;
}

/* 为ConversationList和MessagePanel设置最小高度 */
.el-aside,
.el-main {
  min-height: 400px;
}
</style>
