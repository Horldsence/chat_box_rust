import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Message,
  Conversation,
  ApiResponse,
  AppConfig,
  SystemInfo,
  HealthStatus,
  MessageChunk,
  VoiceStatus,
  FileDialogOptions,
  ExportFormat,
  ConfigFormat,
} from "../types";

// Conversation API
export const conversationApi = {
  async getAll(): Promise<Conversation[]> {
    return invoke("get_conversations");
  },

  async getMessages(conversationId: number): Promise<Message[]> {
    return invoke("get_conversation_messages", { conversationId });
  },

  async create(title: string): Promise<Conversation> {
    return invoke("create_conversation", { title });
  },

  async delete(conversationId: number): Promise<void> {
    return invoke("delete_conversation", {
      request: {
        conversation_id: conversationId,
      },
    });
  },
};

// Message API
export const messageApi = {
  async sendUserMessage(
    content: string,
    conversationId: number,
  ): Promise<Message> {
    return invoke("send_user_message", {
      request: {
        content,
        conversation_id: conversationId,
      },
    });
  },

  async generateAIResponse(
    userMessageContent: string,
    conversationId: number,
  ): Promise<void> {
    return invoke("generate_ai_response", {
      request: {
        user_message_content: userMessageContent,
        conversation_id: conversationId,
      },
    });
  },

  // Listen for streaming message chunks
  onMessageChunk(callback: (chunk: MessageChunk) => void) {
    return listen("message_chunk", (event) => {
      callback(event.payload as MessageChunk);
    });
  },
};

// Voice API
export const voiceApi = {
  async startVoiceInput(conversationId: number): Promise<string> {
    if (conversationId == null || conversationId <= 0) {
      throw new Error("Invalid conversationId: must be a positive number");
    }
    return invoke("voice_input", { conversationId });
  },

  // Listen for voice status updates
  onVoiceStatus(callback: (status: VoiceStatus) => void) {
    return listen("voice_status", (event) => {
      callback(event.payload as VoiceStatus);
    });
  },

  // Listen for partial voice transcriptions
  onVoicePartial(callback: (text: string) => void) {
    return listen("voice_partial", (event) => {
      callback(event.payload as string);
    });
  },
};

// Settings API
export const settingsApi = {
  async getConfig(): Promise<ApiResponse<AppConfig>> {
    return invoke("get_app_config");
  },

  async saveConfig(config: AppConfig): Promise<ApiResponse<void>> {
    return invoke("save_app_config", { config });
  },

  async resetConfig(): Promise<ApiResponse<AppConfig>> {
    return invoke("reset_app_config");
  },

  async getSystemInfo(): Promise<ApiResponse<SystemInfo>> {
    return invoke("get_system_info");
  },

  async getHealthStatus(): Promise<ApiResponse<HealthStatus>> {
    return invoke("get_health_status");
  },

  async ping(): Promise<ApiResponse<string>> {
    return invoke("ping");
  },

  async showNotification(
    title: string,
    body: string,
    icon?: string,
  ): Promise<ApiResponse<void>> {
    return invoke("show_notification", { title, body, icon });
  },

  async logError(
    errorCode: string,
    errorMessage: string,
    errorDetails?: string,
  ): Promise<ApiResponse<void>> {
    return invoke("log_error", { errorCode, errorMessage, errorDetails });
  },

  async logWarning(
    warningMessage: string,
    warningDetails?: string,
  ): Promise<ApiResponse<void>> {
    return invoke("log_warning", { warningMessage, warningDetails });
  },

  async logInfo(
    infoMessage: string,
    infoDetails?: string,
  ): Promise<ApiResponse<void>> {
    return invoke("log_info", { infoMessage, infoDetails });
  },
};

