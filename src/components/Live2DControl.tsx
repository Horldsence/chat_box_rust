import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Pause,
  RotateCcw,
  Settings,
  Volume2,
  VolumeX,
  Eye,
  Smile,
  Brain,
  MessageSquare,
  Hand,
} from "lucide-react";

// Live2D 相关类型定义
interface Live2DAction {
  action_type: string;
  motion_group: string;
  motion_index?: number;
  expression?: string;
  duration?: number;
  priority: number;
}

interface Live2DConfig {
  model_path: string;
  scale: number;
  position: [number, number];
  auto_blink: boolean;
  auto_breath: boolean;
  default_actions: Record<string, Live2DAction>;
  text_triggers: Record<string, string>;
}

interface Live2DState {
  current_action?: Live2DAction;
  action_queue: Live2DAction[];
  is_speaking: boolean;
  last_action_time: number;
}

interface Live2DControlProps {
  className?: string;
  onActionExecuted?: (action: string) => void;
}

const Live2DControl: React.FC<Live2DControlProps> = ({
  className = "",
  onActionExecuted,
}) => {
  const [config, setConfig] = useState<Live2DConfig | null>(null);
  const [state, setState] = useState<Live2DState | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // 预定义动作列表
  const predefinedActions = [
    { id: "greeting", label: "问候", icon: Hand, color: "bg-green-500" },
    { id: "happy", label: "开心", icon: Smile, color: "bg-yellow-500" },
    { id: "thinking", label: "思考", icon: Brain, color: "bg-purple-500" },
    {
      id: "speaking",
      label: "说话",
      icon: MessageSquare,
      color: "bg-blue-500",
    },
    { id: "surprised", label: "惊讶", icon: Eye, color: "bg-pink-500" },
    { id: "confused", label: "困惑", icon: RotateCcw, color: "bg-orange-500" },
    { id: "farewell", label: "告别", icon: Hand, color: "bg-gray-500" },
    { id: "idle", label: "待机", icon: Pause, color: "bg-gray-400" },
  ];

  // 表情列表
  const expressions = [
    "default",
    "happy",
    "sad",
    "angry",
    "surprised",
    "confused",
    "thinking",
    "excited",
    "sleepy",
    "focused",
    "smile",
    "wink",
  ];

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      const live2dConfig = await invoke<Live2DConfig>("get_live2d_config");
      setConfig(live2dConfig);
    } catch (err) {
      setError(err instanceof Error ? err.message : "加载配置失败");
    }
  }, []);

  // 加载状态
  const loadState = useCallback(async () => {
    try {
      const live2dState = await invoke<Live2DState>("get_live2d_state");
      setState(live2dState);
    } catch (err) {
      console.error("加载状态失败:", err);
    }
  }, []);

  // 执行动作
  const executeAction = useCallback(
    async (actionType: string) => {
      try {
        setIsLoading(true);
        await invoke("execute_live2d_action_by_type", { actionType });
        await loadState();
        onActionExecuted?.(actionType);
        setSuccessMessage(`执行动作: ${actionType}`);
        setTimeout(() => setSuccessMessage(null), 2000);
      } catch (err) {
        setError(err instanceof Error ? err.message : "执行动作失败");
      } finally {
        setIsLoading(false);
      }
    },
    [loadState, onActionExecuted],
  );

  // 设置表情
  const setExpression = useCallback(async (expression: string) => {
    try {
      setIsLoading(true);
      await invoke("set_live2d_expression", { expression });
      setSuccessMessage(`设置表情: ${expression}`);
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "设置表情失败");
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 开始说话
  const startSpeaking = useCallback(async () => {
    try {
      await invoke("start_live2d_speaking");
      await loadState();
      setSuccessMessage("开始说话状态");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "开始说话失败");
    }
  }, [loadState]);

  // 停止说话
  const stopSpeaking = useCallback(async () => {
    try {
      await invoke("stop_live2d_speaking");
      await loadState();
      setSuccessMessage("停止说话状态");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "停止说话失败");
    }
  }, [loadState]);

  // 开始思考
  const startThinking = useCallback(async () => {
    try {
      await invoke("start_live2d_thinking");
      await loadState();
      setSuccessMessage("开始思考状态");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "开始思考失败");
    }
  }, [loadState]);

  // 清空动作队列
  const clearActionQueue = useCallback(async () => {
    try {
      await invoke("clear_live2d_text_buffer");
      await loadState();
      setSuccessMessage("清空动作队列");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "清空队列失败");
    }
  }, [loadState]);

  // 重置配置
  const resetConfig = useCallback(async () => {
    try {
      await invoke("reset_live2d_config");
      await loadConfig();
      setSuccessMessage("配置已重置");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "重置配置失败");
    }
  }, [loadConfig]);

  // 测试连接
  const testConnection = useCallback(async () => {
    try {
      setIsLoading(true);
      const result = await invoke("test_live2d_connection");
      console.log("Live2D连接测试结果:", result);
      setSuccessMessage("连接测试成功");
      setTimeout(() => setSuccessMessage(null), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "连接测试失败");
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初始化
  useEffect(() => {
    loadConfig();
    loadState();
  }, [loadConfig, loadState]);

  // 定期更新状态
  useEffect(() => {
    const interval = setInterval(() => {
      loadState();
    }, 2000);

    return () => clearInterval(interval);
  }, [loadState]);

  return (
    <div
      className={`live2d-control bg-white border border-gray-200 rounded-lg ${className}`}
    >
      {/* 头部 */}
      <div className="border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Settings className="h-5 w-5 text-blue-500" />
            <h3 className="text-lg font-medium text-gray-900">Live2D 控制</h3>
          </div>
          <button
            onClick={testConnection}
            disabled={isLoading}
            className="text-sm text-blue-600 hover:text-blue-800 disabled:opacity-50"
          >
            测试连接
          </button>
        </div>

        {/* 通知消息 */}
        {error && (
          <div className="mt-2 p-2 bg-red-100 border border-red-300 text-red-700 rounded text-sm">
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
          <div className="mt-2 p-2 bg-green-100 border border-green-300 text-green-700 rounded text-sm">
            {successMessage}
          </div>
        )}
      </div>

      <div className="p-4 space-y-6">
        {/* 状态显示 */}
        {state && (
          <div className="bg-gray-50 rounded-lg p-3">
            <h4 className="text-sm font-medium text-gray-700 mb-2">当前状态</h4>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-600">说话状态:</span>
                <span
                  className={`ml-2 ${state.is_speaking ? "text-green-600" : "text-gray-400"}`}
                >
                  {state.is_speaking ? "正在说话" : "静默"}
                </span>
              </div>
              <div>
                <span className="text-gray-600">当前动作:</span>
                <span className="ml-2 text-blue-600">
                  {state.current_action?.action_type || "无"}
                </span>
              </div>
              <div>
                <span className="text-gray-600">队列长度:</span>
                <span className="ml-2 text-purple-600">
                  {state.action_queue.length}
                </span>
              </div>
              <div>
                <span className="text-gray-600">最后动作:</span>
                <span className="ml-2 text-gray-500">
                  {state.last_action_time
                    ? new Date(state.last_action_time).toLocaleTimeString()
                    : "无"}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* 快速控制 */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-3">快速控制</h4>
          <div className="flex flex-wrap gap-2">
            <button
              onClick={state?.is_speaking ? stopSpeaking : startSpeaking}
              className={`flex items-center space-x-1 px-3 py-2 rounded-md text-sm font-medium ${
                state?.is_speaking
                  ? "bg-red-100 text-red-700 hover:bg-red-200"
                  : "bg-green-100 text-green-700 hover:bg-green-200"
              }`}
            >
              {state?.is_speaking ? (
                <VolumeX className="h-4 w-4" />
              ) : (
                <Volume2 className="h-4 w-4" />
              )}
              <span>{state?.is_speaking ? "停止说话" : "开始说话"}</span>
            </button>

            <button
              onClick={startThinking}
              className="flex items-center space-x-1 px-3 py-2 bg-purple-100 text-purple-700 rounded-md text-sm font-medium hover:bg-purple-200"
            >
              <Brain className="h-4 w-4" />
              <span>思考</span>
            </button>

            <button
              onClick={clearActionQueue}
              className="flex items-center space-x-1 px-3 py-2 bg-gray-100 text-gray-700 rounded-md text-sm font-medium hover:bg-gray-200"
            >
              <RotateCcw className="h-4 w-4" />
              <span>清空队列</span>
            </button>

            <button
              onClick={resetConfig}
              className="flex items-center space-x-1 px-3 py-2 bg-orange-100 text-orange-700 rounded-md text-sm font-medium hover:bg-orange-200"
            >
              <Settings className="h-4 w-4" />
              <span>重置配置</span>
            </button>
          </div>
        </div>

        {/* 预定义动作 */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-3">预定义动作</h4>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            {predefinedActions.map(({ id, label, icon: Icon, color }) => (
              <button
                key={id}
                onClick={() => executeAction(id)}
                disabled={isLoading}
                className={`flex flex-col items-center justify-center p-3 rounded-lg text-white hover:opacity-90 transition-opacity disabled:opacity-50 ${color}`}
              >
                <Icon className="h-6 w-6 mb-1" />
                <span className="text-xs font-medium">{label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* 表情控制 */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-3">表情控制</h4>
          <div className="grid grid-cols-3 sm:grid-cols-6 gap-2">
            {expressions.map((expression) => (
              <button
                key={expression}
                onClick={() => setExpression(expression)}
                disabled={isLoading}
                className="px-3 py-2 text-sm bg-blue-50 text-blue-700 rounded-md hover:bg-blue-100 transition-colors disabled:opacity-50"
              >
                {expression}
              </button>
            ))}
          </div>
        </div>

        {/* 配置面板 */}
        {config && (
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-3">配置信息</h4>
            <div className="bg-gray-50 rounded-lg p-3 space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">模型路径:</span>
                <span className="text-gray-900 font-mono text-xs">
                  {config.model_path}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">缩放比例:</span>
                <span className="text-gray-900">{config.scale}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">位置:</span>
                <span className="text-gray-900">
                  [{config.position.join(", ")}]
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">自动眨眼:</span>
                <span
                  className={
                    config.auto_blink ? "text-green-600" : "text-red-600"
                  }
                >
                  {config.auto_blink ? "启用" : "禁用"}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">自动呼吸:</span>
                <span
                  className={
                    config.auto_breath ? "text-green-600" : "text-red-600"
                  }
                >
                  {config.auto_breath ? "启用" : "禁用"}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">文本触发器:</span>
                <span className="text-gray-900">
                  {Object.keys(config.text_triggers).length} 个
                </span>
              </div>
            </div>
          </div>
        )}

        {/* 文本触发器测试 */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-3">
            文本触发测试
          </h4>
          <div className="space-y-2">
            <input
              type="text"
              placeholder="输入文本测试触发器..."
              onKeyPress={async (e) => {
                if (e.key === "Enter") {
                  const text = e.currentTarget.value.trim();
                  if (text) {
                    try {
                      await invoke("process_ai_text_for_live2d", {
                        textChunk: text,
                      });
                      e.currentTarget.value = "";
                      setSuccessMessage("文本已处理");
                      setTimeout(() => setSuccessMessage(null), 2000);
                    } catch (err) {
                      setError(
                        err instanceof Error ? err.message : "处理文本失败",
                      );
                    }
                  }
                }
              }}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
            />
            <div className="flex flex-wrap gap-1">
              {config &&
                Object.keys(config.text_triggers).map((trigger) => (
                  <button
                    key={trigger}
                    onClick={async () => {
                      try {
                        await invoke("process_ai_text_for_live2d", {
                          textChunk: trigger,
                        });
                        setSuccessMessage(`触发: ${trigger}`);
                        setTimeout(() => setSuccessMessage(null), 2000);
                      } catch (err) {
                        setError(
                          err instanceof Error ? err.message : "触发失败",
                        );
                      }
                    }}
                    className="px-2 py-1 text-xs bg-yellow-100 text-yellow-800 rounded hover:bg-yellow-200"
                  >
                    {trigger}
                  </button>
                ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Live2DControl;
