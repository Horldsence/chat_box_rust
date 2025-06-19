import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

// Live2D配置类型
export interface Live2DConfig {
  enabled: boolean;
  model_path: string;
  model_name: string;
  scale: number;
  position_x: number;
  position_y: number;
  auto_blink: boolean;
  auto_breath: boolean;
  check_model_on_startup: boolean;
  fallback_to_simple_character: boolean;
}

// 模型状态类型
export interface ModelStatus {
  config: Live2DConfig;
  enabled: boolean;
  status: string;
  message: string;
  action_required?: string;
  download_suggestions?: Array<{
    name: string;
    description: string;
    url: string;
    file_name: string;
    recommended: string;
  }>;
  model_check?: any;
  can_fallback: boolean;
  error_details?: string;
}

// 模型检查结果类型
export interface ModelCheckResult {
  model_path: string;
  exists: boolean;
  valid_extension: boolean;
  readable: boolean;
  valid_json: boolean;
  valid_model: boolean;
  all_files_exist: boolean;
  missing_files?: string[];
  status: string;
  error?: string;
  can_fallback: boolean;
}

// 环境检查结果类型
export interface EnvironmentCheck {
  models_dir: string;
  models_dir_exists: boolean;
  available_models: Array<{
    name: string;
    path: string;
    full_path: string;
  }>;
  models_count: number;
  webgl_check_required: boolean;
}

// Hook返回类型
export interface UseLive2DConfigReturn {
  // 状态
  config: Live2DConfig | null;
  modelStatus: ModelStatus | null;
  environment: EnvironmentCheck | null;
  isLoading: boolean;
  error: string | null;

  // 操作函数
  loadConfig: () => Promise<void>;
  updateConfig: (newConfig: Live2DConfig) => Promise<void>;
  checkModelStatus: () => Promise<ModelStatus | null>;
  checkModel: (modelPath: string) => Promise<ModelCheckResult | null>;
  checkEnvironment: () => Promise<EnvironmentCheck | null>;
  enableLive2D: () => Promise<void>;
  disableLive2D: () => Promise<void>;

  // 便捷方法
  isModelValid: boolean;
  canUseFallback: boolean;
  needsSetup: boolean;
  isReady: boolean;
}

export const useLive2DConfig = (): UseLive2DConfigReturn => {
  // 状态管理
  const [config, setConfig] = useState<Live2DConfig | null>(null);
  const [modelStatus, setModelStatus] = useState<ModelStatus | null>(null);
  const [environment, setEnvironment] = useState<EnvironmentCheck | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载配置
  const loadConfig = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const live2dConfig = await invoke<Live2DConfig>(
        "get_live2d_config_from_file",
      );
      setConfig(live2dConfig);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "加载配置失败";
      setError(errorMessage);
      console.error("加载Live2D配置失败:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 更新配置
  const updateConfig = useCallback(async (newConfig: Live2DConfig) => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke("update_live2d_config_in_file", { live2dConfig: newConfig });
      setConfig(newConfig);

      // 重新检查模型状态
      await checkModelStatus();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "更新配置失败";
      setError(errorMessage);
      console.error("更新Live2D配置失败:", err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 检查模型状态
  const checkModelStatus =
    useCallback(async (): Promise<ModelStatus | null> => {
      try {
        const status = await invoke<ModelStatus>("get_live2d_model_status");
        setModelStatus(status);
        return status;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "检查模型状态失败";
        setError(errorMessage);
        console.error("检查模型状态失败:", err);
        return null;
      }
    }, []);

  // 检查指定模型
  const checkModel = useCallback(
    async (modelPath: string): Promise<ModelCheckResult | null> => {
      try {
        const result = await invoke<ModelCheckResult>("check_live2d_model", {
          modelPath,
        });
        return result;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "检查模型失败";
        setError(errorMessage);
        console.error("检查模型失败:", err);
        return null;
      }
    },
    [],
  );

  // 检查环境
  const checkEnvironment =
    useCallback(async (): Promise<EnvironmentCheck | null> => {
      try {
        const env = await invoke<EnvironmentCheck>("check_live2d_environment");
        setEnvironment(env);
        return env;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "检查环境失败";
        setError(errorMessage);
        console.error("检查Live2D环境失败:", err);
        return null;
      }
    }, []);

  // 启用Live2D
  const enableLive2D = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke("enable_live2d");
      await loadConfig();
      await checkModelStatus();
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "启用Live2D失败";
      setError(errorMessage);
      console.error("启用Live2D失败:", err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [loadConfig, checkModelStatus]);

  // 禁用Live2D
  const disableLive2D = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke("disable_live2d");
      await loadConfig();
      setModelStatus(null);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "禁用Live2D失败";
      setError(errorMessage);
      console.error("禁用Live2D失败:", err);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [loadConfig]);

  // 计算便捷属性
  const isModelValid = modelStatus?.status === "ready";
  const canUseFallback = modelStatus?.can_fallback || false;
  const needsSetup = ["model_not_found", "model_invalid"].includes(
    modelStatus?.status || "",
  );
  const isReady = !!(config?.enabled && (isModelValid || canUseFallback));

  // 初始化时加载配置
  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  // 配置变化时检查模型状态
  useEffect(() => {
    if (config?.enabled && config?.check_model_on_startup) {
      checkModelStatus();
    }
  }, [config, checkModelStatus]);

  // 错误自动清除
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        setError(null);
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  return {
    // 状态
    config,
    modelStatus,
    environment,
    isLoading,
    error,

    // 操作函数
    loadConfig,
    updateConfig,
    checkModelStatus,
    checkModel,
    checkEnvironment,
    enableLive2D,
    disableLive2D,

    // 便捷属性
    isModelValid,
    canUseFallback,
    needsSetup,
    isReady,
  };
};

// 默认导出
export default useLive2DConfig;

// 便捷函数：WebGL支持检查
export const checkWebGLSupport = (): boolean => {
  try {
    const canvas = document.createElement("canvas");
    const gl =
      canvas.getContext("webgl") || canvas.getContext("experimental-webgl");
    return !!gl;
  } catch (e) {
    return false;
  }
};

// 便捷函数：获取推荐模型路径
export const getRecommendedModelPath = (): string => {
  return "models/live2d/hiyori/hiyori_free_en.model3.json";
};

// 便捷函数：验证模型路径格式
export const isValidModelPath = (path: string): boolean => {
  return path.endsWith(".model3.json") || path.endsWith(".model.json");
};

// 便捷函数：格式化错误消息
export const formatErrorMessage = (
  error: string,
  modelPath?: string,
): string => {
  const messages: Record<string, string> = {
    model_not_found: `模型文件未找到${modelPath ? `: ${modelPath}` : ""}`,
    model_invalid: "模型文件无效或损坏",
    invalid_structure: "模型文件结构不正确",
    incomplete: "模型文件不完整，缺少相关文件",
    disabled: "Live2D功能已禁用",
  };

  return messages[error] || error;
};

// 便捷函数：获取状态颜色
export const getStatusColor = (status: string): string => {
  const colors: Record<string, string> = {
    ready: "text-green-600",
    model_not_found: "text-red-600",
    model_invalid: "text-red-600",
    disabled: "text-gray-600",
    incomplete: "text-yellow-600",
  };

  return colors[status] || "text-gray-600";
};

// 便捷函数：获取状态图标
export const getStatusIcon = (status: string): string => {
  const icons: Record<string, string> = {
    ready: "✅",
    model_not_found: "❌",
    model_invalid: "❌",
    disabled: "⏹️",
    incomplete: "⚠️",
  };

  return icons[status] || "❓";
};
