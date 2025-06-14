import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ask, confirm, message } from "@tauri-apps/plugin-dialog";

// 类型定义
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: string;
    timestamp: number;
  };
}

interface AppConfig {
  config_path: string;
  ai_model: {
    model_type: string;
    model_name: string;
    server_url: string;
    server_port: number;
    system_prompt: string;
    candle_model_id?: string;
    candle_revision?: string;
    candle_use_flash_attn: boolean;
  };
  voice: {
    enabled: boolean;
    model_path: string;
    timeout_seconds: number;
  };
  ui: {
    theme: string;
    language: string;
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

// DOM 元素引用
class SettingsElements {
  // AI 模型配置
  modelType: HTMLSelectElement;
  modelName: HTMLInputElement;
  serverUrl: HTMLInputElement;
  serverPort: HTMLInputElement;
  systemPrompt: HTMLTextAreaElement;
  useFlashAttn: HTMLInputElement;

  // 语音配置
  voiceEnabled: HTMLInputElement;
  voiceModelPath: HTMLInputElement;
  voiceTimeout: HTMLInputElement;
  voiceTimeoutValue: HTMLSpanElement;

  // UI 配置
  uiTheme: HTMLSelectElement;
  uiLanguage: HTMLSelectElement;

  // 数据库配置
  databaseEnabled: HTMLInputElement;
  databasePath: HTMLInputElement;

  // 应用行为配置
  logLevel: HTMLSelectElement;
  defaultTitle: HTMLInputElement;
  welcomeMessage: HTMLTextAreaElement;
  bufferSize: HTMLInputElement;
  sendInterval: HTMLInputElement;
  showErrorDialogs: HTMLInputElement;
  autoRetry: HTMLInputElement;

  // 按钮
  saveBtn: HTMLButtonElement;
  resetBtn: HTMLButtonElement;
  testBtn: HTMLButtonElement;
  exportBtn: HTMLButtonElement;
  importBtn: HTMLButtonElement;
  closeBtn: HTMLButtonElement;

  constructor() {
    // AI 模型配置
    this.modelType = this.getElement("model-type") as HTMLSelectElement;
    this.modelName = this.getElement("model-name") as HTMLInputElement;
    this.serverUrl = this.getElement("server-url") as HTMLInputElement;
    this.serverPort = this.getElement("server-port") as HTMLInputElement;
    this.systemPrompt = this.getElement("system-prompt") as HTMLTextAreaElement;
    this.useFlashAttn = this.getElement("use-flash-attn") as HTMLInputElement;

    // 语音配置
    this.voiceEnabled = this.getElement("voice-enabled") as HTMLInputElement;
    this.voiceModelPath = this.getElement(
      "voice-model-path",
    ) as HTMLInputElement;
    this.voiceTimeout = this.getElement("voice-timeout") as HTMLInputElement;
    this.voiceTimeoutValue = this.getElement(
      "voice-timeout-value",
    ) as HTMLSpanElement;

    // UI 配置
    this.uiTheme = this.getElement("ui-theme") as HTMLSelectElement;
    this.uiLanguage = this.getElement("ui-language") as HTMLSelectElement;

    // 数据库配置
    this.databaseEnabled = this.getElement(
      "database-enabled",
    ) as HTMLInputElement;
    this.databasePath = this.getElement("database-path") as HTMLInputElement;

    // 应用行为配置
    this.logLevel = this.getElement("log-level") as HTMLSelectElement;
    this.defaultTitle = this.getElement("default-title") as HTMLInputElement;
    this.welcomeMessage = this.getElement(
      "welcome-message",
    ) as HTMLTextAreaElement;
    this.bufferSize = this.getElement("buffer-size") as HTMLInputElement;
    this.sendInterval = this.getElement("send-interval") as HTMLInputElement;
    this.showErrorDialogs = this.getElement(
      "show-error-dialogs",
    ) as HTMLInputElement;
    this.autoRetry = this.getElement("auto-retry") as HTMLInputElement;

    // 按钮
    this.saveBtn = this.getElement("save-btn") as HTMLButtonElement;
    this.resetBtn = this.getElement("reset-btn") as HTMLButtonElement;
    this.testBtn = this.getElement("test-btn") as HTMLButtonElement;
    this.exportBtn = this.getElement("export-btn") as HTMLButtonElement;
    this.importBtn = this.getElement("import-btn") as HTMLButtonElement;
    this.closeBtn = this.getElement("close-btn") as HTMLButtonElement;
  }

  private getElement(id: string): HTMLElement {
    const element = document.getElementById(id);
    if (!element) {
      throw new Error(`Element with id '${id}' not found`);
    }
    return element;
  }
}

class EnhancedSettingsManager {
  private elements: SettingsElements;
  private currentConfig: AppConfig | null = null;
  private isDirty = false;

  constructor() {
    this.elements = new SettingsElements();
    this.init();
  }

  private async init() {
    try {
      await this.loadSettings();
      this.setupEventListeners();
      this.showMessage("设置界面已加载", "success");
    } catch (error) {
      console.error("初始化设置失败:", error);
      this.showMessage("初始化设置失败: " + error, "error");
    }
  }

  private async loadSettings() {
    try {
      const response = await invoke<ApiResponse<AppConfig>>("get_app_config");

      if (response.success && response.data) {
        this.currentConfig = response.data;
        this.populateForm(response.data);
        this.isDirty = false;
        this.updateSaveButton();
      } else {
        throw new Error(response.error?.message || "未知错误");
      }
    } catch (error) {
      console.error("加载设置失败:", error);
      throw error;
    }
  }

  private populateForm(config: AppConfig) {
    // AI 模型配置
    this.elements.modelType.value = config.ai_model.model_type;
    this.elements.modelName.value = config.ai_model.model_name;
    this.elements.serverUrl.value = config.ai_model.server_url;
    this.elements.serverPort.value = config.ai_model.server_port.toString();
    this.elements.systemPrompt.value = config.ai_model.system_prompt;
    this.elements.useFlashAttn.checked = config.ai_model.candle_use_flash_attn;

    // 语音配置
    this.elements.voiceEnabled.checked = config.voice.enabled;
    this.elements.voiceModelPath.value = config.voice.model_path;
    this.elements.voiceTimeout.value = config.voice.timeout_seconds.toString();
    this.elements.voiceTimeoutValue.textContent =
      config.voice.timeout_seconds.toString() + "s";

    // UI 配置
    this.elements.uiTheme.value = config.ui.theme;
    this.elements.uiLanguage.value = config.ui.language;

    // 数据库配置
    this.elements.databaseEnabled.checked = config.database.enabled;
    this.elements.databasePath.value = config.database.path;

    // 应用行为配置
    this.elements.logLevel.value = config.app_behavior.log_level;
    this.elements.defaultTitle.value =
      config.app_behavior.default_conversation_title;
    this.elements.welcomeMessage.value = config.app_behavior.welcome_message;
    this.elements.bufferSize.value =
      config.app_behavior.message_chunk_buffer_size.toString();
    this.elements.sendInterval.value =
      config.app_behavior.message_chunk_send_interval_ms.toString();
    this.elements.showErrorDialogs.checked =
      config.app_behavior.show_error_dialogs;
    this.elements.autoRetry.checked =
      config.app_behavior.auto_retry_failed_init;

    // 根据模型类型显示/隐藏相关选项
    this.toggleModelTypeOptions();
    this.toggleVoiceOptions();
    this.toggleDatabaseOptions();
  }

  private setupEventListeners() {
    // 监听所有输入变化
    const inputs = [
      this.elements.modelType,
      this.elements.modelName,
      this.elements.serverUrl,
      this.elements.serverPort,
      this.elements.systemPrompt,
      this.elements.useFlashAttn,
      this.elements.voiceEnabled,
      this.elements.voiceModelPath,
      this.elements.voiceTimeout,
      this.elements.uiTheme,
      this.elements.uiLanguage,
      this.elements.databaseEnabled,
      this.elements.databasePath,
      this.elements.logLevel,
      this.elements.defaultTitle,
      this.elements.welcomeMessage,
      this.elements.bufferSize,
      this.elements.sendInterval,
      this.elements.showErrorDialogs,
      this.elements.autoRetry,
    ];

    inputs.forEach((input) => {
      input.addEventListener("input", () => this.markDirty());
      input.addEventListener("change", () => this.markDirty());
    });

    // 特殊事件监听
    this.elements.voiceTimeout.addEventListener("input", (e) => {
      const value = (e.target as HTMLInputElement).value;
      this.elements.voiceTimeoutValue.textContent = value + "s";
    });

    this.elements.modelType.addEventListener("change", () =>
      this.toggleModelTypeOptions(),
    );
    this.elements.voiceEnabled.addEventListener("change", () =>
      this.toggleVoiceOptions(),
    );
    this.elements.databaseEnabled.addEventListener("change", () =>
      this.toggleDatabaseOptions(),
    );

    // 按钮事件
    this.elements.saveBtn.addEventListener("click", () => this.saveSettings());
    this.elements.resetBtn.addEventListener("click", () =>
      this.resetSettings(),
    );
    this.elements.testBtn.addEventListener("click", () =>
      this.testConnection(),
    );
    this.elements.exportBtn.addEventListener("click", () =>
      this.exportSettings(),
    );
    this.elements.importBtn.addEventListener("click", () =>
      this.importSettings(),
    );
    this.elements.closeBtn.addEventListener("click", () => this.closeWindow());

    // 快捷键
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.key === "s") {
        e.preventDefault();
        this.saveSettings();
      }
      if (e.key === "Escape") {
        this.closeWindow();
      }
    });

