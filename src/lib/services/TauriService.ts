/**
 * Tauri API服务层
 * 提供与Rust后端交互的统一接口，包括错误处理、类型安全和事件管理
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, emit, type UnlistenFn, type EventCallback } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type {
  Conversation,
  Message,
  ApiResponse,
  TauriCommand,
  TauriEventPayload,
} from "$lib/types";

// 命令超时配置
const DEFAULT_TIMEOUT = 10000; // 10秒
const RETRY_ATTEMPTS = 3;
const RETRY_DELAY = 1000; // 1秒

// 事件监听器管理
interface EventListener {
  event: string;
  callback: EventCallback<any>;
  unlisten: UnlistenFn;
}

class TauriService {
  private static instance: TauriService;
  private eventListeners: Map<string, EventListener[]> = new Map();
  private isInitialized = false;

  private constructor() {}

  public static getInstance(): TauriService {
    if (!TauriService.instance) {
      TauriService.instance = new TauriService();
    }
    return TauriService.instance;
  }

  /**
   * 初始化服务
   */
  public async init(): Promise<void> {
    if (this.isInitialized) return;

    try {
      // 可以在这里进行初始化检查
      const response = await this.invokeCommand<ApiResponse<string>>("ping");
      this.handleApiResponse(response);
      this.isInitialized = true;
      console.log("TauriService initialized successfully");
    } catch (error) {
      console.error("Failed to initialize TauriService:", error);
      throw error;
    }
  }

  /**
   * 销毁服务，清理所有事件监听器
   */
  public destroy(): void {
    this.eventListeners.forEach((listeners) => {
      listeners.forEach((listener) => {
        listener.unlisten();
      });
    });
    this.eventListeners.clear();
    this.isInitialized = false;
  }

  // ===== 核心调用方法 =====

  /**
   * 安全的Tauri命令调用
   */
  private async invokeCommand<T>(
    command: string,
    args?: Record<string, any>,
    options: { timeout?: number; retries?: number } = {}
  ): Promise<T> {
    const { timeout = DEFAULT_TIMEOUT, retries = RETRY_ATTEMPTS } = options;

    let lastError: Error = new Error("Unknown error occurred");

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        // 创建超时Promise
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(
            () => reject(new Error(`Command '${command}' timed out after ${timeout}ms`)),
            timeout
          );
        });

        // 执行命令
        const commandPromise = invoke<T>(command, args);

        // 等待命令完成或超时
        const result = await Promise.race([commandPromise, timeoutPromise]);

        return result;
      } catch (error) {
        lastError = error as Error;

        // 如果不是最后一次尝试，等待重试
        if (attempt < retries) {
          console.warn(
            `Command '${command}' failed (attempt ${attempt + 1}/${retries + 1}):`,
            error
          );
          await this.delay(RETRY_DELAY * (attempt + 1)); // 递增延迟
        }
      }
    }

    // 所有重试都失败了
    throw new Error(
      `Command '${command}' failed after ${retries + 1} attempts: ${lastError.message}`
    );
  }

  /**
   * 处理API响应
   */
  private handleApiResponse<T>(response: ApiResponse<T>): T {
    if (!response.success) {
      const error = new Error(response.error?.message || "Unknown API error");
      Object.assign(error, {
        code: response.error?.code,
        details: response.error?.details,
        timestamp: response.error?.timestamp,
        apiError: true,
      });
      throw error;
    }

    if (response.data === undefined || response.data === null) {
      throw new Error("API response data is missing");
    }

    return response.data;
  }

  /**
   * 延迟函数
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // ===== 对话管理API =====

  /**
   * 获取所有对话
   */
  public async getConversations(): Promise<Conversation[]> {
    const response = await this.invokeCommand<ApiResponse<Conversation[]>>("get_conversations");
    return this.handleApiResponse(response);
  }

  /**
   * 创建新对话
   */
  public async createConversation(title: string): Promise<Conversation> {
    const response = await this.invokeCommand<ApiResponse<Conversation>>("create_conversation", {
      title,
    });
    return this.handleApiResponse(response);
  }

  /**
   * 获取对话消息
   */
  public async getConversationMessages(conversationId: string): Promise<Message[]> {
    const response = await this.invokeCommand<ApiResponse<Message[]>>("get_conversation_messages", {
      conversationId,
    });
    return this.handleApiResponse(response);
  }

  /**
   * 发送消息
   */
  public async sendMessage(
    conversationId: string,
    content: string,
    sender: "user" | "assistant" = "user"
  ): Promise<Message> {
    const response = await this.invokeCommand<ApiResponse<Message>>("send_message", {
      conversationId,
      content,
      sender,
    });
    return this.handleApiResponse(response);
  }

  /**
   * 清空对话
   */
  public async clearConversation(conversationId: string): Promise<boolean> {
    const response = await this.invokeCommand<ApiResponse<boolean>>("clear_conversation", {
      conversationId,
    });
    return this.handleApiResponse(response);
  }

  /**
   * 删除对话
   */
  public async deleteConversation(conversationId: string): Promise<boolean> {
    const response = await this.invokeCommand<ApiResponse<boolean>>("delete_conversation", {
      conversationId,
    });
    return this.handleApiResponse(response);
  }

  // ===== 日志API =====

  /**
   * 记录错误日志
   */
  public async logError(code: string, message: string, details?: string): Promise<void> {
    await this.invokeCommand("log_error", { code, message, details });
  }

  /**
   * 记录警告日志
   */
  public async logWarning(message: string, details?: string): Promise<void> {
    await this.invokeCommand("log_warning", { message, details });
  }

  /**
   * 记录信息日志
   */
  public async logInfo(message: string, details?: string): Promise<void> {
    await this.invokeCommand("log_info", { message, details });
  }

  // ===== 事件管理 =====

  /**
   * 监听事件
   */
  public async addEventListener<T>(
    event: string,
    callback: EventCallback<T>,
    options: { once?: boolean } = {}
  ): Promise<() => void> {
    try {
      const unlisten = await listen<T>(event, callback);

      const listener: EventListener = {
        event,
        callback,
        unlisten,
      };

      // 存储监听器
      if (!this.eventListeners.has(event)) {
        this.eventListeners.set(event, []);
      }
      this.eventListeners.get(event)!.push(listener);

      // 如果是一次性监听器，包装回调
      if (options.once) {
        const originalCallback = callback;
        const onceCallback: EventCallback<T> = (event) => {
          try {
            originalCallback(event);
          } finally {
            this.removeEventListener(event.event, callback);
          }
        };
      }

      // 返回取消监听的函数
      return () => {
        this.removeEventListener(event, callback);
      };
    } catch (error) {
      console.error(`Failed to add event listener for '${event}':`, error);
      throw error;
    }
  }

  /**
   * 移除事件监听器
   */
  public removeEventListener(event: string, callback?: EventCallback<any>): void {
    const listeners = this.eventListeners.get(event);
    if (!listeners) return;

    if (callback) {
      // 移除特定回调的监听器
      const index = listeners.findIndex((listener) => listener.callback === callback);
      if (index !== -1) {
        listeners[index].unlisten();
        listeners.splice(index, 1);
      }
    } else {
      // 移除该事件的所有监听器
      listeners.forEach((listener) => listener.unlisten());
      this.eventListeners.delete(event);
    }
  }

  /**
   * 发送事件
   */
  public async emitEvent<T>(event: string, payload?: T): Promise<void> {
    try {
      await emit(event, payload);
    } catch (error) {
      console.error(`Failed to emit event '${event}':`, error);
      throw error;
    }
  }

  /**
   * 监听聊天响应事件
   */
  public async onChatResponse(callback: (message: Message) => void): Promise<() => void> {
    return this.addEventListener<Message>("chat_response", (event) => {
      callback(event.payload);
    });
  }

  /**
   * 监听系统事件
   */
  public async onSystemEvent(callback: (event: any) => void): Promise<() => void> {
    return this.addEventListener("system_event", (event) => {
      callback(event.payload);
    });
  }

  // ===== 窗口管理 =====

  /**
   * 获取当前窗口
   */
  public getCurrentWindow(): WebviewWindow {
    return WebviewWindow.getCurrent();
  }

  /**
   * 最小化窗口
   */
  public async minimizeWindow(): Promise<void> {
    const window = this.getCurrentWindow();
    await window.minimize();
  }

  /**
   * 最大化窗口
   */
  public async maximizeWindow(): Promise<void> {
    const window = this.getCurrentWindow();
    await window.maximize();
  }

  /**
   * 取消最大化窗口
   */
  public async unmaximizeWindow(): Promise<void> {
    const window = this.getCurrentWindow();
    await window.unmaximize();
  }

  /**
   * 关闭窗口
   */
  public async closeWindow(): Promise<void> {
    const window = this.getCurrentWindow();
    await window.close();
  }

  /**
   * 切换窗口最大化状态
   */
  public async toggleMaximize(): Promise<void> {
    const window = this.getCurrentWindow();
    const isMaximized = await window.isMaximized();

    if (isMaximized) {
      await window.unmaximize();
    } else {
      await window.maximize();
    }
  }

  /**
   * 设置窗口标题
   */
  public async setWindowTitle(title: string): Promise<void> {
    const window = this.getCurrentWindow();
    await window.setTitle(title);
  }

  /**
   * 获取窗口是否最大化
   */
  public async isWindowMaximized(): Promise<boolean> {
    const window = this.getCurrentWindow();
    return window.isMaximized();
  }

  /**
   * 获取窗口是否最小化
   */
  public async isWindowMinimized(): Promise<boolean> {
    const window = this.getCurrentWindow();
    return window.isMinimized();
  }

  // ===== 批量操作 =====

  /**
   * 批量发送命令
   */
  public async batchInvoke<T>(commands: TauriCommand[]): Promise<T[]> {
    const promises = commands.map((cmd) =>
      this.invokeCommand<T>(cmd.command, cmd.args, { timeout: cmd.timeout })
    );

    return Promise.all(promises);
  }

  /**
   * 并发发送命令（带错误处理）
   */
  public async concurrentInvoke<T>(
    commands: TauriCommand[],
    options: { failFast?: boolean } = {}
  ): Promise<Array<T | Error>> {
    const { failFast = false } = options;

    const promises = commands.map(async (cmd) => {
      try {
        return await this.invokeCommand<T>(cmd.command, cmd.args, { timeout: cmd.timeout });
      } catch (error) {
        if (failFast) {
          throw error;
        }
        return error as Error;
      }
    });

    return Promise.all(promises);
  }

  // ===== 健康检查 =====

  /**
   * 检查后端连接状态
   */
  public async healthCheck(): Promise<boolean> {
    try {
      const response = await this.invokeCommand<ApiResponse<string>>("ping", undefined, {
        timeout: 5000,
        retries: 1,
      });
      this.handleApiResponse(response);
      return true;
    } catch (error) {
      console.warn("Backend health check failed:", error);
      return false;
    }
  }

  /**
   * 获取服务状态
   */
  public getServiceStatus(): {
    initialized: boolean;
    activeListeners: number;
    listenersByEvent: Record<string, number>;
  } {
    const listenersByEvent: Record<string, number> = {};
    let totalListeners = 0;

    this.eventListeners.forEach((listeners, event) => {
      listenersByEvent[event] = listeners.length;
      totalListeners += listeners.length;
    });

    return {
      initialized: this.isInitialized,
      activeListeners: totalListeners,
      listenersByEvent,
    };
  }

  // ===== 工具方法 =====

  /**
   * 创建安全的调用包装器
   */
  public createSafeInvoker<T>(
    command: string,
    defaultValue: T,
    options: { timeout?: number; retries?: number; logErrors?: boolean } = {}
  ) {
    const { timeout, retries, logErrors = true } = options;

    return async (args?: Record<string, any>): Promise<T> => {
      try {
        return await this.invokeCommand<T>(command, args, { timeout, retries });
      } catch (error) {
        if (logErrors) {
          console.error(`Safe invoker for '${command}' failed:`, error);
        }
        return defaultValue;
      }
    };
  }

  /**
   * 创建事件监听器管理器
   */
  public createEventManager() {
    const listeners: (() => void)[] = [];

    return {
      listen: async <T>(event: string, callback: EventCallback<T>) => {
        const unlisten = await this.addEventListener(event, callback);
        listeners.push(unlisten);
        return unlisten;
      },

      cleanup: () => {
        listeners.forEach((unlisten) => unlisten());
        listeners.length = 0;
      },
    };
  }
}

// 导出单例实例
export const tauriService = TauriService.getInstance();

// 导出便利函数
export const {
  getConversations,
  createConversation,
  getConversationMessages,
  sendMessage,
  clearConversation,
  deleteConversation,
  logError,
  logWarning,
  logInfo,
  addEventListener,
  removeEventListener,
  emitEvent,
  onChatResponse,
  onSystemEvent,
  healthCheck,
} = tauriService;

// 导出类型
export type { EventListener };
