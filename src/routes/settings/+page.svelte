<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { themeStore } from "$lib/stores/themeStore";
  import { configService, type ConfigChangeEvent } from "$lib/services/ConfigService";
  import type { AppConfig } from "$lib/types";
  import ConfigFieldGroup from "$lib/components/ConfigFieldGroup.svelte";
  import ConfigField from "$lib/components/ConfigField.svelte";
  import SettingsSection from "$lib/components/SettingsSection.svelte";

  // 配置状态
  let config: AppConfig | null = null;
  let originalConfig: AppConfig | null = null;
  let isLoading = false;
  let isSaving = false;
  let isResetting = false;
  let error: string = "";
  let successMessage: string = "";
  let hasChanges = false;
  let activeSection = "general";

  // 表单验证错误
  let validationErrors: Record<string, string> = {};

  // 配置变更监听器
  let configChangeUnsubscribe: (() => void) | null = null;

  // 导航项
  const navigationItems = [
    { id: "general", label: "通用设置", icon: "⚙️", description: "基本应用配置" },
    { id: "ai_model", label: "AI 模型", icon: "🤖", description: "AI 模型和推理设置" },
    { id: "voice", label: "语音识别", icon: "🎤", description: "语音输入和识别配置" },
    { id: "ui", label: "界面设置", icon: "🎨", description: "主题和界面偏好" },
    { id: "database", label: "数据存储", icon: "🗄️", description: "数据库和存储设置" },
    { id: "advanced", label: "高级设置", icon: "🔧", description: "应用行为和性能调优" },
    { id: "import_export", label: "导入导出", icon: "📁", description: "配置文件管理" },
  ];

  // AI 模型类型选项
  const modelTypeOptions = [
    { value: "candle", label: "Candle (本地推理)", description: "使用 Candle 进行本地 AI 推理" },
    { value: "ollama", label: "Ollama (外部服务)", description: "连接到 Ollama 服务" },
  ];

  // 主题选项
  const themeOptions = [
    { value: "light", label: "浅色主题", description: "明亮的界面风格" },
    { value: "dark", label: "深色主题", description: "护眼的暗色界面" },
    { value: "auto", label: "跟随系统", description: "根据系统设置自动切换" },
  ];

  // 语言选项
  const languageOptions = [
    { value: "zh-CN", label: "简体中文", description: "中文界面" },
    { value: "en-US", label: "English", description: "英文界面" },
    { value: "ja-JP", label: "日本語", description: "日语界面" },
  ];

  // 日志级别选项
  const logLevelOptions = [
    { value: "error", label: "错误", description: "仅记录错误信息" },
    { value: "warn", label: "警告", description: "记录警告和错误" },
    { value: "info", label: "信息", description: "记录一般信息（推荐）" },
    { value: "debug", label: "调试", description: "详细的调试信息" },
    { value: "trace", label: "追踪", description: "最详细的日志信息" },
  ];

  // Toast 通知函数
  function showToast(message: string, type: "success" | "error" | "warning" = "success") {
    if (type === "success") {
      successMessage = message;
      error = "";
    } else {
      error = message;
      successMessage = "";
    }

    setTimeout(() => {
      error = "";
      successMessage = "";
    }, 5000);
  }

  // 组件挂载时初始化
  onMount(async () => {
    await loadConfig();
    setupConfigListener();

    // 从 URL 参数获取默认选中的节
    const urlSection = $page.url.searchParams.get("section");
    if (urlSection && navigationItems.some((item) => item.id === urlSection)) {
      activeSection = urlSection;
    }
  });

  // 组件销毁时清理
  onDestroy(() => {
    if (configChangeUnsubscribe) {
      configChangeUnsubscribe();
    }
  });

  // 设置配置变更监听器
  function setupConfigListener() {
    configChangeUnsubscribe = configService.addListener((event: ConfigChangeEvent) => {
      switch (event.type) {
        case "loaded":
          if (event.config) {
            config = { ...event.config };
            originalConfig = { ...event.config };
            hasChanges = false;
            validationErrors = {};
          }
          break;
        case "saved":
          if (event.config) {
            originalConfig = { ...event.config };
            hasChanges = false;
            showToast("配置已保存", "success");
          }
          break;
        case "reset":
          if (event.config) {
            config = { ...event.config };
            originalConfig = { ...event.config };
            hasChanges = false;
            showToast("配置已重置", "success");
          }
          break;
        case "error":
          showToast(event.error || "配置操作失败", "error");
          break;
      }
    });
  }

  // 加载配置
  async function loadConfig() {
    isLoading = true;
    error = "";

    try {
      const loadedConfig = await configService.getConfig(false);
      config = { ...loadedConfig };
      originalConfig = { ...loadedConfig };
      hasChanges = false;
      validationErrors = {};
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "加载配置失败";
      showToast(errorMessage, "error");
      console.error("加载配置失败:", err);
    } finally {
      isLoading = false;
    }
  }

  // 保存配置
  async function saveConfig() {
    if (!config || !hasChanges) return;

    // 验证配置
    const errors = validateConfig(config);
    if (Object.keys(errors).length > 0) {
      validationErrors = errors;
      showToast("请修正配置错误后再保存", "error");
      return;
    }

    isSaving = true;
    error = "";

    try {
      await configService.saveConfig(config);

      // 如果主题发生变化，更新主题 store
      if (config.ui.theme !== originalConfig?.ui.theme) {
        themeStore.setMode(config.ui.theme as any);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "保存配置失败";
      showToast(errorMessage, "error");
      console.error("保存配置失败:", err);
    } finally {
      isSaving = false;
    }
  }

  // 重置配置
  async function resetConfig() {
    if (!confirm("确定要重置所有设置到默认值吗？此操作无法撤销。")) {
      return;
    }

    isResetting = true;
    error = "";

    try {
      const resetConfig = await configService.resetConfig();
      config = { ...resetConfig };
      validationErrors = {};

      // 重置主题
      if (resetConfig.ui.theme) {
        themeStore.setMode(resetConfig.ui.theme as any);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "重置配置失败";
      showToast(errorMessage, "error");
      console.error("重置配置失败:", err);
    } finally {
      isResetting = false;
    }
  }

  // 检测配置变更
  function detectChanges() {
    if (!config || !originalConfig) return false;

    const currentStr = JSON.stringify(config);
    const originalStr = JSON.stringify(originalConfig);
    hasChanges = currentStr !== originalStr;

    return hasChanges;
  }

  // 验证配置
  function validateConfig(config: AppConfig): Record<string, string> {
    const errors: Record<string, string> = {};

    // AI 模型验证
    if (!config.ai_model.model_name.trim()) {
      errors["ai_model.model_name"] = "模型名称不能为空";
    }

    if (!config.ai_model.server_url.trim()) {
      errors["ai_model.server_url"] = "服务器地址不能为空";
    } else if (!/^https?:\/\/.+/.test(config.ai_model.server_url)) {
      errors["ai_model.server_url"] = "请输入有效的 HTTP/HTTPS 地址";
    }

    const port = parseInt(config.ai_model.server_port);
    if (isNaN(port) || port < 1 || port > 65535) {
      errors["ai_model.server_port"] = "端口号必须在 1-65535 范围内";
    }

    // 语音配置验证
    if (config.voice.enabled && !config.voice.model_path.trim()) {
      errors["voice.model_path"] = "启用语音识别时，模型路径不能为空";
    }

    if (config.voice.timeout_seconds < 5 || config.voice.timeout_seconds > 300) {
      errors["voice.timeout_seconds"] = "超时时间必须在 5-300 秒范围内";
    }

    // 数据库配置验证
    if (config.database.enabled && !config.database.path.trim()) {
      errors["database.path"] = "启用数据库时，数据库路径不能为空";
    }

    // 应用行为验证
    if (!config.app_behavior.default_conversation_title.trim()) {
      errors["app_behavior.default_conversation_title"] = "默认对话标题不能为空";
    }

    if (
      config.app_behavior.message_chunk_buffer_size < 1 ||
      config.app_behavior.message_chunk_buffer_size > 100
    ) {
      errors["app_behavior.message_chunk_buffer_size"] = "消息缓冲区大小必须在 1-100 范围内";
    }

    if (
      config.app_behavior.message_chunk_send_interval_ms < 1 ||
      config.app_behavior.message_chunk_send_interval_ms > 10000
    ) {
      errors["app_behavior.message_chunk_send_interval_ms"] = "发送间隔必须在 1-10000 毫秒范围内";
    }

    return errors;
  }

  // 处理配置字段变更
  function handleFieldChange(path: string, value: any) {
    if (!config) return;

    // 使用路径设置嵌套值
    const pathParts = path.split(".");
    let current: any = config;

    for (let i = 0; i < pathParts.length - 1; i++) {
      current = current[pathParts[i]];
    }

    current[pathParts[pathParts.length - 1]] = value;

    // 触发响应式更新
    config = { ...config };

    // 检测变更
    detectChanges();

    // 清除相关的验证错误
    if (validationErrors[path]) {
      delete validationErrors[path];
      validationErrors = { ...validationErrors };
    }
  }

  // 导出配置
  async function exportConfig() {
    try {
      const configJson = await configService.exportConfig();
      const blob = new Blob([configJson], { type: "application/json" });
      const url = URL.createObjectURL(blob);

      const a = document.createElement("a");
      a.href = url;
      a.download = `chat-app-config-${new Date().toISOString().split("T")[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      showToast("配置已导出", "success");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "导出配置失败";
      showToast(errorMessage, "error");
    }
  }

  // 导入配置
  async function importConfig(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];

    if (!file) return;

    try {
      const text = await file.text();
      await configService.importConfig(text);
      await loadConfig(); // 重新加载配置
      showToast("配置已导入", "success");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "导入配置失败";
      showToast(errorMessage, "error");
    } finally {
      // 清除文件输入
      input.value = "";
    }
  }

  // 响应式检测配置变更
  $: if (config && originalConfig) {
    detectChanges();
  }
</script>

<svelte:head>
  <title>设置 - 聊天应用</title>
</svelte:head>

<div class="settings-page">
  <!-- 页面头部 -->
  <header class="page-header">
    <div class="header-content">
      <div class="header-info">
        <h1 class="page-title">⚙️ 应用设置</h1>
        <p class="page-description">配置您的聊天应用偏好和功能</p>
      </div>

      <div class="header-actions">
        {#if hasChanges}
          <button
            type="button"
            class="btn btn-secondary"
            disabled={isSaving || isResetting}
            on:click={() => loadConfig()}
          >
            取消更改
          </button>
        {/if}

        <button
          type="button"
          class="btn btn-danger"
          disabled={isLoading || isSaving || isResetting}
          on:click={resetConfig}
        >
          {#if isResetting}
            🔄 重置中...
          {:else}
            🔄 重置默认
          {/if}
        </button>

        <button
          type="button"
          class="btn btn-primary"
          disabled={!hasChanges ||
            isLoading ||
            isSaving ||
            isResetting ||
            Object.keys(validationErrors).length > 0}
          on:click={saveConfig}
        >
          {#if isSaving}
            💾 保存中...
          {:else}
            💾 保存更改
          {/if}
        </button>
      </div>
    </div>

    <!-- 全局消息 -->
    {#if error}
      <div class="global-message error" role="alert">
        <svg class="message-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="10" fill="currentColor" />
          <path
            d="M7.5 7.5l5 5M12.5 7.5l-5 5"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
        <span>{error}</span>
      </div>
    {/if}

    {#if successMessage}
      <div class="global-message success" role="alert">
        <svg class="message-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="10" fill="currentColor" />
          <path
            d="M7.5 10l2.5 2.5 5-5"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>{successMessage}</span>
      </div>
    {/if}

    {#if hasChanges}
      <div class="global-message warning">
        <svg class="message-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
          <path d="M10 2l8 16H2L10 2z" fill="currentColor" />
          <path d="M10 7v5M10 14h.01" stroke="white" stroke-width="2" stroke-linecap="round" />
        </svg>
        <span>您有未保存的更改</span>
      </div>
    {/if}
  </header>

  <div class="settings-container">
    <!-- 侧边导航 -->
    <nav class="settings-nav" aria-label="设置导航">
      <ul class="nav-list">
        {#each navigationItems as item}
          <li>
            <button
              type="button"
              class="nav-item"
              class:active={activeSection === item.id}
              on:click={() => (activeSection = item.id)}
            >
              <span class="nav-icon" aria-hidden="true">{item.icon}</span>
              <div class="nav-content">
                <span class="nav-label">{item.label}</span>
                <span class="nav-description">{item.description}</span>
              </div>
            </button>
          </li>
        {/each}
      </ul>
    </nav>

    <!-- 主要设置内容 -->
    <main class="settings-content">
      {#if isLoading}
        <div class="loading-state">
          <div class="loading-spinner"></div>
          <p>正在加载配置...</p>
        </div>
      {:else if config}
        <!-- 通用设置 -->
        {#if activeSection === "general"}
          <SettingsSection title="通用设置" description="基本的应用配置和偏好设置" icon="⚙️">
            <ConfigFieldGroup title="应用配置" icon="📱">
              <ConfigField
                label="配置文件路径"
                bind:value={config.config_path}
                type="text"
                placeholder="config.yaml"
                description="应用配置文件的保存路径"
                readonly
                on:change={(e) => handleFieldChange("config_path", e.detail.value)}
              />
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}

        <!-- AI 模型设置 -->
        {#if activeSection === "ai_model"}
          <SettingsSection title="AI 模型设置" description="配置 AI 模型和推理参数" icon="🤖">
            <ConfigFieldGroup title="模型配置" icon="🧠" required>
              <ConfigField
                label="模型类型"
                bind:value={config.ai_model.model_type}
                type="select"
                options={modelTypeOptions}
                description="选择 AI 推理引擎"
                required
                on:change={(e) => handleFieldChange("ai_model.model_type", e.detail.value)}
              />

              <ConfigField
                label="模型名称"
                bind:value={config.ai_model.model_name}
                type="text"
                placeholder="microsoft/DialoGPT-medium"
                description="要使用的具体模型名称"
                required
                error={validationErrors["ai_model.model_name"]}
                on:change={(e) => handleFieldChange("ai_model.model_name", e.detail.value)}
              />

              <ConfigField
                label="系统提示词"
                bind:value={config.ai_model.system_prompt}
                type="textarea"
                rows={3}
                placeholder="你是一个友好、乐于助人的AI助手..."
                description="AI 助手的角色设定和行为指导"
                on:change={(e) => handleFieldChange("ai_model.system_prompt", e.detail.value)}
              />
            </ConfigFieldGroup>

            <ConfigFieldGroup title="服务器配置" icon="🌐">
              <ConfigField
                label="服务器地址"
                bind:value={config.ai_model.server_url}
                type="text"
                placeholder="http://localhost"
                description="AI 服务的基础 URL"
                required
                error={validationErrors["ai_model.server_url"]}
                on:change={(e) => handleFieldChange("ai_model.server_url", e.detail.value)}
              />

              <ConfigField
                label="服务器端口"
                bind:value={config.ai_model.server_port}
                type="number"
                min={1}
                max={65535}
                placeholder="11434"
                description="AI 服务监听的端口号"
                required
                error={validationErrors["ai_model.server_port"]}
                on:change={(e) =>
                  handleFieldChange("ai_model.server_port", e.detail.value.toString())}
              />
            </ConfigFieldGroup>

            {#if config.ai_model.model_type === "candle"}
              <ConfigFieldGroup title="Candle 专用设置" icon="🕯️">
                <ConfigField
                  label="模型 ID"
                  bind:value={config.ai_model.candle_model_id}
                  type="text"
                  placeholder="microsoft/DialoGPT-medium"
                  description="Hugging Face 模型标识符"
                  on:change={(e) => handleFieldChange("ai_model.candle_model_id", e.detail.value)}
                />

                <ConfigField
                  label="模型版本"
                  bind:value={config.ai_model.candle_revision}
                  type="text"
                  placeholder="main"
                  description="模型的 Git 分支或标签"
                  on:change={(e) => handleFieldChange("ai_model.candle_revision", e.detail.value)}
                />

                <ConfigField
                  label="Flash Attention"
                  bind:value={config.ai_model.candle_use_flash_attn}
                  type="toggle"
                  description="启用 Flash Attention 优化（需要兼容的 GPU）"
                  on:change={(e) =>
                    handleFieldChange("ai_model.candle_use_flash_attn", e.detail.value)}
                />
              </ConfigFieldGroup>
            {/if}
          </SettingsSection>
        {/if}

        <!-- 语音设置 -->
        {#if activeSection === "voice"}
          <SettingsSection title="语音识别设置" description="配置语音输入和识别功能" icon="🎤">
            <ConfigFieldGroup title="语音功能" icon="🗣️">
              <ConfigField
                label="启用语音识别"
                bind:value={config.voice.enabled}
                type="toggle"
                description="开启麦克风语音输入功能"
                on:change={(e) => handleFieldChange("voice.enabled", e.detail.value)}
              />

              <ConfigField
                label="模型路径"
                bind:value={config.voice.model_path}
                type="text"
                placeholder="model/vosk-model-small-cn-0.22"
                description="Vosk 语音识别模型的路径"
                disabled={!config.voice.enabled}
                error={validationErrors["voice.model_path"]}
                on:change={(e) => handleFieldChange("voice.model_path", e.detail.value)}
              />

              <ConfigField
                label="识别超时"
                bind:value={config.voice.timeout_seconds}
                type="range"
                min={5}
                max={300}
                step={5}
                description="语音识别的超时时间（秒）"
                disabled={!config.voice.enabled}
                error={validationErrors["voice.timeout_seconds"]}
                on:change={(e) => handleFieldChange("voice.timeout_seconds", e.detail.value)}
              />
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}

        <!-- 界面设置 -->
        {#if activeSection === "ui"}
          <SettingsSection title="界面设置" description="自定义应用的外观和体验" icon="🎨">
            <ConfigFieldGroup title="外观主题" icon="🌙">
              <ConfigField
                label="主题模式"
                bind:value={config.ui.theme}
                type="select"
                options={themeOptions}
                description="选择应用的颜色主题"
                on:change={(e) => handleFieldChange("ui.theme", e.detail.value)}
              />

              <ConfigField
                label="界面语言"
                bind:value={config.ui.language}
                type="select"
                options={languageOptions}
                description="选择应用界面的显示语言"
                on:change={(e) => handleFieldChange("ui.language", e.detail.value)}
              />
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}

        <!-- 数据库设置 -->
        {#if activeSection === "database"}
          <SettingsSection title="数据存储设置" description="配置聊天记录和数据存储" icon="🗄️">
            <ConfigFieldGroup title="数据库配置" icon="💾">
              <ConfigField
                label="启用数据库"
                bind:value={config.database.enabled}
                type="toggle"
                description="是否保存聊天记录到本地数据库"
                on:change={(e) => handleFieldChange("database.enabled", e.detail.value)}
              />

              <ConfigField
                label="数据库路径"
                bind:value={config.database.path}
                type="text"
                placeholder="database/chat_database.db"
                description="SQLite 数据库文件的保存路径"
                disabled={!config.database.enabled}
                error={validationErrors["database.path"]}
                on:change={(e) => handleFieldChange("database.path", e.detail.value)}
              />
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}

        <!-- 高级设置 -->
        {#if activeSection === "advanced"}
          <SettingsSection title="高级设置" description="应用行为和性能调优选项" icon="🔧">
            <ConfigFieldGroup title="应用行为" icon="⚡">
              <ConfigField
                label="日志级别"
                bind:value={config.app_behavior.log_level}
                type="select"
                options={logLevelOptions}
                description="控制应用日志的详细程度"
                on:change={(e) => handleFieldChange("app_behavior.log_level", e.detail.value)}
              />

              <ConfigField
                label="默认对话标题"
                bind:value={config.app_behavior.default_conversation_title}
                type="text"
                placeholder="新对话"
                description="创建新对话时的默认标题"
                required
                error={validationErrors["app_behavior.default_conversation_title"]}
                on:change={(e) =>
                  handleFieldChange("app_behavior.default_conversation_title", e.detail.value)}
              />

              <ConfigField
                label="欢迎消息"
                bind:value={config.app_behavior.welcome_message}
                type="textarea"
                rows={2}
                placeholder="欢迎使用聊天应用!"
                description="应用启动时显示的欢迎信息"
                on:change={(e) => handleFieldChange("app_behavior.welcome_message", e.detail.value)}
              />

              <ConfigField
                label="显示错误对话框"
                bind:value={config.app_behavior.show_error_dialogs}
                type="toggle"
                description="发生错误时是否显示弹窗提示"
                on:change={(e) =>
                  handleFieldChange("app_behavior.show_error_dialogs", e.detail.value)}
              />

              <ConfigField
                label="自动重试失败初始化"
                bind:value={config.app_behavior.auto_retry_failed_init}
                type="toggle"
                description="初始化失败时是否自动重试"
                on:change={(e) =>
                  handleFieldChange("app_behavior.auto_retry_failed_init", e.detail.value)}
              />
            </ConfigFieldGroup>

            <ConfigFieldGroup title="性能调优" icon="🚀" collapsible collapsed>
              <ConfigField
                label="消息缓冲区大小"
                bind:value={config.app_behavior.message_chunk_buffer_size}
                type="range"
                min={1}
                max={100}
                step={1}
                description="流式输出时的消息分块缓冲区大小"
                error={validationErrors["app_behavior.message_chunk_buffer_size"]}
                on:change={(e) =>
                  handleFieldChange("app_behavior.message_chunk_buffer_size", e.detail.value)}
              />

              <ConfigField
                label="发送间隔"
                bind:value={config.app_behavior.message_chunk_send_interval_ms}
                type="range"
                min={1}
                max={10000}
                step={1}
                description="消息分块的发送间隔（毫秒）"
                error={validationErrors["app_behavior.message_chunk_send_interval_ms"]}
                on:change={(e) =>
                  handleFieldChange("app_behavior.message_chunk_send_interval_ms", e.detail.value)}
              />
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}

        <!-- 导入导出 -->
        {#if activeSection === "import_export"}
          <SettingsSection title="导入导出" description="备份和恢复您的配置设置" icon="📁">
            <ConfigFieldGroup title="配置管理" icon="📋">
              <div class="import-export-actions">
                <button
                  type="button"
                  class="btn btn-outline"
                  on:click={exportConfig}
                  disabled={isLoading || isSaving}
                >
                  📤 导出配置
                </button>

                <div class="file-input-wrapper">
                  <input
                    type="file"
                    accept=".json"
                    on:change={importConfig}
                    class="file-input"
                    id="config-import"
                  />
                  <label for="config-import" class="btn btn-outline"> 📥 导入配置 </label>
                </div>
              </div>

              <div class="export-info">
                <p class="info-text">
                  📝 配置文件包含所有设置信息，可用于备份或在不同设备间同步配置。
                </p>
                <p class="info-text">⚠️ 导入配置将覆盖当前所有设置，请确保备份重要配置。</p>
              </div>
            </ConfigFieldGroup>
          </SettingsSection>
        {/if}
      {:else}
        <div class="error-state">
          <h3>无法加载配置</h3>
          <p>请检查应用状态或重新启动应用</p>
          <button type="button" class="btn btn-primary" on:click={loadConfig}> 重新加载 </button>
        </div>
      {/if}
    </main>
  </div>
</div>

<style>
  .settings-page {
    min-height: 100vh;
    background-color: #f9fafb;
  }

  :global(.dark) .settings-page {
    background-color: #111827;
  }

  .page-header {
    position: sticky;
    top: 0;
    z-index: 10;
    background-color: white;
    border-bottom: 1px solid #e5e7eb;
    padding: 1rem 1.5rem;
  }

  :global(.dark) .page-header {
    background-color: #1f2937;
    border-bottom-color: #374151;
  }

  .header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 80rem;
    margin: 0 auto;
  }

  .header-info {
    flex: 1;
  }

  .page-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: #111827;
    margin-bottom: 0.25rem;
  }

  :global(.dark) .page-title {
    color: #f9fafb;
  }

  .page-description {
    font-size: 0.875rem;
    color: #6b7280;
  }

  :global(.dark) .page-description {
    color: #9ca3af;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .global-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
    padding: 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    max-width: 80rem;
    margin-left: auto;
    margin-right: auto;
  }

  .global-message.success {
    background-color: #ecfdf5;
    color: #065f46;
  }

  :global(.dark) .global-message.success {
    background-color: rgba(6, 95, 70, 0.2);
    color: #6ee7b7;
  }

  .global-message.error {
    background-color: #fef2f2;
    color: #991b1b;
  }

  :global(.dark) .global-message.error {
    background-color: rgba(153, 27, 27, 0.2);
    color: #fca5a5;
  }

  .global-message.warning {
    background-color: #fffbeb;
    color: #92400e;
  }

  :global(.dark) .global-message.warning {
    background-color: rgba(146, 64, 14, 0.2);
    color: #fcd34d;
  }

  .message-icon {
    flex-shrink: 0;
  }

  .settings-container {
    display: flex;
    max-width: 80rem;
    margin: 0 auto;
  }

  .settings-nav {
    position: sticky;
    top: 5rem;
    height: fit-content;
    width: 16rem;
    flex-shrink: 0;
    padding: 1.5rem;
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .nav-item {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: 0.5rem;
    text-align: left;
    transition: background-color 0.15s;
    background: none;
    border: none;
    cursor: pointer;
  }

  .nav-item:hover {
    background-color: #f3f4f6;
  }

  :global(.dark) .nav-item:hover {
    background-color: #1f2937;
  }

  .nav-item.active {
    background-color: #eff6ff;
    color: #1d4ed8;
    border: 1px solid #dbeafe;
  }

  :global(.dark) .nav-item.active {
    background-color: rgba(29, 78, 216, 0.2);
    color: #93c5fd;
    border-color: rgba(29, 78, 216, 0.8);
  }

  .nav-icon {
    font-size: 1.125rem;
    flex-shrink: 0;
    margin-top: 0.125rem;
  }

  .nav-content {
    flex: 1;
    min-width: 0;
  }

  .nav-label {
    display: block;
    font-weight: 500;
    color: #111827;
  }

  :global(.dark) .nav-label {
    color: #f9fafb;
  }

  .nav-description {
    display: block;
    font-size: 0.75rem;
    color: #6b7280;
    margin-top: 0.125rem;
  }

  :global(.dark) .nav-description {
    color: #9ca3af;
  }

  .settings-content {
    flex: 1;
    padding: 1.5rem;
  }

  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 0;
    text-align: center;
  }

  .loading-spinner {
    width: 2rem;
    height: 2rem;
    border: 2px solid #d1d5db;
    border-top-color: #2563eb;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .import-export-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  .file-input-wrapper {
    position: relative;
  }

  .file-input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }

  .export-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background-color: #f9fafb;
    border-radius: 0.5rem;
  }

  :global(.dark) .export-info {
    background-color: #1f2937;
  }

  .info-text {
    font-size: 0.875rem;
    color: #6b7280;
  }

  :global(.dark) .info-text {
    color: #9ca3af;
  }

  /* Button styles */
  .btn {
    display: inline-flex;
    align-items: center;
    padding: 0.5rem 1rem;
    border: 1px solid;
    font-weight: 500;
    border-radius: 0.375rem;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    font-size: 0.875rem;
    transition:
      background-color 0.15s,
      border-color 0.15s;
    cursor: pointer;
  }

  .btn:focus {
    outline: none;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.5);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    border-color: transparent;
    color: white;
    background-color: #2563eb;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #1d4ed8;
  }

  .btn-secondary {
    border-color: #d1d5db;
    color: #374151;
    background-color: white;
  }

  :global(.dark) .btn-secondary {
    border-color: #4b5563;
    color: #e5e7eb;
    background-color: #1f2937;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #f9fafb;
  }

  :global(.dark) .btn-secondary:hover:not(:disabled) {
    background-color: #374151;
  }

  .btn-danger {
    border-color: transparent;
    color: white;
    background-color: #dc2626;
  }

  .btn-danger:hover:not(:disabled) {
    background-color: #b91c1c;
  }

  .btn-outline {
    border-color: #d1d5db;
    color: #374151;
    background-color: white;
  }

  :global(.dark) .btn-outline {
    border-color: #4b5563;
    color: #e5e7eb;
    background-color: #1f2937;
  }

  .btn-outline:hover:not(:disabled) {
    background-color: #f9fafb;
  }

  :global(.dark) .btn-outline:hover:not(:disabled) {
    background-color: #374151;
  }

  /* Responsive design */
  @media (max-width: 1024px) {
    .settings-container {
      flex-direction: column;
    }

    .settings-nav {
      position: static;
      width: 100%;
      padding: 1rem;
    }

    .nav-list {
      display: flex;
      overflow-x: auto;
      flex-direction: row;
      gap: 0.5rem;
      padding-bottom: 0.5rem;
    }

    .nav-item {
      flex-shrink: 0;
      min-width: fit-content;
    }

    .header-content {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .header-actions {
      width: 100%;
      justify-content: flex-start;
    }
  }

  @media (max-width: 640px) {
    .page-header {
      padding: 0.75rem 1rem;
    }

    .settings-content {
      padding: 1rem;
    }

    .header-actions {
      flex-direction: column;
      gap: 0.5rem;
    }

    .import-export-actions {
      flex-direction: column;
    }
  }
</style>
