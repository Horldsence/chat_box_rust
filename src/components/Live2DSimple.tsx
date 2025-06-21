import React, { useEffect, useRef, useState } from "react";

interface Live2DSimpleProps {
  className?: string;
  onReady?: () => void;
  onError?: (error: string) => void;
}

const Live2DSimple: React.FC<Live2DSimpleProps> = ({
  className = "",
  onReady,
  onError,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const appRef = useRef<any>(null);
  const characterRef = useRef<any>(null);

  // 创建简单的2D角色
  const createSimple2DCharacter = async () => {
    if (!canvasRef.current) return;

    try {
      console.log("开始创建简单2D角色...");

      // 动态导入PIXI.js
      const PIXI = await import("pixi.js");
      console.log("PIXI.js加载成功，版本:", PIXI.VERSION);

      const canvas = canvasRef.current;
      const width = canvas.clientWidth || 400;
      const height = canvas.clientHeight || 600;

      // 使用最简单的PIXI配置
      const app = new PIXI.Application({
        width,
        height,
        backgroundAlpha: 0,
        antialias: false,
      });

      // 手动挂载canvas
      if (canvas.parentNode) {
        canvas.parentNode.replaceChild(app.view as HTMLCanvasElement, canvas);
      }

      appRef.current = app;
      console.log("PIXI应用创建成功");

      // 创建角色容器
      const character = new PIXI.Container();

      // 创建身体 - 矩形
      const body = new PIXI.Graphics();
      body.beginFill(0x4A90E2);
      body.drawRoundedRect(-30, -50, 60, 100, 10);
      body.endFill();

      // 创建头部 - 圆形
      const head = new PIXI.Graphics();
      head.beginFill(0x66CCFF);
      head.drawCircle(0, -80, 25);
      head.endFill();

      // 创建眼睛
      const leftEye = new PIXI.Graphics();
      leftEye.beginFill(0x000000);
      leftEye.drawCircle(-10, -85, 3);
      leftEye.endFill();

      const rightEye = new PIXI.Graphics();
      rightEye.beginFill(0x000000);
      rightEye.drawCircle(10, -85, 3);
      rightEye.endFill();

      // 创建嘴巴
      const mouth = new PIXI.Graphics();
      mouth.beginFill(0xFF6B6B);
      mouth.drawEllipse(0, -75, 8, 4);
      mouth.endFill();

      // 组装角色
      character.addChild(body, head, leftEye, rightEye, mouth);

      // 设置位置到画布中心底部
      character.x = width / 2;
      character.y = height - 50;

      // 添加到舞台
      app.stage.addChild(character);

      // 添加交互
      character.interactive = true;
      character.cursor = "pointer";
      character.on("pointerdown", () => {
        console.log("角色被点击!");
        // 简单的点击动画
        character.scale.set(1.1);
        setTimeout(() => {
          character.scale.set(1.0);
        }, 200);
      });

      characterRef.current = character;
      console.log("角色创建完成");

      setIsLoaded(true);
      setError(null);
      onReady?.();

    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "创建角色失败";
      console.error("创建角色失败:", err);
      setError(errorMsg);
      onError?.(errorMsg);
    }
  };

  // 角色动作
  const playAction = (action: string) => {
    if (!characterRef.current || !isLoaded) return;

    const character = characterRef.current;

    switch (action) {
      case "happy":
        character.tint = 0xFFFF00; // 黄色
        break;
      case "sad":
        character.tint = 0x0000FF; // 蓝色
        break;
      case "angry":
        character.tint = 0xFF0000; // 红色
        break;
      default:
        character.tint = 0xFFFFFF; // 白色
    }

    // 0.5秒后恢复原色
    setTimeout(() => {
      character.tint = 0xFFFFFF;
    }, 500);
  };

  // 初始化
  useEffect(() => {
    createSimple2DCharacter();

    return () => {
      // 清理
      if (appRef.current) {
        appRef.current.destroy(true);
        appRef.current = null;
      }
      characterRef.current = null;
    };
  }, []);

  // 暴露方法到window对象用于测试
  useEffect(() => {
    if (typeof window !== 'undefined') {
      (window as any).live2dSimpleTest = {
        playAction,
        getStatus: () => ({ isLoaded, error }),
      };
    }
  }, [isLoaded, error]);

  return (
    <div className={`relative ${className}`}>
      <canvas
        ref={canvasRef}
        className="w-full h-full"
        style={{ display: isLoaded ? 'block' : 'none' }}
      />

      {!isLoaded && !error && (
        <div className="absolute inset-0 flex items-center justify-center bg-gray-100">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto mb-2"></div>
            <p className="text-sm text-gray-600">加载中...</p>
          </div>
        </div>
      )}

      {error && (
        <div className="absolute inset-0 flex items-center justify-center bg-red-50">
          <div className="text-center p-4">
            <p className="text-sm text-red-600 mb-2">加载失败</p>
            <p className="text-xs text-gray-500">{error}</p>
            <button
              onClick={createSimple2DCharacter}
              className="mt-2 px-3 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600"
            >
              重试
            </button>
          </div>
        </div>
      )}

      {isLoaded && (
        <div className="absolute top-2 left-2 z-10">
          <div className="flex gap-1">
            <button
              onClick={() => playAction("happy")}
              className="px-2 py-1 bg-yellow-500 text-white text-xs rounded hover:bg-yellow-600"
            >
              😊
            </button>
            <button
              onClick={() => playAction("sad")}
              className="px-2 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600"
            >
              😢
            </button>
            <button
              onClick={() => playAction("angry")}
              className="px-2 py-1 bg-red-500 text-white text-xs rounded hover:bg-red-600"
            >
              😡
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default Live2DSimple;
