import React, { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AlertTriangle, Download, X, ExternalLink } from "lucide-react";
import { useLive2DConfig, checkWebGLSupport } from "../hooks/useLive2DConfig";

// Live2D 相关类型定义
interface Live2DAction {
  action_type: string;
  motion_group: string;
  motion_index?: number;
  expression?: string;
  duration?: number;
  priority: number;
}

interface Live2DEvent {
  event_type: string;
  action: Live2DAction;
  timestamp: number;
  metadata: Record<string, string>;
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

interface Live2DProps {
  modelPath?: string;
  scale?: number;
  position?: [number, number];
  className?: string;
  onActionChange?: (action: Live2DAction) => void;
  onExpressionChange?: (expression: string) => void;
}

// 简化的角色组件
interface SimpleCharacter {
  graphics: any;
  animate: (action: string) => void;
  setExpression: (expression: string) => void;
}

const Live2D: React.FC<Live2DProps> = ({
  modelPath = "/models/live2d/default/model.json",
  scale = 1.0,
  position = [0, 0],
  className = "",
  onActionChange,
  onExpressionChange,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [config] = useState<Live2DConfig | null>(null);
  const [state, setState] = useState<Live2DState | null>(null);
  const [currentExpression, setCurrentExpression] = useState<string>("default");

  // 使用配置管理Hook
  const {
    config: appConfig,
    modelStatus,
    isLoading: isConfigLoading,
    error: configError,
    loadConfig,
    enableLive2D,
    disableLive2D,
    isModelValid,
    canUseFallback,
    needsSetup,
  } = useLive2DConfig();

  // 组件状态
  const [showModelSetup, setShowModelSetup] = useState(false);
  const [useSimpleCharacter, setUseSimpleCharacter] = useState(false);
  const [webglSupported, setWebglSupported] = useState(true);

  // Live2D应用实例
  const live2dApp = useRef<any>(null);
  const currentModel = useRef<any>(null);

  // 创建简化角色
  const createSimpleCharacter = useCallback(
    (app: any, scale: number, position: [number, number]) => {
      const container = new app.Container();

      // 身体
      const body = new app.Graphics();
      body.beginFill(0x4a90e2);
      body.drawRoundedRect(-40, -60, 80, 120, 20);
      body.endFill();

      // 头部
      const head = new app.Graphics();
      head.beginFill(0x66ccff);
      head.drawCircle(0, -120, 40);
      head.endFill();

      // 眼睛
      const leftEye = new app.Graphics();
      leftEye.beginFill(0x000000);
      leftEye.drawCircle(-15, -130, 5);
      leftEye.endFill();

      const rightEye = new app.Graphics();
      rightEye.beginFill(0x000000);
      rightEye.drawCircle(15, -130, 5);
      rightEye.endFill();

      // 嘴巴
      const mouth = new app.Graphics();
      mouth.beginFill(0xff6b6b);
      mouth.drawEllipse(0, -110, 15, 8);
      mouth.endFill();

      container.addChild(body, head, leftEye, rightEye, mouth);
      container.scale.set(scale);
      container.position.set(
        position[0] + app.screen.width / 2,
        position[1] + app.screen.height,
      );

      const character: SimpleCharacter = {
        graphics: container,
        animate: (action: string) => {
          // 简单的动画效果
          const colors: Record<string, number> = {
            happy: 0xffff66,
            sad: 0x6666ff,
            thinking: 0x9966ff,
            speaking: 0x66ff66,
            surprised: 0xff66ff,
            angry: 0xff6666,
            default: 0x66ccff,
          };

          const color = colors[action] || colors.default;
          head.tint = color;

          // 添加简单的缩放动画
          container.scale.set(scale * 1.1);
          setTimeout(() => {
            container.scale.set(scale);
            head.tint = 0xffffff;
          }, 500);
        },
        setExpression: (expression: string) => {
          // 根据表情改变眼睛和嘴巴
          switch (expression) {
            case "happy":
              mouth.tint = 0xffff66;
              break;
            case "sad":
              mouth.tint = 0x6666ff;
              break;
            case "angry":
              leftEye.tint = 0xff0000;
              rightEye.tint = 0xff0000;
              break;
            default:
              mouth.tint = 0xffffff;
              leftEye.tint = 0xffffff;
              rightEye.tint = 0xffffff;
          }

          setTimeout(() => {
            mouth.tint = 0xffffff;
            leftEye.tint = 0xffffff;
            rightEye.tint = 0xffffff;
          }, 1000);
        },
      };

      return character;
    },
    [],
  );

  // 初始化Live2D
  const initializeLive2D = useCallback(async () => {
    if (!canvasRef.current) return;

    setIsLoading(true);
    setError(null);

    try {
      // 检查是否应该使用真实模型
      if (!useSimpleCharacter && appConfig && modelStatus?.status === "ready") {
        // 尝试加载真实Live2D模型
        console.log("尝试加载真实Live2D模型:", appConfig.model_path);
        // 这里可以扩展支持真实的Live2D模型
        // 暂时还是使用简化角色
      }

      // 动态导入PIXI.js
      const PIXI = await import("pixi.js");

      // 创建PIXI应用
      const app = new PIXI.Application({
        view: canvasRef.current,
        autoStart: true,
        backgroundAlpha: 0,
        width: canvasRef.current.clientWidth,
        height: canvasRef.current.clientHeight,
      });

      live2dApp.current = app;

      // 创建角色（使用配置中的参数）
      const configScale = appConfig?.scale || scale;
      const configPosition: [number, number] = appConfig
        ? [appConfig.position_x, appConfig.position_y]
        : position;

      const character = createSimpleCharacter(
        PIXI,
        configScale,
        configPosition,
      );
      currentModel.current = character;

      // 添加到舞台
      app.stage.addChild(character.graphics);

      // 添加交互
      character.graphics.interactive = true;
      character.graphics.buttonMode = true;
      character.graphics.on("pointerdown", () => {
        handleModelTap();
      });

      setIsLoaded(true);
      console.log(
        useSimpleCharacter ? "简化Live2D角色创建成功" : "Live2D角色创建成功",
      );
    } catch (err) {
      console.error("Live2D 初始化失败:", err);
      setError(err instanceof Error ? err.message : "未知错误");
    } finally {
      setIsLoading(false);
    }
  }, [
    modelPath,
    scale,
    position,
    config,
    createSimpleCharacter,
    useSimpleCharacter,
    appConfig,
    modelStatus,
  ]);

  // 处理模型点击
  const handleModelTap = useCallback(async () => {
    try {
      await invoke("execute_live2d_action_by_type", {
        actionType: "happy",
      });
    } catch (err) {
      console.error("执行Live2D动作失败:", err);
    }
  }, []);

  // 执行动作
  const executeAction = useCallback(
    (action: Live2DAction) => {
      if (!currentModel.current || !isLoaded) return;

      try {
        const character = currentModel.current as SimpleCharacter;
        console.log("执行Live2D动作:", action.action_type);

        // 执行动画
        character.animate(action.action_type);

        // 设置表情
        if (action.expression) {
          character.setExpression(action.expression);
          setCurrentExpression(action.expression);
          onExpressionChange?.(action.expression);
        }

        onActionChange?.(action);
      } catch (err) {
        console.error("执行动作失败:", err);
      }
    },
    [isLoaded, onActionChange, onExpressionChange],
  );

  // 设置表情
  const setExpression = useCallback(
    (expression: string) => {
      if (!currentModel.current || !isLoaded) return;

      try {
        const character = currentModel.current as SimpleCharacter;
        console.log("设置Live2D表情:", expression);

        character.setExpression(expression);
        setCurrentExpression(expression);
        onExpressionChange?.(expression);
      } catch (error) {
        console.warn("Failed to set expression:", error);
      }
    },
    [isLoaded, onExpressionChange],
  );

  // 检查WebGL支持
  useEffect(() => {
    const supported = checkWebGLSupport();
    setWebglSupported(supported);
    if (!supported) {
      console.warn("WebGL不支持，Live2D功能可能无法正常使用");
    }
  }, []);

  // 根据模型状态决定使用模式
  useEffect(() => {
    if (modelStatus) {
      if (needsSetup && !canUseFallback) {
        setShowModelSetup(true);
        setUseSimpleCharacter(false);
      } else if (needsSetup && canUseFallback) {
        setUseSimpleCharacter(true);
        setShowModelSetup(false);
      } else if (isModelValid) {
        setUseSimpleCharacter(false);
        setShowModelSetup(false);
      }
    }
  }, [modelStatus, needsSetup, canUseFallback, isModelValid]);

  // 加载状态
  const loadState = useCallback(async () => {
    try {
      const live2dState = await invoke<Live2DState>("get_live2d_state");
      setState(live2dState);
    } catch (err) {
      console.error("加载Live2D状态失败:", err);
    }
  }, []);

  // 监听Live2D事件
  useEffect(() => {
    const unlistenAction = listen<Live2DEvent>("live2d_action", (event) => {
      const { action } = event.payload;
      executeAction(action);
    });

    const unlistenExpression = listen<Live2DEvent>(
      "live2d_expression",
      (event) => {
        const { action } = event.payload;
        if (action.expression) {
          setExpression(action.expression);
        }
      },
    );

    const unlistenAgentTrigger = listen<any>(
      "agent_live2d_trigger",
      (event) => {
        const { action } = event.payload;
        if (action) {
          invoke("execute_live2d_action_by_type", {
            actionType: action,
          }).catch((err) => {
            console.error("执行Agent触发的Live2D动作失败:", err);
          });
        }
      },
    );

    return () => {
      unlistenAction.then((fn) => fn());
      unlistenExpression.then((fn) => fn());
      unlistenAgentTrigger.then((fn) => fn());
    };
  }, [executeAction, setExpression]);

  // 组件初始化
  useEffect(() => {
    // 加载Live2D配置
    loadConfig();
    loadState();
  }, [loadConfig, loadState]);

  // 初始化Live2D
  useEffect(() => {
    if (appConfig?.enabled && canvasRef.current && modelStatus) {
      initializeLive2D();
    }

    return () => {
      // 清理资源
      if (live2dApp.current) {
        live2dApp.current.destroy(true);
        live2dApp.current = null;
      }
      currentModel.current = null;
    };
  }, [config, initializeLive2D, appConfig, modelStatus]);

  // 处理画布大小变化
  useEffect(() => {
    const handleResize = () => {
      if (live2dApp.current && canvasRef.current) {
        const canvas = canvasRef.current;
        live2dApp.current.renderer.resize(
          canvas.clientWidth,
          canvas.clientHeight,
        );

        // 重新定位模型
        if (currentModel.current) {
          const model = currentModel.current;
          model.position.set(canvas.clientWidth / 2, canvas.clientHeight);
        }
      }
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  // 定期处理动作队列
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        await invoke("process_live2d_action_queue");
        loadState();
      } catch (err) {
        console.error("处理Live2D动作队列失败:", err);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [loadState]);

  // 处理禁用Live2D
  const handleDisableLive2D = useCallback(async () => {
    try {
      await disableLive2D();
      setIsLoaded(false);
      setShowModelSetup(false);
    } catch (err) {
      console.error("禁用Live2D失败:", err);
    }
  }, [disableLive2D]);

  // 处理启用Live2D
  const handleEnableLive2D = useCallback(async () => {
    try {
      await enableLive2D();
    } catch (err) {
      console.error("启用Live2D失败:", err);
    }
  }, [enableLive2D]);

  // 渲染模型设置界面
  const renderModelSetup = () => {
    if (!showModelSetup || !modelStatus) return null;

    return (
      <div className="absolute inset-0 bg-white bg-opacity-95 flex items-center justify-center z-10">
        <div className="bg-white rounded-lg shadow-lg p-6 max-w-md w-full mx-4">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold flex items-center">
              <AlertTriangle className="h-5 w-5 text-yellow-500 mr-2" />
              Live2D模型设置
            </h3>
            <button
              onClick={() => setShowModelSetup(false)}
              className="text-gray-400 hover:text-gray-600"
            >
              <X className="h-5 w-5" />
            </button>
          </div>

          <div className="mb-4">
            <p className="text-sm text-gray-600 mb-2">{modelStatus.message}</p>
            {modelStatus.error_details && (
              <p className="text-xs text-red-600 bg-red-50 p-2 rounded">
                {modelStatus.error_details}
              </p>
            )}
          </div>

          {modelStatus.download_suggestions && (
            <div className="mb-4">
              <h4 className="text-sm font-medium mb-2">推荐下载:</h4>
              {modelStatus.download_suggestions.map((suggestion, index) => (
                <div key={index} className="border rounded p-3 mb-2">
                  <div className="flex items-center justify-between">
                    <div>
                      <h5 className="font-medium">{suggestion.name}</h5>
                      <p className="text-xs text-gray-600">
                        {suggestion.description}
                      </p>
                    </div>
                    <a
                      href={suggestion.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center text-blue-500 hover:text-blue-700"
                    >
                      <Download className="h-4 w-4 mr-1" />
                      下载
                      <ExternalLink className="h-3 w-3 ml-1" />
                    </a>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="flex space-x-2">
            {modelStatus.can_fallback && (
              <button
                onClick={() => {
                  setUseSimpleCharacter(true);
                  setShowModelSetup(false);
                }}
                className="flex-1 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                使用简化角色
              </button>
            )}
            <button
              onClick={() => {
                handleDisableLive2D();
                setShowModelSetup(false);
              }}
              className="flex-1 px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
            >
              禁用Live2D
            </button>
          </div>
        </div>
      </div>
    );
  };

  // 渲染Live2D禁用状态
  const renderDisabledState = () => {
    if (appConfig?.enabled !== false) return null;

    return (
      <div className="flex items-center justify-center w-full h-full">
        <div className="text-center">
          <div className="text-gray-400 mb-4">Live2D功能已禁用</div>
          <button
            onClick={handleEnableLive2D}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            启用Live2D
          </button>
        </div>
      </div>
    );
  };

  return (
    <div className={`live2d-container relative ${className}`}>
      {/* 模型设置界面 */}
      {renderModelSetup()}

      <canvas
        ref={canvasRef}
        className="live2d-canvas w-full h-full"
        style={{
          display: isLoaded ? "block" : "none",
          cursor: "pointer",
        }}
      />

      {(isLoading || isConfigLoading) && (
        <div className="live2d-loading flex items-center justify-center w-full h-full">
          <div className="flex flex-col items-center space-y-2">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
            <span className="text-sm text-gray-500">
              {isConfigLoading ? "加载配置..." : "初始化Live2D角色..."}
            </span>
          </div>
        </div>
      )}

      {(error || configError) && (
        <div className="live2d-error flex items-center justify-center w-full h-full">
          <div className="text-center">
            <div className="text-red-500 mb-2">Live2D加载失败</div>
            <div className="text-sm text-gray-500">{error || configError}</div>
            <div className="mt-2 space-x-2">
              <button
                onClick={initializeLive2D}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                重试
              </button>
              <button
                onClick={() => setShowModelSetup(true)}
                className="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
              >
                设置
              </button>
            </div>
            {!webglSupported && (
              <div className="mt-2 text-xs text-orange-600 bg-orange-50 p-2 rounded">
                ⚠️ 您的浏览器不支持WebGL，Live2D功能可能无法正常使用
              </div>
            )}
          </div>
        </div>
      )}

      {/* 禁用状态 */}
      {renderDisabledState()}

      {!isLoaded &&
        !isLoading &&
        !error &&
        !isConfigLoading &&
        appConfig?.enabled && (
          <div className="live2d-placeholder flex items-center justify-center w-full h-full">
            <div className="text-center">
              <div className="text-gray-400 mb-2">Live2D角色</div>
              <button
                onClick={initializeLive2D}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                创建角色
              </button>
            </div>
          </div>
        )}

      {/* 调试信息 */}
      {import.meta.env.DEV && (
        <div className="live2d-debug absolute top-2 left-2 bg-black bg-opacity-50 text-white p-2 rounded text-xs">
          {state && (
            <>
              <div>说话状态: {state.is_speaking ? "是" : "否"}</div>
              <div>当前动作: {state.current_action?.action_type || "无"}</div>
              <div>队列长度: {state.action_queue.length}</div>
              <div>当前表情: {currentExpression}</div>
            </>
          )}
          <div>模式: {useSimpleCharacter ? "简化角色" : "Live2D模型"}</div>
          <div>状态: {modelStatus?.status || "未知"}</div>
          <div>启用: {appConfig?.enabled ? "是" : "否"}</div>
        </div>
      )}
    </div>
  );
};

export default Live2D;
