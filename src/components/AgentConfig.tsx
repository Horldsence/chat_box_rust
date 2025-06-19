import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Save,
  Download,
  Upload,
  RefreshCw,
  User,
  Settings,
  Heart,
  Smile,
  Clock,
  Zap,
} from "lucide-react";

// Agent 相关类型定义
interface AgentPersonality {
  friendliness: number;
  professionalism: number;
  humor: number;
  patience: number;
  creativity: number;
  expression_style: string;
  language_preference: string;
}

interface AgentBehavior {
  response_length: "Brief" | "Moderate" | "Detailed" | "Adaptive";
  use_emojis: boolean;
  ask_questions: boolean;
  offer_suggestions: boolean;
  remember_context: boolean;
  personalized_responses: boolean;
}

interface AgentLive2DIntegration {
  enabled: boolean;
  emotion_mapping: Record<string, string>;
  action_triggers: Record<string, string>;
  auto_expression: boolean;
  speaking_actions: string[];
  thinking_actions: string[];
}

interface AgentConfig {
  name: string;
  role: string;
  system_prompt: string;
  personality: AgentPersonality;
  behavior: AgentBehavior;
  live2d_integration: AgentLive2DIntegration;
  preset_responses: Record<string, string>;
  knowledge_domains: string[];
  created_at: number;
  updated_at: number;
}

interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  config: AgentConfig;
  preview_image?: string;
  tags: string[];
}

interface AgentConfigProps {
  onConfigChange?: (config: AgentConfig) => void;
  className?: string;
}

