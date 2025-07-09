import React, { useEffect, useRef, useState } from "react";
import * as live2d from "live2d-render";
import { useLive2DConfig } from "../hooks/useLive2DConfig";

interface Live2DRenderProps {
  modelPath?: string;
  width?: number;
  height?: number;
  scale?: number;
  showToolBox?: boolean;
  loadFromCache?: boolean;
  className?: string;
  onModelLoad?: () => void;
  onModelError?: (error: string) => void;
}

const Live2DRender: React.FC<Live2DRenderProps> = ({
  modelPath = "whitecatfree_vts/sdwhite cat free.model3.json",
  width = 400,
  height = 500,
  scale = 1.0,
  showToolBox = true,
  loadFromCache = true,
  className = "",
  onModelLoad,
  onModelError,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [, setIsLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 新增：从后端获取Live2D配置
  const {
    config: live2dConfig,
    isLoading: configLoading,
    error: configError,
  } = useLive2DConfig();

  // 计算最终模型路径和参数
  const finalModelPath = live2dConfig?.model_path || modelPath;
  console.log("Final Model Path:", finalModelPath);
  console.log("Live2D Config:", live2dConfig);
  const finalWidth = width;
  const finalHeight = height;
  const finalScale = live2dConfig?.scale || scale;

  useEffect(() => {
    if (!live2dConfig) return;
    const initializeLive2DModel = async () => {
      if (!containerRef.current) return;
      try {
        setIsLoading(true);
        setError(null);
        containerRef.current.innerHTML = "";
        await live2d.initializeLive2D({
          BackgroundRGBA: [0.0, 0.0, 0.0, 0.0],
          ResourcesPath: finalModelPath,
          CanvasSize: {
            height: finalHeight,
            width: finalWidth,
          },
          ShowToolBox: showToolBox,
          LoadFromCache: loadFromCache,
          MinifiedJSUrl: "https://unpkg.com/core-js-bundle@3.6.1/minified.js",
          Live2dCubismcoreUrl:
            "https://cubism.live2d.com/sdk-web/cubismcore/live2dcubismcore.min.js",
          CanvasId: "default",
        });
        setIsLoaded(true);
        onModelLoad?.();
        console.log("Live2D 模型加载完成");
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Live2D 模型加载失败";
        setError(errorMessage);
        onModelError?.(errorMessage);
        console.error("Live2D 初始化失败:", err);
      } finally {
        setIsLoading(false);
      }
    };
    initializeLive2DModel();
    return () => {
      if (containerRef.current) {
        containerRef.current.innerHTML = "";
      }
    };
  }, [
    finalModelPath,
    finalWidth,
    finalHeight,
    finalScale,
    showToolBox,
    loadFromCache,
    live2dConfig,
  ]);

  // 公开的方法
  // const triggerMotion = (groupName: string, motionIndex?: number) => {
  //   if (!isLoaded) return;

  //   try {
  //     // 这里可能需要根据 live2d-render 的 API 调整
  //     // live2d.triggerMotion?.(groupName, motionIndex);
  //     console.log(`触发动作: ${groupName}, 索引: ${motionIndex}`);
  //   } catch (err) {
  //     console.error("触发动作失败:", err);
  //   }
  // };

  // const setExpression = (expressionName: string) => {
  //   if (!isLoaded) return;

  //   try {
  //     // 这里可能需要根据 live2d-render 的 API 调整
  //     // live2d.setExpression?.(expressionName);
  //     console.log(`设置表情: ${expressionName}`);
  //   } catch (err) {
  //     console.error("设置表情失败:", err);
  //   }
  // };

  // const startSpeaking = () => {
  //   triggerMotion("Speaking");
  // };

  // const stopSpeaking = () => {
  //   triggerMotion("Idle");
  // };

  return (
    <div
      className={`live2d-container ${className}`}
      style={{
        width: `${finalWidth}px`,
        height: `${finalHeight}px`,
        position: "relative",
        overflow: "hidden",
      }}
    >
      {(isLoading || configLoading) && (
        <div className="absolute inset-0 flex items-center justify-center bg-gray-100 bg-opacity-50">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto mb-2"></div>
            <p className="text-sm text-gray-600">加载 Live2D 配置/模型中...</p>
          </div>
        </div>
      )}
      {(error || configError) && (
        <div className="absolute inset-0 flex items-center justify-center bg-red-50">
          <div className="text-center p-4">
            <div className="text-red-500 mb-2">⚠️</div>
            <p className="text-sm text-red-600">模型加载失败</p>
            <p className="text-xs text-red-500 mt-1">{error || configError}</p>
            <button
              onClick={() => window.location.reload()}
              className="mt-2 px-3 py-1 bg-red-500 text-white text-xs rounded hover:bg-red-600"
            >
              重新加载
            </button>
          </div>
        </div>
      )}
      <div
        ref={containerRef}
        className="w-full h-full"
        style={{
          width: "100%",
          height: "100%",
        }}
      />
    </div>
  );
};

export default Live2DRender;
