/**
 * 配置管理服务
 * 提供与后端配置API的统一接口，支持获取、保存和重置应用配置
 */

import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse, AppConfig } from "$lib/types";

// 配置API响应类型
interface ConfigApiResponse<T> extends ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: string;
    timestamp: string;
  };
}

// 配置服务错误类型
export class ConfigServiceError extends Error {
  constructor(
    message: string,
    public code: string = "CONFIG_ERROR",
    public details?: string
  ) {
    super(message);
    this.name = "ConfigServiceError";
  }
}

// 配置变更事件
interface ConfigChangeEvent {
  type: "loaded" | "saved" | "reset" | "error";
  config?: AppConfig;
  error?: string;
  timestamp: number;
}

class ConfigService {
  private static instance: ConfigService;
  private currentConfig: AppConfig | null = null;
  private isLoading = false;
  private listeners: ((event: ConfigChangeEvent) => void)[] = [];

  private constructor() {}

  public static getInstance(): ConfigService {
    if (!ConfigService.instance) {
      ConfigService.instance = new ConfigService();
    }
    return ConfigService.instance;
  }

  /**
   * 添加配置变更监听器
   */
  public addListener(listener: (event: ConfigChangeEvent) => void): () => void {
    this.listeners.push(listener);
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * 触发配置变更事件
   */
  private emitEvent(event: ConfigChangeEvent): void {
    this.listeners.forEach((listener) => {
      try {
        listener(event);
      } catch (error) {
        console.error("配置监听器执行错误:", error);
      }
    });
  }

  /**
   * 处理API响应
   */
  private handleApiResponse<T>(response: ConfigApiResponse<T>): T {
    if (!response.success) {
      const error = new ConfigServiceError(
        response.error?.message || "配置操作失败",
        response.error?.code || "CONFIG_ERROR",
        response.error?.details
      );
      throw error;
    }

    if (response.data === undefined || response.data === null) {
      throw new ConfigServiceError("配置数据为空", "EMPTY_DATA");
    }

    return response.data;
  }

  /**
   * 获取应用配置
   */
  public async getConfig(useCache: boolean = true): Promise<AppConfig> {
    // 如果使用缓存且已有配置，直接返回
    if (useCache && this.currentConfig && !this.isLoading) {
      return this.currentConfig;
    }

    // 避免重复加载
    if (this.isLoading) {
      throw new ConfigServiceError("配置正在加载中", "LOADING_IN_PROGRESS");
    }

    this.isLoading = true;

    try {
      console.log("正在从后端获取配置...");

      const response = await invoke<ConfigApiResponse<AppConfig>>("get_app_config");
      const config = this.handleApiResponse(response);

      this.currentConfig = config;

      this.emitEvent({
        type: "loaded",
        config,
        timestamp: Date.now(),
      });

      console.log("配置加载成功:", config);
      return config;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "获取配置失败";
      console.error("获取配置失败:", error);

      this.emitEvent({
        type: "error",
        error: errorMessage,
        timestamp: Date.now(),
      });

      throw error instanceof ConfigServiceError
        ? error
        : new ConfigServiceError(errorMessage, "GET_CONFIG_ERROR");
    } finally {
      this.isLoading = false;
    }
  }

  /**
   * 保存应用配置
   */
  public async saveConfig(config: AppConfig): Promise<void> {
    try {
      console.log("正在保存配置到后端...", config);

      const response = await invoke<ConfigApiResponse<void>>("save_app_config", { config });
      this.handleApiResponse(response);

      // 更新本地缓存
      this.currentConfig = { ...config };

      this.emitEvent({
        type: "saved",
        config: this.currentConfig,
        timestamp: Date.now(),
      });

      console.log("配置保存成功");
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "保存配置失败";
      console.error("保存配置失败:", error);

      this.emitEvent({
        type: "error",
        error: errorMessage,
        timestamp: Date.now(),
      });

      throw error instanceof ConfigServiceError
        ? error
        : new ConfigServiceError(errorMessage, "SAVE_CONFIG_ERROR");
    }
  }

  /**
   * 重置应用配置到默认值
   */
  public async resetConfig(): Promise<AppConfig> {
    try {
      console.log("正在重置配置到默认值...");

      const response = await invoke<ConfigApiResponse<AppConfig>>("reset_app_config");
      const config = this.handleApiResponse(response);

      // 更新本地缓存
      this.currentConfig = config;

      this.emitEvent({
        type: "reset",
        config,
        timestamp: Date.now(),
      });

      console.log("配置重置成功:", config);
      return config;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "重置配置失败";
      console.error("重置配置失败:", error);

      this.emitEvent({
        type: "error",
        error: errorMessage,
        timestamp: Date.now(),
      });

      throw error instanceof ConfigServiceError
        ? error
        : new ConfigServiceError(errorMessage, "RESET_CONFIG_ERROR");
    }
  }

  /**
   * 部分更新配置
   */
  public async updateConfig(updates: Partial<AppConfig>): Promise<AppConfig> {
    if (!this.currentConfig) {
      await this.getConfig();
    }

    if (!this.currentConfig) {
      throw new ConfigServiceError("无法获取当前配置", "NO_CURRENT_CONFIG");
    }

    // 深度合并配置
    const newConfig = this.deepMerge(this.currentConfig, updates);
    await this.saveConfig(newConfig);

    return newConfig;
  }

  /**
   * 获取特定配置节
   */
  public async getConfigSection<K extends keyof AppConfig>(
    section: K,
    useCache: boolean = true
  ): Promise<AppConfig[K]> {
    const config = await this.getConfig(useCache);
    return config[section];
  }

  /**
   * 更新特定配置节
   */
  public async updateConfigSection<K extends keyof AppConfig>(
    section: K,
    updates: Partial<AppConfig[K]>
  ): Promise<AppConfig[K]> {
    const fullUpdate = { [section]: updates } as Partial<AppConfig>;
    const newConfig = await this.updateConfig(fullUpdate);
    return newConfig[section];
  }

  /**
   * 验证配置结构
   */
  public validateConfig(config: any): config is AppConfig {
    if (!config || typeof config !== "object") {
      return false;
    }

    const requiredSections = ["ai_model", "voice", "ui", "database", "app_behavior"];
    for (const section of requiredSections) {
      if (!config[section] || typeof config[section] !== "object") {
        return false;
      }
    }

    return true;
  }

  /**
   * 获取配置摘要
   */
  public async getConfigSummary(): Promise<{
    isLoaded: boolean;
    hasChanges: boolean;
    lastModified: number | null;
    sections: string[];
  }> {
    return {
      isLoaded: this.currentConfig !== null,
      hasChanges: false, // TODO: 实现变更检测
      lastModified: this.currentConfig ? Date.now() : null,
      sections: this.currentConfig ? Object.keys(this.currentConfig) : [],
    };
  }

  /**
   * 清除缓存
   */
  public clearCache(): void {
    this.currentConfig = null;
    console.log("配置缓存已清除");
  }

  /**
   * 导出配置
   */
  public async exportConfig(): Promise<string> {
    const config = await this.getConfig();
    return JSON.stringify(config, null, 2);
  }

  /**
   * 导入配置
   */
  public async importConfig(configJson: string): Promise<AppConfig> {
    try {
      const config = JSON.parse(configJson);

      if (!this.validateConfig(config)) {
        throw new ConfigServiceError("配置格式无效", "INVALID_CONFIG_FORMAT");
      }

      await this.saveConfig(config);
      return config;
    } catch (error) {
      if (error instanceof SyntaxError) {
        throw new ConfigServiceError("配置JSON格式错误", "INVALID_JSON");
      }
      throw error;
    }
  }

  /**
   * 深度合并对象
   */
  private deepMerge<T>(target: T, source: Partial<T>): T {
    const result = { ...target };

    for (const key in source) {
      if (source[key] !== undefined) {
        if (
          typeof source[key] === "object" &&
          source[key] !== null &&
          !Array.isArray(source[key]) &&
          typeof result[key] === "object" &&
          result[key] !== null &&
          !Array.isArray(result[key])
        ) {
          result[key] = this.deepMerge(result[key], source[key] as any);
        } else {
          result[key] = source[key] as any;
        }
      }
    }

    return result;
  }

  /**
   * 获取默认配置（不调用后端）
   */
  public getDefaultConfig(): AppConfig {
    return {
      config_path: "config.yaml",
      ai_model: {
        model_type: "candle",
        model_name: "microsoft/DialoGPT-medium",
        server_url: "http://localhost",
        server_port: "11434",
        system_prompt: "你是一个友好、乐于助人的AI助手，使用中文回答问题。",
        candle_model_id: "microsoft/DialoGPT-medium",
        candle_revision: "main",
        candle_use_flash_attn: false,
      },
      voice: {
        enabled: false,
        model_path: "model/vosk-model-small-cn-0.22",
        timeout_seconds: 15,
      },
      ui: {
        theme: "light",
        language: "zh-CN",
      },
      database: {
        enabled: true,
        path: "database/chat_database.db",
      },
      app_behavior: {
        log_level: "info",
        default_conversation_title: "新对话",
        welcome_message: "欢迎使用聊天应用!",
        message_chunk_buffer_size: 2,
        message_chunk_send_interval_ms: 3,
        show_error_dialogs: true,
        auto_retry_failed_init: false,
      },
    };
  }
}

// 导出单例实例
export const configService = ConfigService.getInstance();

// 导出便利函数
export const {
  getConfig,
  saveConfig,
  resetConfig,
  updateConfig,
  getConfigSection,
  updateConfigSection,
  validateConfig,
  getConfigSummary,
  clearCache,
  exportConfig,
  importConfig,
  addListener,
} = configService;

// 导出类型
export type { ConfigChangeEvent };
