import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Conversation, Message } from "$lib/types";

// ===== Store State Interface =====

interface ChatStoreState {
  conversations: Conversation[];
  currentConversation: Conversation | null;
  messages: Message[];
  isLoading: boolean;
  isTyping: boolean;
  error: string | null;
  searchResults: Message[];
  searchQuery: string;
}

// ===== Initial State =====

const initialState: ChatStoreState = {
  conversations: [],
  currentConversation: null,
  messages: [],
  isLoading: false,
  isTyping: false,
  error: null,
  searchResults: [],
  searchQuery: "",
};

// ===== Create Chat Store =====

function createChatStore() {
  const { subscribe, set, update } = writable<ChatStoreState>(initialState);

  return {
    subscribe,

    // ===== 初始化和清理 =====

    async init() {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        // 加载对话列表
        await this.loadConversations();

        update((state) => ({ ...state, isLoading: false }));
        console.log("Chat store initialized successfully");
      } catch (error) {
        console.error("Failed to initialize chat store:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `初始化失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    async destroy() {
      set(initialState);
    },

    // ===== 对话管理 =====

    async loadConversations() {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        const conversations: Conversation[] = await invoke("get_conversations");

        // 按时间戳降序排列
        const sortedConversations = conversations.sort((a, b) => b.timestamp - a.timestamp);

        update((state) => ({
          ...state,
          conversations: sortedConversations,
          isLoading: false,
        }));

        console.log("Loaded conversations:", sortedConversations.length);
      } catch (error) {
        console.error("Failed to load conversations:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `加载对话失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    async createConversation(title: string) {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        const newConversation: Conversation = await invoke("create_conversation", { title });

        update((state) => ({
          ...state,
          conversations: [newConversation, ...state.conversations],
          currentConversation: newConversation,
          messages: [],
          isLoading: false,
        }));

        console.log("Created new conversation:", newConversation);
        return newConversation;
      } catch (error) {
        console.error("Failed to create conversation:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `创建对话失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    async selectConversation(conversationId: number) {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        const conversation = await this.getConversationById(conversationId);
        if (!conversation) {
          throw new Error("对话不存在");
        }

        // 加载对话消息
        await this.loadMessages(conversationId);

        update((state) => ({
          ...state,
          currentConversation: conversation,
          isLoading: false,
        }));

        console.log("Selected conversation:", conversation);
      } catch (error) {
        console.error("Failed to select conversation:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `选择对话失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    async deleteConversation(conversationId: number) {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        await invoke("delete_conversation", { conversation_id: conversationId });

        update((state) => {
          const filteredConversations = state.conversations.filter((c) => c.id !== conversationId);
          const isCurrentConversation = state.currentConversation?.id === conversationId;

          return {
            ...state,
            conversations: filteredConversations,
            currentConversation: isCurrentConversation ? null : state.currentConversation,
            messages: isCurrentConversation ? [] : state.messages,
            isLoading: false,
          };
        });

        console.log("Deleted conversation:", conversationId);
      } catch (error) {
        console.error("Failed to delete conversation:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `删除对话失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    // ===== 消息管理 =====

    async loadMessages(conversationId: number) {
      try {
        const messages: Message[] = await invoke("get_conversation_messages", {
          conversation_id: conversationId,
        });

        // 按时间戳升序排列
        const sortedMessages = messages.sort((a, b) => a.timestamp - b.timestamp);

        update((state) => ({
          ...state,
          messages: sortedMessages,
        }));

        console.log("Loaded messages for conversation", conversationId, ":", sortedMessages.length);
      } catch (error) {
        console.error("Failed to load messages:", error);
        update((state) => ({
          ...state,
          error: `加载消息失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    async sendMessage(content: string) {
      const state = this.getCurrentState();
      if (!state.currentConversation) {
        throw new Error("没有选中的对话");
      }

      try {
        update((state) => ({ ...state, isLoading: true, isTyping: true, error: null }));

        const userMessage: Message = await invoke("send_user_message", {
          content,
          conversation_id: state.currentConversation!.id,
        });

        // 添加用户消息到界面
        this.addMessage(userMessage);

        // 更新对话的最后消息
        this.updateConversationLastMessage(state.currentConversation!.id, content);

        // TODO: 这里可以添加AI响应逻辑

        update((state) => ({ ...state, isLoading: false, isTyping: false }));

        console.log("Sent message:", userMessage);
        return userMessage;
      } catch (error) {
        console.error("Failed to send message:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          isTyping: false,
          error: `发送消息失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    addMessage(message: Message) {
      update((state) => ({
        ...state,
        messages: [...state.messages, message],
      }));
    },

    // ===== 工具方法 =====

    async getConversationById(id: number): Promise<Conversation | null> {
      const state = this.getCurrentState();
      return state.conversations.find((c) => c.id === id) || null;
    },

    getCurrentState(): ChatStoreState {
      let currentState: ChatStoreState = initialState;
      subscribe((state) => {
        currentState = state;
      })();
      return currentState;
    },

    updateConversationLastMessage(conversationId: number, lastMessage: string) {
      update((state) => ({
        ...state,
        conversations: state.conversations
          .map((conv) =>
            conv.id === conversationId
              ? { ...conv, last_message: lastMessage, timestamp: Date.now() }
              : conv
          )
          .sort((a, b) => b.timestamp - a.timestamp),
      }));
    },

    // ===== 状态管理 =====

    setLoading(loading: boolean) {
      update((state) => ({ ...state, isLoading: loading }));
    },

    setTyping(typing: boolean) {
      update((state) => ({ ...state, isTyping: typing }));
    },

    setError(error: string | null) {
      update((state) => ({ ...state, error }));
    },

    clearError() {
      update((state) => ({ ...state, error: null }));
    },

    // ===== 搜索功能 =====

    async searchMessages(query: string) {
      if (!query.trim()) {
        this.clearSearch();
        return;
      }

      try {
        update((state) => ({ ...state, isLoading: true, searchQuery: query }));

        const results = this.performLocalSearch(query);

        update((state) => ({
          ...state,
          searchResults: results,
          isLoading: false,
        }));

        console.log("Search results:", results.length);
      } catch (error) {
        console.error("Search failed:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `搜索失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    clearSearch() {
      update((state) => ({
        ...state,
        searchResults: [],
        searchQuery: "",
      }));
    },

    performLocalSearch(query: string): Message[] {
      const state = this.getCurrentState();
      const searchTerm = query.toLowerCase();

      return state.messages.filter((message) => message.content.toLowerCase().includes(searchTerm));
    },

    // ===== 统计信息 =====

    getStats() {
      const state = this.getCurrentState();
      const userMessages = state.messages.filter((m) => m.sender === "user");
      const assistantMessages = state.messages.filter((m) => m.sender === "assistant");

      return {
        totalConversations: state.conversations.length,
        totalMessages: state.messages.length,
        userMessages: userMessages.length,
        assistantMessages: assistantMessages.length,
        averageMessagesPerConversation:
          state.conversations.length > 0
            ? Math.round(state.messages.length / state.conversations.length)
            : 0,
      };
    },
  };
}

// ===== 导出 Store 实例 =====

export const chatStore = createChatStore();

// ===== 导出派生 Store =====

export const conversations = derived(chatStore, ($store) => $store.conversations);
export const currentConversation = derived(chatStore, ($store) => $store.currentConversation);
export const messages = derived(chatStore, ($store) => $store.messages);
export const isLoading = derived(chatStore, ($store) => $store.isLoading);
export const isTyping = derived(chatStore, ($store) => $store.isTyping);
export const chatError = derived(chatStore, ($store) => $store.error);
export const searchResults = derived(chatStore, ($store) => $store.searchResults);
export const hasActiveConversation = derived(
  chatStore,
  ($store) => $store.currentConversation !== null
);
export const messageCount = derived(chatStore, ($store) => $store.messages.length);
export const lastMessage = derived(chatStore, ($store) => {
  if ($store.messages.length === 0) return null;
  return $store.messages[$store.messages.length - 1];
});
