/**
 * Chat Box API 接口管理
 * 统一管理与后端 Tauri 命令的交互
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { errorService } from "@/services/ErrorService";

// 类型定义
export interface Message {
  id: number;
  conversation_id: number;
  content: string;
  sender: "user" | "bot";
  timestamp: number;
}

export interface Conversation {
  id: number;
  title: string;
  timestamp: number;
}

export interface MessageChunk {
  conversation_id: number;
  content: string;
  is_complete: boolean;
}

export interface AppConfig {
  ai_model: {
    model_type: string;
    model_name: string;
    server_url: string;
    server_port: string;
    system_prompt: string;
    candle_model_id: string;
    candle_revision: string;
    candle_use_flash_attn: boolean;
  };
  ui: {
    theme: string;
    language: string;
  };
  voice: {
    enabled: boolean;
    model_path: string;
    timeout_seconds: number;
  };
  database: {
    enabled: boolean;
    path: string;
  };
  app_behavior: {
    log_level: string;
    default_conversation_title: string;
    welcome_message: string;
    message_chunk_buffer_size: number;
    message_chunk_send_interval_ms: number;
    show_error_dialogs: boolean;
    auto_retry_failed_init: boolean;
  };
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: string;
    timestamp: number;
  };
}

// API 类
export class ChatAPI {
  private static instance: ChatAPI;
  private messageListeners: Map<string, (data: MessageChunk) => void> =
    new Map();

  private constructor() {
    this.setupEventListeners();
  }

  public static getInstance(): ChatAPI {
    if (!ChatAPI.instance) {
      ChatAPI.instance = new ChatAPI();
    }
    return ChatAPI.instance;
  }

  // 设置事件监听器
  private async setupEventListeners() {
    await listen<MessageChunk>("message_chunk", (event) => {
      this.messageListeners.forEach((callback) => {
        callback(event.payload);
      });
    });
  }

  // 注册消息监听器
  public onMessage(id: string, callback: (data: MessageChunk) => void) {
    this.messageListeners.set(id, callback);
  }

  // 移除消息监听器
  public offMessage(id: string) {
    this.messageListeners.delete(id);
  }

  // 对话管理 API
  public async getConversations(): Promise<Conversation[]> {
    try {
      const response =
        await invoke<ApiResponse<Conversation[]>>("get_conversations");
      return errorService.handleApiResponse(response, "获取对话列表");
    } catch (error) {
      await errorService.handleError(error, "获取对话列表");
      throw error;
    }
  }

  public async createConversation(title: string): Promise<Conversation> {
    try {
      const response = await invoke<ApiResponse<Conversation>>(
        "create_conversation",
        { title },
      );
      return errorService.handleApiResponse(response, "创建对话");
    } catch (error) {
      await errorService.handleError(error, "创建对话");
      throw error;
    }
  }

  public async deleteConversation(conversationId: number): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("delete_conversation", {
        conversationId,
      });
      errorService.handleApiResponse(response, "删除对话");
    } catch (error) {
      await errorService.handleError(error, "删除对话");
      throw error;
    }
  }

  public async updateConversationTitle(
    conversationId: number,
    title: string,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>(
        "update_conversation_title",
        { conversationId, title },
      );
      errorService.handleApiResponse(response, "更新对话标题");
    } catch (error) {
      await errorService.handleError(error, "更新对话标题");
      throw error;
    }
  }

  // 消息管理 API
  public async getConversationMessages(
    conversationId: number,
  ): Promise<Message[]> {
    try {
      const response = await invoke<ApiResponse<Message[]>>(
        "get_conversation_messages",
        {
          conversationId,
        },
      );
      return errorService.handleApiResponse(response, "获取对话消息");
    } catch (error) {
      await errorService.handleError(error, "获取对话消息");
      throw error;
    }
  }

  public async sendUserMessage(
    content: string,
    conversationId: number,
  ): Promise<Message> {
    try {
      const response = await invoke<ApiResponse<Message>>("send_user_message", {
        content,
        conversationId,
      });
      return errorService.handleApiResponse(response, "发送用户消息");
    } catch (error) {
      await errorService.handleError(error, "发送用户消息");
      throw error;
    }
  }

  public async generateAIResponse(
    userMessageContent: string,
    conversationId: number,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("generate_ai_response", {
        userMessageContent,
        conversationId,
      });
      errorService.handleApiResponse(response, "生成AI回复");
    } catch (error) {
      await errorService.handleError(error, "生成AI回复");
      throw error;
    }
  }

  public async deleteMessage(messageId: number): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("delete_message", {
        messageId,
      });
      errorService.handleApiResponse(response, "删除消息");
    } catch (error) {
      await errorService.handleError(error, "删除消息");
      throw error;
    }
  }

  // 配置管理 API
  public async getAppConfig(): Promise<AppConfig> {
    try {
      const response = await invoke<ApiResponse<AppConfig>>("get_app_config");
      return errorService.handleApiResponse(response, "获取应用配置");
    } catch (error) {
      await errorService.handleError(error, "获取应用配置");
      throw error;
    }
  }

  public async saveAppConfig(config: AppConfig): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("save_app_config", {
        config,
      });
      errorService.handleApiResponse(response, "保存应用配置");
      // 发送配置更新事件到其他窗口
      await emit("config_updated", config);
      errorService.showSuccess("配置保存成功");
    } catch (error) {
      await errorService.handleError(error, "保存应用配置");
      throw error;
    }
  }

  public async resetAppConfig(): Promise<AppConfig> {
    try {
      const response = await invoke<ApiResponse<AppConfig>>("reset_app_config");
      const config = errorService.handleApiResponse(response, "重置应用配置");
      await emit("config_reset", {});
      errorService.showSuccess("配置重置成功");
      return config;
    } catch (error) {
      await errorService.handleError(error, "重置应用配置");
      throw error;
    }
  }

  // 语音功能 API
  public async startVoiceRecording(): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("start_voice_recording");
      errorService.handleApiResponse(response, "开始语音录制");
    } catch (error) {
      await errorService.handleError(error, "开始语音录制");
      throw error;
    }
  }

  public async stopVoiceRecording(): Promise<string> {
    try {
      const response = await invoke<ApiResponse<string>>(
        "stop_voice_recording",
      );
      return errorService.handleApiResponse(response, "停止语音录制");
    } catch (error) {
      await errorService.handleError(error, "停止语音录制");
      throw error;
    }
  }

  public async isVoiceRecording(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<boolean>>("is_voice_recording");
      return errorService.handleApiResponse(response, "检查语音录制状态");
    } catch (error) {
      await errorService.handleError(error, "检查语音录制状态", {
        showNotification: false,
      });
      return false;
    }
  }

  public async voiceInput(filePath?: string): Promise<string> {
    try {
      const response = await invoke<ApiResponse<string>>("voice_input", {
        filePath,
      });
      return errorService.handleApiResponse(response, "语音输入");
    } catch (error) {
      await errorService.handleError(error, "语音输入");
      throw error;
    }
  }

  // 数据库管理 API
  public async exportDatabase(format: string = "json"): Promise<string> {
    try {
      const response = await invoke<ApiResponse<string>>("export_database", {
        format,
      });
      const result = errorService.handleApiResponse(response, "导出数据库");
      errorService.showSuccess("数据库导出成功");
      return result;
    } catch (error) {
      await errorService.handleError(error, "导出数据库");
      throw error;
    }
  }

  public async importDatabase(filePath: string): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("import_database", {
        filePath,
      });
      errorService.handleApiResponse(response, "导入数据库");
      errorService.showSuccess("数据库导入成功");
    } catch (error) {
      await errorService.handleError(error, "导入数据库");
      throw error;
    }
  }

  public async clearDatabase(): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("clear_database");
      errorService.handleApiResponse(response, "清空数据库");
      errorService.showSuccess("数据库已清空");
    } catch (error) {
      await errorService.handleError(error, "清空数据库");
      throw error;
    }
  }

  public async getDatabaseConversations(): Promise<Conversation[]> {
    try {
      const response = await invoke<ApiResponse<Conversation[]>>(
        "get_database_conversations",
      );
      return errorService.handleApiResponse(response, "获取数据库对话");
    } catch (error) {
      await errorService.handleError(error, "获取数据库对话");
      throw error;
    }
  }

  public async deleteDatabaseConversation(
    conversationId: number,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>(
        "delete_database_conversation",
        { conversationId },
      );
      errorService.handleApiResponse(response, "删除数据库对话");
    } catch (error) {
      await errorService.handleError(error, "删除数据库对话");
      throw error;
    }
  }

  // 系统功能 API
  public async getSystemInfo(): Promise<Record<string, any>> {
    try {
      const response =
        await invoke<ApiResponse<Record<string, any>>>("get_system_info");
      return errorService.handleApiResponse(response, "获取系统信息");
    } catch (error) {
      await errorService.handleError(error, "获取系统信息");
      throw error;
    }
  }

  public async getHealthStatus(): Promise<Record<string, any>> {
    try {
      const response =
        await invoke<ApiResponse<Record<string, any>>>("get_health_status");
      return errorService.handleApiResponse(response, "获取健康状态");
    } catch (error) {
      await errorService.handleError(error, "获取健康状态");
      throw error;
    }
  }

  public async openFileDialog(
    filters?: Array<{ name: string; extensions: string[] }>,
  ): Promise<string | null> {
    try {
      const response = await invoke<ApiResponse<string | null>>(
        "open_file_dialog",
        { filters },
      );
      return errorService.handleApiResponse(response, "打开文件对话框");
    } catch (error) {
      await errorService.handleError(error, "打开文件对话框", {
        showNotification: false,
      });
      return null;
    }
  }

  public async saveFileDialog(
    defaultPath?: string,
    filters?: Array<{ name: string; extensions: string[] }>,
  ): Promise<string | null> {
    try {
      const response = await invoke<ApiResponse<string | null>>(
        "save_file_dialog",
        {
          defaultPath,
          filters,
        },
      );
      return errorService.handleApiResponse(response, "打开保存对话框");
    } catch (error) {
      await errorService.handleError(error, "打开保存对话框", {
        showNotification: false,
      });
      return null;
    }
  }

  // 实用方法
  public async checkConnection(): Promise<boolean> {
    try {
      const response = await invoke<ApiResponse<string>>("ping");
      errorService.handleApiResponse(response, "连接检查");
      return true;
    } catch (error) {
      await errorService.handleError(error, "连接检查", {
        showNotification: false,
      });
      return false;
    }
  }

  public async showNotification(
    title: string,
    body: string,
    icon?: string,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("show_notification", {
        title,
        body,
        icon,
      });
      errorService.handleApiResponse(response, "显示通知");
    } catch (error) {
      await errorService.handleError(error, "显示通知", {
        showNotification: false,
      });
    }
  }

  // 日志记录 API
  public async log_error(
    errorCode: string,
    errorMessage: string,
    errorDetails?: string,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("log_error", {
        error_code: errorCode,
        error_message: errorMessage,
        error_details: errorDetails,
      });
      errorService.handleApiResponse(response, "记录错误");
    } catch (error) {
      console.error("记录错误失败:", error);
    }
  }

  public async log_warning(
    warningMessage: string,
    warningDetails?: string,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("log_warning", {
        warning_message: warningMessage,
        warning_details: warningDetails,
      });
      errorService.handleApiResponse(response, "记录警告");
    } catch (error) {
      console.error("记录警告失败:", error);
    }
  }

  public async log_info(
    infoMessage: string,
    infoDetails?: string,
  ): Promise<void> {
    try {
      const response = await invoke<ApiResponse<void>>("log_info", {
        info_message: infoMessage,
        info_details: infoDetails,
      });
      errorService.handleApiResponse(response, "记录信息");
    } catch (error) {
      console.error("记录信息失败:", error);
    }
  }
}

// 导出单例实例
export const chatAPI = ChatAPI.getInstance();

// 导出便利函数
export const {
  getConversations,
  createConversation,
  deleteConversation,
  updateConversationTitle,
  getConversationMessages,
  sendUserMessage,
  generateAIResponse,
  deleteMessage,
  getAppConfig,
  saveAppConfig,
  resetAppConfig,
  startVoiceRecording,
  stopVoiceRecording,
  isVoiceRecording,
  voiceInput,
  exportDatabase,
  importDatabase,
  clearDatabase,
  getDatabaseConversations,
  deleteDatabaseConversation,
  getSystemInfo,
  getHealthStatus,
  openFileDialog,
  saveFileDialog,
  checkConnection,
  showNotification,
  log_error,
  log_warning,
  log_info,
  onMessage,
  offMessage,
} = chatAPI;
