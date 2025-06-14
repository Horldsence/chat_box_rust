/**
 * Chat Box 状态管理 Store
 * 使用 Vue 3 Composition API + Pinia 进行状态管理
 */

import { ref, computed, reactive } from "vue";
import { defineStore } from "pinia";
import {
  chatAPI,
  type Conversation,
  type Message,
  type AppConfig,
} from "../api";

// 主应用状态 Store
export const useAppStore = defineStore("app", () => {
  // 应用配置
  const config = ref<AppConfig>({
    ai_model: {
      model_type: "ollama",
      model_name: "llama3.2",
      server_url: "http://localhost",
      server_port: "11434",
      system_prompt: "You are a helpful assistant.",
      candle_model_id: "",
      candle_revision: "main",
      candle_use_flash_attn: false,
    },
    ui: {
      theme: "light",
      language: "zh-CN",
    },
    voice: {
      enabled: true,
      model_path: "",
      timeout_seconds: 30,
    },
    database: {
      enabled: true,
      path: "",
    },
    app_behavior: {
      log_level: "info",
      default_conversation_title: "新对话",
      welcome_message: "欢迎使用聊天助手！",
      message_chunk_buffer_size: 1024,
      message_chunk_send_interval_ms: 100,
      show_error_dialogs: true,
      auto_retry_failed_init: true,
    },
  });

  // 应用状态
  const isLoading = ref(false);
  const isConnected = ref(true);
  const currentView = ref("chat");
  const sidebarCollapsed = ref(false);

  // 错误状态
  const errors = reactive({
    connection: null as string | null,
    api: null as string | null,
    voice: null as string | null,
  });

  // 计算属性
  const isDarkMode = computed(() => {
    if (config.value.ui.theme === "auto") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    return config.value.ui.theme === "dark";
  });

  // 加载应用配置
  const loadConfig = async () => {
    try {
      isLoading.value = true;
      const loadedConfig = await chatAPI.getAppConfig();
      config.value = { ...config.value, ...loadedConfig };
    } catch (error) {
      console.error("加载配置失败:", error);
      errors.api = "加载配置失败";
    } finally {
      isLoading.value = false;
    }
  };

  // 保存应用配置
  const saveConfig = async (newConfig: Partial<AppConfig>) => {
    try {
      isLoading.value = true;
      const mergedConfig = { ...config.value, ...newConfig };
      await chatAPI.saveAppConfig(mergedConfig);
      config.value = mergedConfig;
      errors.api = null;
    } catch (error) {
      console.error("保存配置失败:", error);
      errors.api = "保存配置失败";
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 重置配置
  const resetConfig = async () => {
    try {
      isLoading.value = true;
      await chatAPI.resetAppConfig();
      await loadConfig();
    } catch (error) {
      console.error("重置配置失败:", error);
      errors.api = "重置配置失败";
      throw error;
    } finally {
      isLoading.value = false;
    }
  };

  // 检查连接状态
  const checkConnection = async () => {
    try {
      isConnected.value = await chatAPI.checkConnection();
      if (isConnected.value) {
        errors.connection = null;
      }
    } catch (error) {
      isConnected.value = false;
      errors.connection = "连接失败";
    }
    return isConnected.value;
  };

  // 切换视图
  const setCurrentView = (view: string) => {
    currentView.value = view;
  };

  // 切换侧边栏
  const toggleSidebar = () => {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  };

  // 清除错误
  const clearError = (type: keyof typeof errors) => {
    errors[type] = null;
  };

  return {
    // 状态
    config,
    isLoading,
    isConnected,
    currentView,
    sidebarCollapsed,
    errors,
    // 计算属性
    isDarkMode,
    // 方法
    loadConfig,
    saveConfig,
    resetConfig,
    checkConnection,
    setCurrentView,
    toggleSidebar,
    clearError,
  };
});

// 聊天状态 Store
export const useChatStore = defineStore("chat", () => {
  // 对话和消息状态
  const conversations = ref<Conversation[]>([]);
  const currentConversation = ref<Conversation | null>(null);
  const messages = ref<Map<number, Message[]>>(new Map());
  const isTyping = ref(false);
  const isGenerating = ref(false);

  // 语音状态
  const isRecording = ref(false);
  const voiceSupported = ref(true);

  // 计算属性
  const currentMessages = computed(() => {
    if (!currentConversation.value) return [];
    return messages.value.get(currentConversation.value.id) || [];
  });

  const hasConversations = computed(() => conversations.value.length > 0);

  const canSendMessage = computed(() => {
    return !isGenerating.value && !isRecording.value;
  });

  // 加载对话列表
  const loadConversations = async () => {
    try {
      const loadedConversations = await chatAPI.getConversations();
      conversations.value = loadedConversations.sort(
        (a, b) => b.timestamp - a.timestamp,
      );

      // 如果没有当前对话但有对话列表，选择第一个
      if (!currentConversation.value && conversations.value.length > 0) {
        await selectConversation(conversations.value[0].id);
      }
    } catch (error) {
      console.error("加载对话列表失败:", error);
      throw error;
    }
  };

  // 创建新对话
  const createConversation = async (title: string) => {
    try {
      const newConversation = await chatAPI.createConversation(title);
      conversations.value.unshift(newConversation);
      await selectConversation(newConversation.id);
      return newConversation;
    } catch (error) {
      console.error("创建对话失败:", error);
      throw error;
    }
  };

  // 删除对话
  const deleteConversation = async (conversationId: number) => {
    try {
      await chatAPI.deleteConversation(conversationId);
      conversations.value = conversations.value.filter(
        (conv) => conv.id !== conversationId,
      );
      messages.value.delete(conversationId);

      // 如果删除的是当前对话
      if (currentConversation.value?.id === conversationId) {
        if (conversations.value.length > 0) {
          await selectConversation(conversations.value[0].id);
        } else {
          currentConversation.value = null;
        }
      }
    } catch (error) {
      console.error("删除对话失败:", error);
      throw error;
    }
  };

  // 选择对话
  const selectConversation = async (conversationId: number) => {
    try {
      const conversation = conversations.value.find(
        (conv) => conv.id === conversationId,
      );
      if (!conversation) {
        throw new Error("对话不存在");
      }

      currentConversation.value = conversation;
      await loadConversationMessages(conversationId);
    } catch (error) {
      console.error("选择对话失败:", error);
      throw error;
    }
  };

  // 加载对话消息
  const loadConversationMessages = async (conversationId: number) => {
    try {
      const loadedMessages =
        await chatAPI.getConversationMessages(conversationId);
      messages.value.set(conversationId, loadedMessages);
    } catch (error) {
      console.error("加载对话消息失败:", error);
      throw error;
    }
  };

  // 发送消息
  const sendMessage = async (content: string) => {
    if (!currentConversation.value || !content.trim()) {
      return;
    }

    const conversationId = currentConversation.value.id;

    try {
      isGenerating.value = true;

      // 发送用户消息
      const userMessage = await chatAPI.sendUserMessage(
        content,
        conversationId,
      );

      // 更新本地消息列表
      const currentMessages = messages.value.get(conversationId) || [];
      messages.value.set(conversationId, [...currentMessages, userMessage]);

      // 创建空的 AI 消息占位符
      const aiMessage: Message = {
        id: Date.now(), // 临时 ID
        conversation_id: conversationId,
        content: "",
        sender: "bot",
        timestamp: Date.now(),
      };

      const updatedMessages = messages.value.get(conversationId) || [];
      messages.value.set(conversationId, [...updatedMessages, aiMessage]);

      // 开始生成 AI 回复
      await chatAPI.generateAIResponse(content, conversationId);
    } catch (error) {
      console.error("发送消息失败:", error);
      throw error;
    } finally {
      isGenerating.value = false;
    }
  };

  // 处理消息块（流式响应）
  const handleMessageChunk = (chunk: {
    conversation_id: number;
    content: string;
    is_complete: boolean;
  }) => {
    const { conversation_id, content, is_complete } = chunk;

    if (conversation_id !== currentConversation.value?.id) return;

    const currentMessages = messages.value.get(conversation_id) || [];
    const lastMessage = currentMessages[currentMessages.length - 1];

    if (lastMessage && lastMessage.sender === "bot") {
      if (!is_complete && content) {
        // 追加内容
        lastMessage.content += content;
      }

      if (is_complete) {
        isGenerating.value = false;
        isTyping.value = false;
        // 重新加载消息以获取正确的 ID
        loadConversationMessages(conversation_id);
      } else {
        isTyping.value = true;
      }

      // 触发响应式更新
      messages.value.set(conversation_id, [...currentMessages]);
    }
  };

  // 删除消息
  const deleteMessage = async (messageId: number) => {
    try {
      await chatAPI.deleteMessage(messageId);

      // 从本地状态中移除消息
      if (currentConversation.value) {
        const conversationId = currentConversation.value.id;
        const currentMessages = messages.value.get(conversationId) || [];
        const filteredMessages = currentMessages.filter(
          (msg) => msg.id !== messageId,
        );
        messages.value.set(conversationId, filteredMessages);
      }
    } catch (error) {
      console.error("删除消息失败:", error);
      throw error;
    }
  };

  // 语音录制控制
  const startVoiceRecording = async () => {
    try {
      await chatAPI.startVoiceRecording();
      isRecording.value = true;
    } catch (error) {
      console.error("开始录音失败:", error);
      voiceSupported.value = false;
      throw error;
    }
  };

  const stopVoiceRecording = async () => {
    try {
      const transcript = await chatAPI.stopVoiceRecording();
      isRecording.value = false;
      return transcript;
    } catch (error) {
      console.error("停止录音失败:", error);
      isRecording.value = false;
      throw error;
    }
  };

  // 初始化聊天数据
  const initializeChat = async () => {
    try {
      await loadConversations();

      // 注册消息监听器
      chatAPI.onMessage("chat-store", handleMessageChunk);
    } catch (error) {
      console.error("初始化聊天失败:", error);
      throw error;
    }
  };

  // 清理资源
  const cleanup = () => {
    chatAPI.offMessage("chat-store");
  };

  return {
    // 状态
    conversations,
    currentConversation,
    messages,
    isTyping,
    isGenerating,
    isRecording,
    voiceSupported,
    // 计算属性
    currentMessages,
    hasConversations,
    canSendMessage,
    // 方法
    loadConversations,
    createConversation,
    deleteConversation,
    selectConversation,
    loadConversationMessages,
    sendMessage,
    deleteMessage,
    startVoiceRecording,
    stopVoiceRecording,
    initializeChat,
    cleanup,
    handleMessageChunk,
  };
});

// 窗口管理 Store
export const useWindowStore = defineStore("window", () => {
  const openWindows = ref<Set<string>>(new Set());
  const windowStates = ref<Map<string, any>>(new Map());

  // 记录窗口打开状态
  const setWindowOpen = (windowId: string, isOpen: boolean) => {
    if (isOpen) {
      openWindows.value.add(windowId);
    } else {
      openWindows.value.delete(windowId);
    }
  };

  // 保存窗口状态
  const saveWindowState = (windowId: string, state: any) => {
    windowStates.value.set(windowId, state);
  };

  // 获取窗口状态
  const getWindowState = (windowId: string) => {
    return windowStates.value.get(windowId);
  };

  // 检查窗口是否打开
  const isWindowOpen = (windowId: string) => {
    return openWindows.value.has(windowId);
  };

  return {
    openWindows,
    windowStates,
    setWindowOpen,
    saveWindowState,
    getWindowState,
    isWindowOpen,
  };
});