    // 窗口关闭前检查
    window.addEventListener("beforeunload", (e) => {
      if (this.isDirty) {
        e.preventDefault();
        e.returnValue = "您有未保存的更改，确定要离开吗？";
      }
    });
  }

  private markDirty() {
    this.isDirty = true;
    this.updateSaveButton();
  }

  private updateSaveButton() {
    this.elements.saveBtn.disabled = !this.isDirty;
    this.elements.saveBtn.textContent = this.isDirty ? "保存更改" : "已保存";
  }

  private toggleModelTypeOptions() {
    const isOllama = this.elements.modelType.value === "ollama";
    const ollamaOptions = document.querySelectorAll(".ollama-option");
    const candleOptions = document.querySelectorAll(".candle-option");

    ollamaOptions.forEach((el) => {
      (el as HTMLElement).style.display = isOllama ? "block" : "none";
    });

    candleOptions.forEach((el) => {
      (el as HTMLElement).style.display = isOllama ? "none" : "block";
    });
  }

  private toggleVoiceOptions() {
    const enabled = this.elements.voiceEnabled.checked;
    const voiceOptions = document.querySelectorAll(".voice-option");

    voiceOptions.forEach((el) => {
      const element = el as HTMLElement;
      element.style.opacity = enabled ? "1" : "0.5";
      const inputs = element.querySelectorAll("input, select, textarea");
      inputs.forEach((input) => {
        (input as HTMLInputElement).disabled = !enabled;
      });
    });
  }

  private toggleDatabaseOptions() {
    const enabled = this.elements.databaseEnabled.checked;
    const dbOptions = document.querySelectorAll(".database-option");

    dbOptions.forEach((el) => {
      const element = el as HTMLElement;
      element.style.opacity = enabled ? "1" : "0.5";
      const inputs = element.querySelectorAll("input, select, textarea");
      inputs.forEach((input) => {
        (input as HTMLInputElement).disabled = !enabled;
      });
    });
  }

  private async saveSettings() {
    if (!this.currentConfig) {
      this.showMessage("没有可保存的配置", "error");
      return;
    }

    try {
      this.elements.saveBtn.disabled = true;
      this.elements.saveBtn.textContent = "保存中...";

      const updatedConfig = this.collectFormData();

      // 验证配置
      const validationError = this.validateConfig(updatedConfig);
      if (validationError) {
        this.showMessage(validationError, "error");
        return;
      }

      const response = await invoke<ApiResponse<void>>("save_app_config", {
        config: updatedConfig,
      });

      if (response.success) {
        this.currentConfig = updatedConfig;
        this.isDirty = false;
        this.updateSaveButton();
        this.showMessage("设置保存成功！", "success");
      } else {
        throw new Error(response.error?.message || "保存失败");
      }
    } catch (error) {
      console.error("保存设置失败:", error);
      this.showMessage("保存设置失败: " + error, "error");
    } finally {
      this.elements.saveBtn.disabled = false;
      if (this.isDirty) {
        this.elements.saveBtn.textContent = "保存更改";
      }
    }
  }

  private collectFormData(): AppConfig {
    if (!this.currentConfig) {
      throw new Error("当前配置为空");
    }

    return {
      ...this.currentConfig,
      ai_model: {
        ...this.currentConfig.ai_model,
        model_type: this.elements.modelType.value,
        model_name: this.elements.modelName.value,
        server_url: this.elements.serverUrl.value,
        server_port: parseInt(this.elements.serverPort.value),
        system_prompt: this.elements.systemPrompt.value,
        candle_use_flash_attn: this.elements.useFlashAttn.checked,
      },
      voice: {
        ...this.currentConfig.voice,
        enabled: this.elements.voiceEnabled.checked,
        model_path: this.elements.voiceModelPath.value,
        timeout_seconds: parseInt(this.elements.voiceTimeout.value),
      },
      ui: {
        ...this.currentConfig.ui,
        theme: this.elements.uiTheme.value,
        language: this.elements.uiLanguage.value,
      },
      database: {
        ...this.currentConfig.database,
        enabled: this.elements.databaseEnabled.checked,
        path: this.elements.databasePath.value,
      },
      app_behavior: {
        ...this.currentConfig.app_behavior,
        log_level: this.elements.logLevel.value,
        default_conversation_title: this.elements.defaultTitle.value,
        welcome_message: this.elements.welcomeMessage.value,
        message_chunk_buffer_size: parseInt(this.elements.bufferSize.value),
        message_chunk_send_interval_ms: parseInt(
          this.elements.sendInterval.value,
        ),
        show_error_dialogs: this.elements.showErrorDialogs.checked,
        auto_retry_failed_init: this.elements.autoRetry.checked,
      },
    };
  }

  private validateConfig(config: AppConfig): string | null {
    // 验证服务器端口
    if (
      config.ai_model.server_port < 1 ||
      config.ai_model.server_port > 65535
    ) {
      return "服务器端口必须在 1-65535 之间";
    }

    // 验证服务器URL
    if (!config.ai_model.server_url.trim()) {
      return "服务器URL不能为空";
    }

    // 验证模型名称
    if (!config.ai_model.model_name.trim()) {
      return "模型名称不能为空";
    }

    // 验证语音超时时间
    if (
      config.voice.timeout_seconds < 1 ||
      config.voice.timeout_seconds > 300
    ) {
      return "语音超时时间必须在 1-300 秒之间";
    }

    // 验证缓冲区大小
    if (
      config.app_behavior.message_chunk_buffer_size < 1 ||
      config.app_behavior.message_chunk_buffer_size > 100
    ) {
      return "消息块缓冲区大小必须在 1-100 之间";
    }

    // 验证发送间隔
    if (
      config.app_behavior.message_chunk_send_interval_ms < 1 ||
      config.app_behavior.message_chunk_send_interval_ms > 10000
    ) {
      return "消息发送间隔必须在 1-10000 毫秒之间";
    }

    return null;
  }

  private async resetSettings() {
    const confirmed = await confirm(
      "确定要重置所有设置到默认值吗？这将丢失当前的所有配置。",
      { title: "重置设置", kind: "warning" },
    );

    if (!confirmed) return;

    try {
      this.elements.resetBtn.disabled = true;
      this.elements.resetBtn.textContent = "重置中...";

      const response = await invoke<ApiResponse<AppConfig>>("reset_app_config");

      if (response.success && response.data) {
        this.currentConfig = response.data;
        this.populateForm(response.data);
        this.isDirty = false;
        this.updateSaveButton();
        this.showMessage("设置已重置为默认值", "success");
      } else {
        throw new Error(response.error?.message || "重置失败");
      }
    } catch (error) {
      console.error("重置设置失败:", error);
      this.showMessage("重置设置失败: " + error, "error");
    } finally {
      this.elements.resetBtn.disabled = false;
      this.elements.resetBtn.textContent = "重置默认";
    }
  }

  private async testConnection() {
    try {
      this.elements.testBtn.disabled = true;
      this.elements.testBtn.textContent = "测试中...";

      // 简单的连接测试 - ping后端
      const response = await invoke<ApiResponse<string>>("ping");

      if (response.success) {
        this.showMessage("连接测试成功！后端响应正常", "success");
      } else {
        throw new Error(response.error?.message || "连接测试失败");
      }
    } catch (error) {
      console.error("连接测试失败:", error);
      this.showMessage("连接测试失败: " + error, "error");
    } finally {
      this.elements.testBtn.disabled = false;
      this.elements.testBtn.textContent = "测试连接";
    }
  }

  private async exportSettings() {
    if (!this.currentConfig) {
      this.showMessage("没有可导出的配置", "error");
      return;
    }

    try {
      const configJson = JSON.stringify(this.currentConfig, null, 2);
      const blob = new Blob([configJson], { type: "application/json" });
      const url = URL.createObjectURL(blob);

      const a = document.createElement("a");
      a.href = url;
      a.download = `chatbox-settings-${new Date().toISOString().split("T")[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      this.showMessage("设置已导出", "success");
    } catch (error) {
      console.error("导出设置失败:", error);
      this.showMessage("导出设置失败: " + error, "error");
    }
  }

  private async importSettings() {
    try {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".json";

      input.onchange = async (e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (!file) return;

        try {
          const text = await file.text();
          const importedConfig = JSON.parse(text) as AppConfig;

          // 验证导入的配置
          const validationError = this.validateConfig(importedConfig);
          if (validationError) {
            throw new Error("配置验证失败: " + validationError);
          }

          const confirmed = await confirm(
            "确定要导入这个配置吗？这将覆盖当前的所有设置。",
            { title: "导入设置", kind: "warning" },
          );

          if (confirmed) {
            this.currentConfig = importedConfig;
            this.populateForm(importedConfig);
            this.markDirty();
            this.showMessage("设置已导入，请保存以应用更改", "success");
          }
        } catch (error) {
          console.error("导入设置失败:", error);
          this.showMessage("导入设置失败: " + error, "error");
        }
      };

      input.click();
    } catch (error) {
      console.error("导入设置失败:", error);
      this.showMessage("导入设置失败: " + error, "error");
    }
  }

  private async closeWindow() {
    if (this.isDirty) {
      const confirmed = await ask("您有未保存的更改，确定要关闭窗口吗？", {
        title: "未保存的更改",
        kind: "warning",
      });

      if (!confirmed) return;
    }

    try {
      const webview = WebviewWindow.getCurrent();
      await webview.close();
    } catch (error) {
      console.error("关闭窗口失败:", error);
    }
  }

  private showMessage(
    messageText: string,
    type: "success" | "error" | "info" = "info",
  ) {
    // 移除现有消息
    const existingMessage = document.querySelector(".message-toast");
    if (existingMessage) {
      existingMessage.remove();
    }

    // 创建新消息
    const messageDiv = document.createElement("div");
    messageDiv.className = `message-toast ${type}`;
    messageDiv.textContent = messageText;

    // 样式
    messageDiv.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 12px 20px;
      border-radius: 8px;
      color: white;
      font-size: 14px;
      z-index: 1000;
      animation: slideInRight 0.3s ease-out;
      max-width: 350px;
      word-wrap: break-word;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    `;

    // 根据类型设置背景色
    switch (type) {
      case "success":
        messageDiv.style.background =
          "linear-gradient(135deg, #48bb78, #38a169)";
        break;
      case "error":
        messageDiv.style.background =
          "linear-gradient(135deg, #f56565, #e53e3e)";
        break;
      default:
        messageDiv.style.background =
          "linear-gradient(135deg, #4299e1, #3182ce)";
        break;
    }

    document.body.appendChild(messageDiv);

    // 自动移除
    setTimeout(() => {
      if (messageDiv.parentNode) {
        messageDiv.style.animation = "slideOutRight 0.3s ease-out";
        setTimeout(() => messageDiv.remove(), 300);
      }
    }, 3000);
  }
}

// 等待DOM加载完成后初始化
document.addEventListener("DOMContentLoaded", () => {
  new EnhancedSettingsManager();
  initializeTabs();
});

// Tab 功能初始化
function initializeTabs() {
  const tabs = document.querySelectorAll(".tab");
  const tabContents = document.querySelectorAll(".tab-content");

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const targetTab = tab.getAttribute("data-tab");

      if (targetTab) {
        // 移除所有活动状态
        tabs.forEach((t) => t.classList.remove("active"));
        tabContents.forEach((content) => content.classList.remove("active"));

        // 激活当前选中的标签页
        tab.classList.add("active");
        const targetContent = document.getElementById(targetTab);
        if (targetContent) {
          targetContent.classList.add("active");
        }
      }
    });
  });
}

// 添加CSS动画
const style = document.createElement("style");
style.textContent = `
  @keyframes slideInRight {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @keyframes slideOutRight {
    from { transform: translateX(0); opacity: 1; }
    to { transform: translateX(100%); opacity: 0; }
  }
`;
document.head.appendChild(style);
