import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, SystemInfo, HealthStatus } from "$lib/types";

// ===== Store State Interface =====

interface SettingsStoreState {
  config: AppConfig | null;
  systemInfo: SystemInfo | null;
  healthStatus: HealthStatus | null;
  isLoading: boolean;
  error: string | null;
  isDirty: boolean; // 是否有未保存的更改
}

// ===== Initial State =====

const initialState: SettingsStoreState = {
  config: null,
  systemInfo: null,
  healthStatus: null,
  isLoading: false,
  error: null,
  isDirty: false,
};

// ===== Create Settings Store =====

function createSettingsStore() {
  const { subscribe, set, update } = writable<SettingsStoreState>(initialState);

  return {
    subscribe,

    // ===== 初始化和清理 =====

    async init() {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        // 加载应用配置
        await this.loadConfig();

        // 加载系统信息
        await this.loadSystemInfo();

        // 加载健康状态
        await this.loadHealthStatus();

        update((state) => ({ ...state, isLoading: false }));
        console.log("Settings store initialized successfully");
      } catch (error) {
        console.error("Failed to initialize settings store:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `初始化设置失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
      }
    },

    async destroy() {
      set(initialState);
    },

    // ===== 配置管理 =====

    async loadConfig() {
      try {
        const config: AppConfig = await invoke("get_app_config");
        update((state) => ({
          ...state,
          config,
          isDirty: false,
          error: null,
        }));
        console.log("Loaded app config:", config);
        return config;
      } catch (error) {
        console.error("Failed to load config:", error);
        update((state) => ({
          ...state,
          error: `加载配置失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    async saveConfig() {
      try {
        const state = this.getCurrentState();
        if (!state.config) {
          throw new Error("没有配置可保存");
        }

        update((s) => ({ ...s, isLoading: true, error: null }));

        await invoke("save_app_config", { config: state.config });

        update((s) => ({
          ...s,
          isLoading: false,
          isDirty: false,
        }));

        console.log("Config saved successfully");
        return true;
      } catch (error) {
        console.error("Failed to save config:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `保存配置失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    async resetConfig() {
      try {
        update((state) => ({ ...state, isLoading: true, error: null }));

        await invoke("reset_app_config");

        // 重新加载配置
        await this.loadConfig();

        update((state) => ({ ...state, isLoading: false, isDirty: false }));
        console.log("Config reset successfully");
        return true;
      } catch (error) {
        console.error("Failed to reset config:", error);
        update((state) => ({
          ...state,
          isLoading: false,
          error: `重置配置失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    // ===== 配置更新方法 =====

    updateUIConfig(updates: Partial<AppConfig["ui"]>) {
      update((state) => {
        if (!state.config) return state;

        return {
          ...state,
          config: {
            ...state.config,
            ui: {
              ...state.config.ui,
              ...updates,
            },
          },
          isDirty: true,
        };
      });
    },

    updateAppBehaviorConfig(updates: Partial<AppConfig["app_behavior"]>) {
      update((state) => {
        if (!state.config) return state;

        return {
          ...state,
          config: {
            ...state.config,
            app_behavior: {
              ...state.config.app_behavior,
              ...updates,
            },
          },
          isDirty: true,
        };
      });
    },

    updateLLMConfig(updates: Partial<AppConfig["llm"]>) {
      update((state) => {
        if (!state.config) return state;

        return {
          ...state,
          config: {
            ...state.config,
            llm: {
              ...state.config.llm,
              ...updates,
            },
          },
          isDirty: true,
        };
      });
    },

    updateVoiceConfig(updates: Partial<AppConfig["voice"]>) {
      update((state) => {
        if (!state.config) return state;

        return {
          ...state,
          config: {
            ...state.config,
            voice: {
              ...state.config.voice,
              ...updates,
            },
          },
          isDirty: true,
        };
      });
    },

    // ===== 系统信息 =====

    async loadSystemInfo() {
      try {
        const systemInfo: SystemInfo = await invoke("get_system_info");
        update((state) => ({
          ...state,
          systemInfo,
        }));
        console.log("Loaded system info:", systemInfo);
        return systemInfo;
      } catch (error) {
        console.error("Failed to load system info:", error);
        update((state) => ({
          ...state,
          error: `加载系统信息失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    // ===== 健康状态 =====

    async loadHealthStatus() {
      try {
        const healthStatus: HealthStatus = await invoke("get_health_status");
        update((state) => ({
          ...state,
          healthStatus,
        }));
        console.log("Loaded health status:", healthStatus);
        return healthStatus;
      } catch (error) {
        console.error("Failed to load health status:", error);
        update((state) => ({
          ...state,
          error: `加载健康状态失败: ${error instanceof Error ? error.message : String(error)}`,
        }));
        throw error;
      }
    },

    async refreshHealthStatus() {
      return this.loadHealthStatus();
    },

    // ===== 文件操作 =====

    async exportConfigFile() {
      try {
        const filePath = await invoke("export_config_file");
        console.log("Config exported to:", filePath);
        return filePath;
      } catch (error) {
        console.error("Failed to export config:", error);
        throw error;
      }
    },

    async importConfigFile() {
      try {
        const result = await invoke("import_config_file");
        if (result) {
          // 重新加载配置
          await this.loadConfig();
          console.log("Config imported successfully");
        }
        return result;
      } catch (error) {
        console.error("Failed to import config:", error);
        throw error;
      }
    },

    async exportChatHistory() {
      try {
        const filePath = await invoke("export_chat_history");
        console.log("Chat history exported to:", filePath);
        return filePath;
      } catch (error) {
        console.error("Failed to export chat history:", error);
        throw error;
      }
    },

    // ===== 文件夹/文件选择 =====

    async selectVoiceModelFolder() {
      try {
        const folderPath = await invoke("select_voice_model_folder");
        if (folderPath) {
          this.updateVoiceConfig({ model_path: folderPath });
        }
        return folderPath;
      } catch (error) {
        console.error("Failed to select voice model folder:", error);
        throw error;
      }
    },

    async selectDatabaseFile() {
      try {
        const filePath = await invoke("select_database_file");
        return filePath;
      } catch (error) {
        console.error("Failed to select database file:", error);
        throw error;
      }
    },

    async createDatabaseFile() {
      try {
        const filePath = await invoke("create_database_file");
        return filePath;
      } catch (error) {
        console.error("Failed to create database file:", error);
        throw error;
      }
    },

    // ===== 对话框方法 =====

    async showInfoDialog(title: string, message: string) {
      try {
        await invoke("show_info_dialog", { title, message });
      } catch (error) {
        console.error("Failed to show info dialog:", error);
        throw error;
      }
    },

    async showWarningDialog(title: string, message: string) {
      try {
        await invoke("show_warning_dialog", { title, message });
      } catch (error) {
        console.error("Failed to show warning dialog:", error);
        throw error;
      }
    },

    async showErrorDialog(title: string, message: string) {
      try {
        await invoke("show_error_dialog", { title, message });
      } catch (error) {
        console.error("Failed to show error dialog:", error);
        throw error;
      }
    },

    async showConfirmDialog(title: string, message: string): Promise<boolean> {
      try {
        return await invoke("show_confirm_dialog", { title, message });
      } catch (error) {
        console.error("Failed to show confirm dialog:", error);
        throw error;
      }
    },

    async showAskDialog(title: string, message: string): Promise<string> {
      try {
        return await invoke("show_ask_dialog", { title, message });
      } catch (error) {
        console.error("Failed to show ask dialog:", error);
        throw error;
      }
    },

    // ===== 工具方法 =====

    getCurrentState(): SettingsStoreState {
      let currentState: SettingsStoreState = initialState;
      const unsubscribe = subscribe((state) => {
        currentState = state;
      });
      unsubscribe();
      return currentState;
    },

    markDirty() {
      update((state) => ({ ...state, isDirty: true }));
    },

    clearError() {
      update((state) => ({ ...state, error: null }));
    },

    setLoading(loading: boolean) {
      update((state) => ({ ...state, isLoading: loading }));
    },
  };
}

// ===== 导出 Store 实例 =====

export const settingsStore = createSettingsStore();

// ===== 导出派生 Store =====

export const config = derived(settingsStore, ($store) => $store.config);
export const systemInfo = derived(settingsStore, ($store) => $store.systemInfo);
export const healthStatus = derived(settingsStore, ($store) => $store.healthStatus);
export const settingsLoading = derived(settingsStore, ($store) => $store.isLoading);
export const settingsError = derived(settingsStore, ($store) => $store.error);
export const isDirty = derived(settingsStore, ($store) => $store.isDirty);

// UI相关派生状态
export const uiConfig = derived(config, ($config) => $config?.ui);
export const themeConfig = derived(uiConfig, ($ui) => $ui?.theme || "auto");
export const languageConfig = derived(uiConfig, ($ui) => $ui?.language || "zh-CN");
export const sidebarWidth = derived(uiConfig, ($ui) => $ui?.sidebar_width || "280px");

// LLM相关派生状态
export const llmConfig = derived(config, ($config) => $config?.llm);
export const modelPath = derived(llmConfig, ($llm) => $llm?.model_path || "");
export const temperature = derived(llmConfig, ($llm) => $llm?.temperature || 0.7);
export const maxTokens = derived(llmConfig, ($llm) => $llm?.max_tokens || 2048);

// 语音相关派生状态
export const voiceConfig = derived(config, ($config) => $config?.voice);
export const voiceEnabled = derived(voiceConfig, ($voice) => $voice?.enabled || false);
export const voiceModelPath = derived(voiceConfig, ($voice) => $voice?.model_path || "");
export const voiceLanguage = derived(voiceConfig, ($voice) => $voice?.language || "zh-CN");

// 应用行为派生状态
export const appBehaviorConfig = derived(config, ($config) => $config?.app_behavior);
export const messageChunkBufferSize = derived(
  appBehaviorConfig,
  ($behavior) => $behavior?.message_chunk_buffer_size || 50
);
export const messageChunkSendInterval = derived(
  appBehaviorConfig,
  ($behavior) => $behavior?.message_chunk_send_interval_ms || 100
);

// 健康状态派生
export const isDatabaseHealthy = derived(healthStatus, ($health) => $health?.database || false);
export const isLLMHealthy = derived(healthStatus, ($health) => $health?.llm || false);
export const isVoiceHealthy = derived(healthStatus, ($health) => $health?.voice || false);
export const isOverallHealthy = derived(healthStatus, ($health) => $health?.overall || false);
