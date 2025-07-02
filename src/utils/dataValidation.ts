import type { Message, Conversation, AppConfig, HealthStatus } from "../types";

// 数据验证工具函数
export const dataValidation = {
  // 验证消息格式
  validateMessage(message: any): message is Message {
    return (
      typeof message === "object" &&
      message !== null &&
      typeof message.id === "number" &&
      typeof message.content === "string" &&
      typeof message.sender === "string" &&
      ["user", "assistant", "system"].includes(message.sender) &&
      typeof message.timestamp === "number" &&
      typeof message.conversation_id === "number"
    );
  },

  // 验证对话格式
  validateConversation(conversation: any): conversation is Conversation {
    return (
      typeof conversation === "object" &&
      conversation !== null &&
      typeof conversation.id === "number" &&
      typeof conversation.title === "string" &&
      typeof conversation.last_message === "string" &&
      typeof conversation.timestamp === "number"
    );
  },

  // 验证消息数组
  validateMessages(messages: any[]): messages is Message[] {
    return Array.isArray(messages) && messages.every(this.validateMessage);
  },

  // 验证对话数组
  validateConversations(conversations: any[]): conversations is Conversation[] {
    return Array.isArray(conversations) && conversations.every(this.validateConversation);
  },

  // 验证应用配置
  validateAppConfig(config: any): config is AppConfig {
    return (
      typeof config === "object" &&
      config !== null &&
      typeof config.app_behavior === "object" &&
      typeof config.app_behavior.message_chunk_buffer_size === "number" &&
      typeof config.app_behavior.message_chunk_send_interval_ms === "number"
    );
  },

  // 验证健康状态
  validateHealthStatus(status: any): status is HealthStatus {
    return (
      typeof status === "object" &&
      status !== null &&
      typeof status.config_loaded === "boolean" &&
      typeof status.database_connected === "boolean" &&
      typeof status.llm_available === "boolean" &&
      typeof status.voice_recognition_available === "boolean" &&
      typeof status.conversation_count === "number" &&
      typeof status.message_count === "number" &&
      typeof status.uptime_ms === "number"
    );
  },

  // 清理和标准化消息数据
  sanitizeMessage(message: any): Message | null {
    try {
      const sanitized = {
        id: Number(message.id),
        content: String(message.content).trim(),
        sender: String(message.sender).toLowerCase(),
        timestamp: Number(message.timestamp),
        conversation_id: Number(message.conversation_id),
      };

      // 确保sender值正确
      if (!["user", "assistant", "system"].includes(sanitized.sender)) {
        if (sanitized.sender === "bot") {
          sanitized.sender = "assistant";
        } else {
          console.warn(`Unknown sender type: ${sanitized.sender}, defaulting to assistant`);
          sanitized.sender = "assistant";
        }
      }

      return this.validateMessage(sanitized) ? sanitized : null;
    } catch (error) {
      console.error("Failed to sanitize message:", error);
      return null;
    }
  },

  // 清理和标准化对话数据
  sanitizeConversation(conversation: any): Conversation | null {
    try {
      const sanitized = {
        id: Number(conversation.id),
        title: String(conversation.title).trim() || "未命名对话",
        last_message: String(conversation.last_message).trim() || "暂无消息",
        timestamp: Number(conversation.timestamp),
      };

      return this.validateConversation(sanitized) ? sanitized : null;
    } catch (error) {
      console.error("Failed to sanitize conversation:", error);
      return null;
    }
  },

  // 批量清理消息数组
  sanitizeMessages(messages: any[]): Message[] {
    if (!Array.isArray(messages)) {
      console.error("Expected array for messages, got:", typeof messages);
      return [];
    }

    return messages
      .map(this.sanitizeMessage.bind(this))
      .filter((message): message is Message => message !== null)
      .sort((a, b) => a.timestamp - b.timestamp);
  },

  // 批量清理对话数组
  sanitizeConversations(conversations: any[]): Conversation[] {
    if (!Array.isArray(conversations)) {
      console.error("Expected array for conversations, got:", typeof conversations);
      return [];
    }

    return conversations
      .map(this.sanitizeConversation.bind(this))
      .filter((conversation): conversation is Conversation => conversation !== null)
      .sort((a, b) => b.timestamp - a.timestamp);
  },

  // 验证API响应格式
  validateApiResponse<T>(response: any, dataValidator?: (data: any) => boolean): {
    success: boolean;
    data?: T;
    error?: string;
  } {
    try {
      // 检查基本结构
      if (typeof response !== "object" || response === null) {
        return { success: false, error: "Invalid response format" };
      }

      // 如果响应有success字段，检查是否为错误响应
      if ("success" in response && response.success === false) {
        return {
          success: false,
          error: response.error?.message || response.error || "Unknown error",
        };
      }

      // 如果有数据验证器，使用它验证数据
      if (dataValidator && response.data && !dataValidator(response.data)) {
        return { success: false, error: "Data validation failed" };
      }

      // 如果响应直接是数据（Tauri命令通常直接返回数据）
      if (!("success" in response) && !("error" in response)) {
        return { success: true, data: response as T };
      }

      return { success: true, data: response.data as T };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Validation failed",
      };
    }
  },

  // 检查数据一致性
  checkDataConsistency(messages: Message[], conversations: Conversation[]): {
    isConsistent: boolean;
    issues: string[];
  } {
    const issues: string[] = [];

    // 检查消息是否都属于存在的对话
    const conversationIds = new Set(conversations.map(c => c.id));
    const orphanMessages = messages.filter(m => !conversationIds.has(m.conversation_id));

    if (orphanMessages.length > 0) {
      issues.push(`Found ${orphanMessages.length} orphan messages`);
    }

    // 检查对话的最后消息时间戳是否与最新消息一致
    for (const conversation of conversations) {
      const conversationMessages = messages.filter(m => m.conversation_id === conversation.id);
      if (conversationMessages.length > 0) {
        const latestMessage = conversationMessages.reduce((latest, current) =>
          current.timestamp > latest.timestamp ? current : latest
        );

        if (Math.abs(conversation.timestamp - latestMessage.timestamp) > 1000) {
          issues.push(`Conversation ${conversation.id} timestamp mismatch`);
        }
      }
    }

    // 检查消息ID唯一性
    const messageIds = messages.map(m => m.id);
    const uniqueIds = new Set(messageIds);
    if (messageIds.length !== uniqueIds.size) {
      issues.push("Duplicate message IDs found");
    }

    // 检查对话ID唯一性
    const conversationIdsArray = conversations.map(c => c.id);
    const uniqueConversationIds = new Set(conversationIdsArray);
    if (conversationIdsArray.length !== uniqueConversationIds.size) {
      issues.push("Duplicate conversation IDs found");
    }

    return {
      isConsistent: issues.length === 0,
      issues,
    };
  },

  // 生成数据统计报告
  generateDataStats(messages: Message[], conversations: Conversation[]) {
    const stats = {
      totalConversations: conversations.length,
      totalMessages: messages.length,
      messagesPerConversation: {} as Record<number, number>,
      senderDistribution: {} as Record<string, number>,
      averageMessageLength: 0,
      oldestConversation: null as Conversation | null,
      newestConversation: null as Conversation | null,
      timeRange: {
        start: 0,
        end: 0,
        duration: 0,
      },
    };

    // 消息分布统计
    messages.forEach(message => {
      stats.messagesPerConversation[message.conversation_id] =
        (stats.messagesPerConversation[message.conversation_id] || 0) + 1;

      stats.senderDistribution[message.sender] =
        (stats.senderDistribution[message.sender] || 0) + 1;
    });

    // 平均消息长度
    if (messages.length > 0) {
      stats.averageMessageLength =
        messages.reduce((sum, msg) => sum + msg.content.length, 0) / messages.length;
    }

    // 时间范围
    if (conversations.length > 0) {
      const timestamps = conversations.map(c => c.timestamp);
      stats.timeRange.start = Math.min(...timestamps);
      stats.timeRange.end = Math.max(...timestamps);
      stats.timeRange.duration = stats.timeRange.end - stats.timeRange.start;

      stats.oldestConversation = conversations.find(c => c.timestamp === stats.timeRange.start) || null;
      stats.newestConversation = conversations.find(c => c.timestamp === stats.timeRange.end) || null;
    }

    return stats;
  }
};

// 错误处理工具
export const errorHandler = {
  // 标准化错误消息
  normalizeError(error: any): string {
    if (typeof error === "string") {
      return error;
    }

    if (error instanceof Error) {
      return error.message;
    }

    if (typeof error === "object" && error !== null) {
      if ("message" in error) {
        return String(error.message);
      }
      if ("error" in error) {
        return String(error.error);
      }
    }

    return "Unknown error occurred";
  },

  // 创建用户友好的错误消息
  createUserFriendlyError(error: any, context?: string): string {
    const baseError = this.normalizeError(error);
    const contextPrefix = context ? `${context}: ` : "";

    // 将技术错误转换为用户友好的消息
    const errorMappings: Record<string, string> = {
      "network error": "网络连接失败，请检查网络设置",
      "database error": "数据库操作失败，请重试",
      "validation failed": "数据格式不正确",
      "timeout": "操作超时，请重试",
      "unauthorized": "认证失败，请重新登录",
      "not found": "请求的资源不存在",
    };

    const lowerError = baseError.toLowerCase();
    for (const [key, message] of Object.entries(errorMappings)) {
      if (lowerError.includes(key)) {
        return `${contextPrefix}${message}`;
      }
    }

    return `${contextPrefix}${baseError}`;
  }
};