const AgentConfig: React.FC<AgentConfigProps> = ({
  onConfigChange,
  className = "",
}) => {
  const [config, setConfig] = useState<AgentConfig | null>(null);
  const [templates, setTemplates] = useState<AgentTemplate[]>([]);
  const [activeTab, setActiveTab] = useState<string>("basic");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      setIsLoading(true);
      const agentConfig = await invoke<AgentConfig>("get_agent_config");
      setConfig(agentConfig);
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载配置失败");
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 加载模板
  const loadTemplates = useCallback(async () => {
    try {
      const agentTemplates = await invoke<AgentTemplate[]>(
        "get_agent_templates",
      );
      setTemplates(agentTemplates);
    } catch (err) {
      console.error("加载模板失败:", err);
    }
  }, []);

  // 保存配置
  const saveConfig = useCallback(async () => {
    if (!config) return;

    try {
      setIsLoading(true);
      await invoke("update_agent_config", { config });
      setSuccessMessage("配置保存成功");
      onConfigChange?.(config);
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "保存配置失败");
    } finally {
      setIsLoading(false);
    }
  }, [config, onConfigChange]);

  // 应用模板
  const applyTemplate = useCallback(
    async (templateId: string) => {
      try {
        setIsLoading(true);
        await invoke("apply_agent_template", { templateId });
        await loadConfig();
        setSuccessMessage("模板应用成功");
        setTimeout(() => setSuccessMessage(null), 3000);
      } catch (err) {
        setError(err instanceof Error ? err.message : "应用模板失败");
      } finally {
        setIsLoading(false);
      }
    },
    [loadConfig],
  );

  // 导出配置
  const exportConfig = useCallback(async () => {
    try {
      const configJson = await invoke<string>("export_agent_config");
      const blob = new Blob([configJson], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `agent_config_${Date.now()}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setSuccessMessage("配置导出成功");
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "导出配置失败");
    }
  }, []);

  // 导入配置
  const importConfig = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        await invoke("import_agent_config", { configJson: text });
        await loadConfig();
        setSuccessMessage("配置导入成功");
        setTimeout(() => setSuccessMessage(null), 3000);
      } catch (err) {
        setError(err instanceof Error ? err.message : "导入配置失败");
      }
    },
    [loadConfig],
  );

  // 更新配置字段
  const updateConfig = useCallback(
    (updates: Partial<AgentConfig>) => {
      if (!config) return;
      setConfig({ ...config, ...updates });
    },
    [config],
  );

  // 更新个性特征
  const updatePersonality = useCallback(
    (updates: Partial<AgentPersonality>) => {
      if (!config) return;
      setConfig({
        ...config,
        personality: { ...config.personality, ...updates },
      });
    },
    [config],
  );

  // 更新行为配置
  const updateBehavior = useCallback(
    (updates: Partial<AgentBehavior>) => {
      if (!config) return;
      setConfig({
        ...config,
        behavior: { ...config.behavior, ...updates },
      });
    },
    [config],
  );

  // 更新Live2D集成
  const updateLive2DIntegration = useCallback(
    (updates: Partial<AgentLive2DIntegration>) => {
      if (!config) return;
      setConfig({
        ...config,
        live2d_integration: { ...config.live2d_integration, ...updates },
      });
    },
    [config],
  );

  // 初始化
  useEffect(() => {
    loadConfig();
    loadTemplates();
  }, [loadConfig, loadTemplates]);

  if (isLoading && !config) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="text-center text-gray-500 p-8">无法加载Agent配置</div>
    );
  }

  return (
    <div className={`agent-config ${className}`}>
      {/* 头部 */}
      <div className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <User className="h-6 w-6 text-blue-500" />
            <h2 className="text-xl font-semibold text-gray-900">Agent 配置</h2>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={exportConfig}
              className="flex items-center space-x-1 px-3 py-1.5 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
            >
              <Download className="h-4 w-4" />
              <span>导出</span>
            </button>
            <label className="flex items-center space-x-1 px-3 py-1.5 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 cursor-pointer">
              <Upload className="h-4 w-4" />
              <span>导入</span>
              <input
                type="file"
                accept=".json"
                className="hidden"
                onChange={importConfig}
              />
            </label>
            <button
              onClick={saveConfig}
              disabled={isLoading}
              className="flex items-center space-x-1 px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
            >
              <Save className="h-4 w-4" />
              <span>保存</span>
            </button>
          </div>
        </div>

        {/* 通知消息 */}
        {error && (
          <div className="mt-4 p-3 bg-red-100 border border-red-300 text-red-700 rounded">
            {error}
            <button
              onClick={() => setError(null)}
              className="ml-2 text-red-500 hover:text-red-700"
            >
              ×
            </button>
          </div>
        )}

        {successMessage && (
          <div className="mt-4 p-3 bg-green-100 border border-green-300 text-green-700 rounded">
            {successMessage}
          </div>
        )}

        {/* 标签页导航 */}
        <div className="mt-4 border-b border-gray-200">
          <nav className="-mb-px flex space-x-8">
            {[
              { id: "basic", label: "基本信息", icon: User },
              { id: "personality", label: "个性特征", icon: Heart },
              { id: "behavior", label: "行为配置", icon: Settings },
              { id: "live2d", label: "Live2D集成", icon: Smile },
              { id: "templates", label: "模板管理", icon: RefreshCw },
            ].map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`flex items-center space-x-2 py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === id
                    ? "border-blue-500 text-blue-600"
                    : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                }`}
              >
                <Icon className="h-4 w-4" />
                <span>{label}</span>
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* 内容区域 */}
      <div className="p-6">
        {/* 基本信息 */}
        {activeTab === "basic" && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Agent 名称
                </label>
                <input
                  type="text"
                  value={config.name}
                  onChange={(e) => updateConfig({ name: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  角色类型
                </label>
                <select
                  value={config.role}
                  onChange={(e) => updateConfig({ role: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="Assistant">助手</option>
                  <option value="Friend">朋友</option>
                  <option value="Mentor">导师</option>
                  <option value="Expert">专家</option>
                  <option value="Entertainment">娱乐</option>
                  <option value="Custom">自定义</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                系统提示词
              </label>
              <textarea
                value={config.system_prompt}
                onChange={(e) =>
                  updateConfig({ system_prompt: e.target.value })
                }
                rows={8}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="输入Agent的系统提示词，描述其角色、性格和行为方式..."
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                知识领域
              </label>
              <div className="flex flex-wrap gap-2 mb-2">
                {config.knowledge_domains.map((domain, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                  >
                    {domain}
                    <button
                      onClick={() => {
                        const newDomains = config.knowledge_domains.filter(
                          (_, i) => i !== index,
                        );
                        updateConfig({ knowledge_domains: newDomains });
                      }}
                      className="ml-1 text-blue-600 hover:text-blue-800"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
              <div className="flex space-x-2">
                <input
                  type="text"
                  placeholder="添加知识领域"
                  onKeyPress={(e) => {
                    if (e.key === "Enter") {
                      const value = e.currentTarget.value.trim();
                      if (value && !config.knowledge_domains.includes(value)) {
                        updateConfig({
                          knowledge_domains: [
                            ...config.knowledge_domains,
                            value,
                          ],
                        });
                        e.currentTarget.value = "";
                      }
                    }
                  }}
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>
          </div>
        )}

        {/* 个性特征 */}
        {activeTab === "personality" && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {[
                { key: "friendliness", label: "友好程度", icon: Heart },
                { key: "professionalism", label: "专业程度", icon: Settings },
                { key: "humor", label: "幽默感", icon: Smile },
                { key: "patience", label: "耐心程度", icon: Clock },
                { key: "creativity", label: "创造力", icon: Zap },
              ].map(({ key, label, icon: Icon }) => (
                <div key={key}>
                  <div className="flex items-center space-x-2 mb-2">
                    <Icon className="h-4 w-4 text-gray-500" />
                    <label className="text-sm font-medium text-gray-700">
                      {label} (
                      {config.personality[key as keyof AgentPersonality]}/10)
                    </label>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="10"
                    value={
                      config.personality[
                        key as keyof AgentPersonality
                      ] as number
                    }
                    onChange={(e) =>
                      updatePersonality({
                        [key]: parseInt(e.target.value),
                      } as Partial<AgentPersonality>)
                    }
                    className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                  />
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  表达风格
                </label>
                <select
                  value={config.personality.expression_style}
                  onChange={(e) =>
                    updatePersonality({ expression_style: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="warm_and_helpful">温暖有用</option>
                  <option value="professional">专业严谨</option>
                  <option value="casual_friendly">随和友好</option>
                  <option value="enthusiastic">热情洋溢</option>
                  <option value="calm_patient">沉着耐心</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  语言偏好
                </label>
                <select
                  value={config.personality.language_preference}
                  onChange={(e) =>
                    updatePersonality({ language_preference: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="zh-CN">中文</option>
                  <option value="en-US">English</option>
                  <option value="ja-JP">日本語</option>
                </select>
              </div>
            </div>
          </div>
        )}

        {/* 行为配置 */}
        {activeTab === "behavior" && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  回复长度偏好
                </label>
                <select
                  value={config.behavior.response_length}
                  onChange={(e) =>
                    updateBehavior({
                      response_length: e.target
                        .value as AgentBehavior["response_length"],
                    })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="Brief">简洁</option>
                  <option value="Moderate">适中</option>
                  <option value="Detailed">详细</option>
                  <option value="Adaptive">自适应</option>
                </select>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {[
                { key: "use_emojis", label: "使用表情符号" },
                { key: "ask_questions", label: "主动提问" },
                { key: "offer_suggestions", label: "提供建议" },
                { key: "remember_context", label: "记住上下文" },
                { key: "personalized_responses", label: "个性化回复" },
              ].map(({ key, label }) => (
                <div key={key} className="flex items-center">
                  <input
                    type="checkbox"
                    checked={
                      config.behavior[key as keyof AgentBehavior] as boolean
                    }
                    onChange={(e) =>
                      updateBehavior({
                        [key]: e.target.checked,
                      } as Partial<AgentBehavior>)
                    }
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <label className="ml-2 text-sm text-gray-700">{label}</label>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Live2D集成 */}
        {activeTab === "live2d" && (
          <div className="space-y-6">
            <div className="flex items-center">
              <input
                type="checkbox"
                checked={config.live2d_integration.enabled}
                onChange={(e) =>
                  updateLive2DIntegration({ enabled: e.target.checked })
                }
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label className="ml-2 text-sm font-medium text-gray-700">
                启用Live2D集成
              </label>
            </div>

            {config.live2d_integration.enabled && (
              <>
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    checked={config.live2d_integration.auto_expression}
                    onChange={(e) =>
                      updateLive2DIntegration({
                        auto_expression: e.target.checked,
                      })
                    }
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <label className="ml-2 text-sm text-gray-700">
                    自动表情切换
                  </label>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    说话时动作
                  </label>
                  <div className="flex flex-wrap gap-2 mb-2">
                    {config.live2d_integration.speaking_actions.map(
                      (action, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
                        >
                          {action}
                          <button
                            onClick={() => {
                              const newActions =
                                config.live2d_integration.speaking_actions.filter(
                                  (_, i) => i !== index,
                                );
                              updateLive2DIntegration({
                                speaking_actions: newActions,
                              });
                            }}
                            className="ml-1 text-green-600 hover:text-green-800"
                          >
                            ×
                          </button>
                        </span>
                      ),
                    )}
                  </div>
                  <input
                    type="text"
                    placeholder="添加说话动作"
                    onKeyPress={(e) => {
                      if (e.key === "Enter") {
                        const value = e.currentTarget.value.trim();
                        if (
                          value &&
                          !config.live2d_integration.speaking_actions.includes(
                            value,
                          )
                        ) {
                          updateLive2DIntegration({
                            speaking_actions: [
                              ...config.live2d_integration.speaking_actions,
                              value,
                            ],
                          });
                          e.currentTarget.value = "";
                        }
                      }
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    思考时动作
                  </label>
                  <div className="flex flex-wrap gap-2 mb-2">
                    {config.live2d_integration.thinking_actions.map(
                      (action, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800"
                        >
                          {action}
                          <button
                            onClick={() => {
                              const newActions =
                                config.live2d_integration.thinking_actions.filter(
                                  (_, i) => i !== index,
                                );
                              updateLive2DIntegration({
                                thinking_actions: newActions,
                              });
                            }}
                            className="ml-1 text-purple-600 hover:text-purple-800"
                          >
                            ×
                          </button>
                        </span>
                      ),
                    )}
                  </div>
                  <input
                    type="text"
                    placeholder="添加思考动作"
                    onKeyPress={(e) => {
                      if (e.key === "Enter") {
                        const value = e.currentTarget.value.trim();
                        if (
                          value &&
                          !config.live2d_integration.thinking_actions.includes(
                            value,
                          )
                        ) {
                          updateLive2DIntegration({
                            thinking_actions: [
                              ...config.live2d_integration.thinking_actions,
                              value,
                            ],
                          });
                          e.currentTarget.value = "";
                        }
                      }
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </>
            )}
          </div>
        )}

        {/* 模板管理 */}
        {activeTab === "templates" && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {templates.map((template) => (
                <div
                  key={template.id}
                  className="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between mb-2">
                    <h3 className="font-medium text-gray-900">
                      {template.name}
                    </h3>
                    <button
                      onClick={() => applyTemplate(template.id)}
                      className="text-sm text-blue-600 hover:text-blue-800"
                    >
                      应用
                    </button>
                  </div>
                  <p className="text-sm text-gray-600 mb-3">
                    {template.description}
                  </p>
                  <div className="flex flex-wrap gap-1">
                    {template.tags.map((tag) => (
                      <span
                        key={tag}
                        className="inline-block px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default AgentConfig;