// Dialog API
export const dialogApi = {
  async showInfo(
    title: string,
    message: string,
  ): Promise<ApiResponse<boolean>> {
    return invoke("show_info_dialog", { title, message });
  },

  async showWarning(
    title: string,
    message: string,
  ): Promise<ApiResponse<boolean>> {
    return invoke("show_warning_dialog", { title, message });
  },

  async showError(
    title: string,
    message: string,
  ): Promise<ApiResponse<boolean>> {
    return invoke("show_error_dialog", { title, message });
  },

  async showConfirm(
    title: string,
    message: string,
  ): Promise<ApiResponse<boolean>> {
    return invoke("show_confirm_dialog", { title, message });
  },

  async showAsk(title: string, message: string): Promise<ApiResponse<boolean>> {
    return invoke("show_ask_dialog", { title, message });
  },

  async openFile(
    options: FileDialogOptions = {},
  ): Promise<ApiResponse<string[] | null>> {
    const { title, filters, multiple } = options;
    return invoke("open_file_dialog", { title, filters, multiple });
  },

  async openFolder(title?: string): Promise<ApiResponse<string | null>> {
    return invoke("open_folder_dialog", { title });
  },

  async saveFile(
    options: FileDialogOptions = {},
  ): Promise<ApiResponse<string | null>> {
    const { title, defaultName, filters } = options;
    return invoke("save_file_dialog", { title, defaultName, filters });
  },

  async importConfig(): Promise<ApiResponse<string | null>> {
    return invoke("import_config_file");
  },

  async exportConfig(
    content: string,
    format: ConfigFormat,
  ): Promise<ApiResponse<boolean>> {
    return invoke("export_config_file", { content, format });
  },

  async exportChatHistory(
    conversationId?: number,
    format: ExportFormat = "json",
  ): Promise<ApiResponse<boolean>> {
    return invoke("export_chat_history", { conversationId, format });
  },

  async selectVoiceModelFolder(): Promise<ApiResponse<string | null>> {
    return invoke("select_voice_model_folder");
  },

  async selectDatabaseFile(): Promise<ApiResponse<string | null>> {
    return invoke("select_database_file");
  },

  async createDatabaseFile(): Promise<ApiResponse<string | null>> {
    return invoke("create_database_file");
  },
};

// Database API
export const databaseApi = {
  async getConversations(): Promise<Conversation[]> {
    return invoke("get_database_conversations");
  },

  async deleteConversation(conversationId: number): Promise<void> {
    return invoke("delete_database_conversation", { conversationId });
  },
};

// Utility functions
export const utils = {
  formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toLocaleString();
  },

  formatRelativeTime(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (seconds < 60) return "just now";
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return new Date(timestamp).toLocaleDateString();
  },

  truncateText(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength) + "...";
  },

  generateConversationTitle(firstMessage: string): string {
    const maxLength = 30;
    const cleaned = firstMessage.trim().replace(/\n/g, " ");
    return utils.truncateText(cleaned, maxLength);
  },

  isValidUrl(string: string): boolean {
    try {
      new URL(string);
      return true;
    } catch {
      return false;
    }
  },

  async copyToClipboard(text: string): Promise<boolean> {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      return false;
    }
  },

  downloadFile(
    content: string,
    filename: string,
    contentType: string = "text/plain",
  ) {
    const blob = new Blob([content], { type: contentType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  },

  async handleApiError(
    error: any,
    context: string = "API call",
  ): Promise<void> {
    console.error(`Error in ${context}:`, error);

    const errorMessage =
      typeof error === "string" ? error : error?.message || "Unknown error";
    const errorCode = error?.code || "UNKNOWN_ERROR";

    await settingsApi.logError(
      errorCode,
      `${context}: ${errorMessage}`,
      JSON.stringify(error),
    );
  },

  debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number,
  ): (...args: Parameters<T>) => void {
    let timeout: number;
    return (...args: Parameters<T>) => {
      clearTimeout(timeout);
      timeout = setTimeout(() => func(...args), wait) as any;
    };
  },

  throttle<T extends (...args: any[]) => any>(
    func: T,
    limit: number,
  ): (...args: Parameters<T>) => void {
    let inThrottle: boolean;
    return (...args: Parameters<T>) => {
      if (!inThrottle) {
        func(...args);
        inThrottle = true;
        setTimeout(() => (inThrottle = false), limit);
      }
    };
  },
};

// Error handling wrapper for API calls
export const withErrorHandling = async <T>(
  apiCall: () => Promise<T>,
  context: string = "API call",
): Promise<T | null> => {
  try {
    return await apiCall();
  } catch (error) {
    await utils.handleApiError(error, context);
    return null;
  }
};

// Health check utility
// Debug API
export const debugApi = {
  async getDatabaseStatus(): Promise<any> {
    return invoke("debug_database_status");
  },

  async getMemoryState(): Promise<string> {
    return invoke("debug_memory_state");
  },

  async clearDatabase(): Promise<string> {
    return invoke("debug_clear_database");
  },

  async testDatabaseConnection(): Promise<string> {
    return invoke("debug_test_database_connection");
  },
};

export const healthCheck = {
  async checkConnection(): Promise<boolean> {
    try {
      const response = await settingsApi.ping();
      return response.success && response.data === "pong";
    } catch {
      return false;
    }
  },

  async getDetailedStatus(): Promise<HealthStatus | null> {
    try {
      const response = await settingsApi.getHealthStatus();
      return response.success ? response.data || null : null;
    } catch {
      return null;
    }
  },
};
