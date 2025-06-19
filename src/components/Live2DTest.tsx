import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, Pause, Settings, TestTube, Heart, Brain } from "lucide-react";

interface Live2DTestProps {
  className?: string;
}

const Live2DTest: React.FC<Live2DTestProps> = ({ className = "" }) => {
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState("");
  const [logs, setLogs] = useState<string[]>([]);

  // 添加日志
  const addLog = (log: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [...prev, `[${timestamp}] ${log}`]);
  };

  // 测试连接
  const testConnection = async () => {
    setIsLoading(true);
    try {
      const result = await invoke("test_live2d_connection");
      setIsConnected(true);
      addLog("Live2D连接测试成功");
      console.log("Live2D连接测试结果:", result);
    } catch (error) {
      setIsConnected(false);
      addLog(`Live2D连接测试失败: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  // 执行动作
  const executeAction = async (actionType: string) => {
    try {
      await invoke("execute_live2d_action_by_type", { actionType });
      addLog(`执行动作: ${actionType}`);
    } catch (error) {
      addLog(`执行动作失败: ${error}`);
    }
  };

  // 设置表情
  const setExpression = async (expression: string) => {
    try {
      await invoke("set_live2d_expression", { expression });
      addLog(`设置表情: ${expression}`);
    } catch (error) {
      addLog(`设置表情失败: ${error}`);
    }
  };

  // 测试文本触发
  const testTextTrigger = async () => {
    if (!message.trim()) return;

    try {
      await invoke("process_ai_text_for_live2d", { textChunk: message });
      addLog(`处理文本: ${message}`);
      setMessage("");
    } catch (error) {
      addLog(`处理文本失败: ${error}`);
    }
  };

  // 开始说话
  const startSpeaking = async () => {
    try {
      await invoke("start_live2d_speaking");
      addLog("开始说话状态");
    } catch (error) {
      addLog(`开始说话失败: ${error}`);
    }
  };

  // 停止说话
  const stopSpeaking = async () => {
    try {
      await invoke("stop_live2d_speaking");
      addLog("停止说话状态");
    } catch (error) {
      addLog(`停止说话失败: ${error}`);
    }
  };

  // 测试Agent功能
  const testAgent = async () => {
    try {
      const result = await invoke("test_agent_config");
      addLog("Agent测试成功");
      console.log("Agent测试结果:", result);
    } catch (error) {
      addLog(`Agent测试失败: ${error}`);
    }
  };

  // 应用模板
  const applyTemplate = async (templateId: string) => {
    try {
      await invoke("apply_agent_template", { templateId });
      addLog(`应用Agent模板: ${templateId}`);
    } catch (error) {
      addLog(`应用模板失败: ${error}`);
    }
  };

  // 清空日志
  const clearLogs = () => {
    setLogs([]);
  };

  useEffect(() => {
    // 页面加载时自动测试连接
    testConnection();
  }, []);

  return (
    <div
      className={`live2d-test p-6 bg-white rounded-lg shadow-lg ${className}`}
    >
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900 flex items-center gap-2">
            <TestTube className="h-6 w-6 text-blue-500" />
            Live2D & Agent 测试面板
          </h2>
          <div className="flex items-center gap-2">
            <div
              className={`w-3 h-3 rounded-full ${isConnected ? "bg-green-500" : "bg-red-500"}`}
            ></div>
            <span className="text-sm text-gray-600">
              {isConnected ? "已连接" : "未连接"}
            </span>
          </div>
        </div>

        <button
          onClick={testConnection}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
        >
          {isLoading ? (
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
          ) : (
            <Settings className="h-4 w-4" />
          )}
          {isLoading ? "测试中..." : "重新测试连接"}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Live2D 控制 */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-800">Live2D 控制</h3>

          {/* 基本动作 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">基本动作</h4>
            <div className="grid grid-cols-2 gap-2">
              {[
                {
                  id: "greeting",
                  label: "问候",
                  icon: "👋",
                  color: "bg-green-500",
                },
                {
                  id: "happy",
                  label: "开心",
                  icon: "😊",
                  color: "bg-yellow-500",
                },
                {
                  id: "thinking",
                  label: "思考",
                  icon: "🤔",
                  color: "bg-purple-500",
                },
                {
                  id: "speaking",
                  label: "说话",
                  icon: "💬",
                  color: "bg-blue-500",
                },
                {
                  id: "surprised",
                  label: "惊讶",
                  icon: "😲",
                  color: "bg-pink-500",
                },
                {
                  id: "confused",
                  label: "困惑",
                  icon: "😕",
                  color: "bg-orange-500",
                },
                {
                  id: "farewell",
                  label: "告别",
                  icon: "👋",
                  color: "bg-gray-500",
                },
                { id: "idle", label: "待机", icon: "😐", color: "bg-gray-400" },
              ].map(({ id, label, icon, color }) => (
                <button
                  key={id}
                  onClick={() => executeAction(id)}
                  className={`flex items-center justify-center gap-2 p-3 text-white rounded-lg hover:opacity-90 transition-opacity ${color}`}
                >
                  <span>{icon}</span>
                  <span className="text-sm font-medium">{label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 说话控制 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">说话控制</h4>
            <div className="flex gap-2">
              <button
                onClick={startSpeaking}
                className="flex items-center gap-2 px-4 py-2 bg-green-100 text-green-700 rounded-lg hover:bg-green-200"
              >
                <Play className="h-4 w-4" />
                开始说话
              </button>
              <button
                onClick={stopSpeaking}
                className="flex items-center gap-2 px-4 py-2 bg-red-100 text-red-700 rounded-lg hover:bg-red-200"
              >
                <Pause className="h-4 w-4" />
                停止说话
              </button>
            </div>
          </div>

          {/* 表情控制 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">表情控制</h4>
            <div className="grid grid-cols-3 gap-2">
              {[
                "default",
                "happy",
                "sad",
                "angry",
                "surprised",
                "confused",
              ].map((expression) => (
                <button
                  key={expression}
                  onClick={() => setExpression(expression)}
                  className="px-3 py-2 text-sm bg-blue-50 text-blue-700 rounded-lg hover:bg-blue-100"
                >
                  {expression}
                </button>
              ))}
            </div>
          </div>

          {/* 文本触发测试 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">
              文本触发测试
            </h4>
            <div className="space-y-2">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                  placeholder="输入测试文本..."
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  onKeyPress={(e) => e.key === "Enter" && testTextTrigger()}
                />
                <button
                  onClick={testTextTrigger}
                  className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                >
                  发送
                </button>
              </div>
              <div className="flex flex-wrap gap-1">
                {["你好", "再见", "谢谢", "什么", "哇", "嗯"].map((trigger) => (
                  <button
                    key={trigger}
                    onClick={() => {
                      setMessage(trigger);
                      setTimeout(() => testTextTrigger(), 100);
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

        {/* Agent 控制 */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-800">Agent 控制</h3>

          {/* Agent 测试 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">
              Agent 测试
            </h4>
            <button
              onClick={testAgent}
              className="flex items-center gap-2 px-4 py-2 bg-purple-500 text-white rounded-lg hover:bg-purple-600"
            >
              <Brain className="h-4 w-4" />
              测试Agent配置
            </button>
          </div>

          {/* 模板切换 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">快速模板</h4>
            <div className="grid grid-cols-1 gap-2">
              {[
                {
                  id: "friendly_assistant",
                  label: "友好助手",
                  desc: "温暖友好的AI助手",
                },
                {
                  id: "professional_mentor",
                  label: "专业导师",
                  desc: "严谨的学习导师",
                },
                {
                  id: "creative_companion",
                  label: "创意伙伴",
                  desc: "富有想象力的伙伴",
                },
                { id: "tech_expert", label: "技术专家", desc: "专业技术顾问" },
              ].map(({ id, label, desc }) => (
                <button
                  key={id}
                  onClick={() => applyTemplate(id)}
                  className="p-3 text-left border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <div className="font-medium text-gray-900">{label}</div>
                  <div className="text-sm text-gray-600">{desc}</div>
                </button>
              ))}
            </div>
          </div>

          {/* 快速操作 */}
          <div>
            <h4 className="text-sm font-medium text-gray-700 mb-2">快速操作</h4>
            <div className="space-y-2">
              <button
                onClick={async () => {
                  try {
                    await invoke("reset_agent_session");
                    addLog("Agent会话已重置");
                  } catch (error) {
                    addLog(`重置会话失败: ${error}`);
                  }
                }}
                className="w-full px-4 py-2 bg-orange-100 text-orange-700 rounded-lg hover:bg-orange-200"
              >
                重置Agent会话
              </button>
              <button
                onClick={async () => {
                  try {
                    await invoke("clear_live2d_text_buffer");
                    addLog("Live2D文本缓冲区已清空");
                  } catch (error) {
                    addLog(`清空缓冲区失败: ${error}`);
                  }
                }}
                className="w-full px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200"
              >
                清空Live2D缓冲区
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* 日志区域 */}
      <div className="mt-6">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold text-gray-800">操作日志</h3>
          <button
            onClick={clearLogs}
            className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
          >
            清空日志
          </button>
        </div>
        <div className="bg-gray-900 text-green-400 p-4 rounded-lg font-mono text-sm h-48 overflow-y-auto">
          {logs.length === 0 ? (
            <div className="text-gray-500">暂无日志...</div>
          ) : (
            logs.map((log, index) => (
              <div key={index} className="mb-1">
                {log}
              </div>
            ))
          )}
        </div>
      </div>

      {/* 状态信息 */}
      <div className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-blue-50 p-4 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Settings className="h-5 w-5 text-blue-500" />
            <span className="font-medium text-blue-900">Live2D状态</span>
          </div>
          <div className="text-sm text-blue-700">
            {isConnected ? "服务正常运行" : "服务未连接"}
          </div>
        </div>

        <div className="bg-purple-50 p-4 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Brain className="h-5 w-5 text-purple-500" />
            <span className="font-medium text-purple-900">Agent状态</span>
          </div>
          <div className="text-sm text-purple-700">配置可用</div>
        </div>

        <div className="bg-green-50 p-4 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Heart className="h-5 w-5 text-green-500" />
            <span className="font-medium text-green-900">集成状态</span>
          </div>
          <div className="text-sm text-green-700">Live2D + Agent 联动</div>
        </div>
      </div>
    </div>
  );
};

export default Live2DTest;
