import { useState, useEffect, useCallback, useRef } from "react";
import { settingsApi, dialogApi, utils } from "../utils/api";
import type {
  AppConfig,
  SystemInfo,
  HealthStatus,
  Theme,
  ConfigFormat,
} from "../types";

export interface UseSettingsOptions {
  autoLoad?: boolean;
  enableHealthCheck?: boolean;
  healthCheckInterval?: number; // in milliseconds
}

export interface SettingsState {
  config: AppConfig | null;
  systemInfo: SystemInfo | null;
  healthStatus: HealthStatus | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  isOnline: boolean;
  theme: Theme;
}

export interface SettingsActions {
  // Configuration management
  loadConfig: () => Promise<void>;
  saveConfig: (config: AppConfig) => Promise<boolean>;
  resetConfig: () => Promise<boolean>;
  updateConfig: (updates: Partial<AppConfig>) => Promise<boolean>;

  // System information
  loadSystemInfo: () => Promise<void>;
  loadHealthStatus: () => Promise<void>;
  checkConnection: () => Promise<boolean>;

  // Import/Export
  importConfig: () => Promise<boolean>;
  exportConfig: (format?: ConfigFormat) => Promise<boolean>;

  // Notifications and logging
  showNotification: (
    title: string,
    body: string,
    icon?: string,
  ) => Promise<void>;
  logError: (code: string, message: string, details?: string) => Promise<void>;
  logWarning: (message: string, details?: string) => Promise<void>;
  logInfo: (message: string, details?: string) => Promise<void>;

  // Theme management
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;

  // Utility actions
  clearError: () => void;
  refresh: () => Promise<void>;
}

const DEFAULT_CONFIG: Partial<AppConfig> = {
  app_behavior: {
    message_chunk_buffer_size: 10,
    message_chunk_send_interval_ms: 100,
  },
};

export function useSettings(
  options: UseSettingsOptions = {},
): [SettingsState, SettingsActions] {
  const {
    autoLoad = true,
    enableHealthCheck = true,
    healthCheckInterval = 30000, // 30 seconds
  } = options;

  // State
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isOnline, setIsOnline] = useState(true);
  const [theme, setThemeState] = useState<Theme>(() => {
    // Initialize theme from localStorage or system preference
    const stored = localStorage.getItem("chat-app-theme") as Theme;
    if (stored && ["light", "dark", "system"].includes(stored)) {
      return stored;
    }
    return "system";
  });

  // Refs for intervals and cleanup
  const healthCheckInterval_ref = useRef<number | null>(null);

  // Apply theme to document
  useEffect(() => {
    const root = document.documentElement;

    if (theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const applySystemTheme = () => {
        root.classList.toggle("dark", mediaQuery.matches);
      };

      applySystemTheme();
      mediaQuery.addEventListener("change", applySystemTheme);

      return () => mediaQuery.removeEventListener("change", applySystemTheme);
    } else {
      root.classList.toggle("dark", theme === "dark");
    }

    localStorage.setItem("chat-app-theme", theme);
  }, [theme]);

  // Load configuration
  const loadConfig = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await settingsApi.getConfig();
      if (response.success && response.data) {
        setConfig(response.data);
      } else {
        throw new Error(
          response.error?.message || "Failed to load configuration",
        );
      }
    } catch (err) {
      const errorMsg = "Failed to load application configuration";
      setError(errorMsg);
      await utils.handleApiError(err, errorMsg);
      // Set default config on error
      setConfig(DEFAULT_CONFIG as AppConfig);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Save configuration
  const saveConfig = useCallback(
    async (newConfig: AppConfig): Promise<boolean> => {
      setIsSaving(true);
      setError(null);
      try {
        const response = await settingsApi.saveConfig(newConfig);
        if (response.success) {
          setConfig(newConfig);
          await settingsApi.showNotification(
            "Settings Saved",
            "Your configuration has been saved successfully.",
          );
          return true;
        } else {
          throw new Error(
            response.error?.message || "Failed to save configuration",
          );
        }
      } catch (err) {
        const errorMsg = "Failed to save configuration";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
        return false;
      } finally {
        setIsSaving(false);
      }
    },
    [],
  );

  // Reset configuration
  const resetConfig = useCallback(async (): Promise<boolean> => {
    setIsSaving(true);
    setError(null);
    try {
      const response = await settingsApi.resetConfig();
      if (response.success && response.data) {
        setConfig(response.data);
        await settingsApi.showNotification(
          "Settings Reset",
          "Configuration has been reset to default values.",
        );
        return true;
      } else {
        throw new Error(
          response.error?.message || "Failed to reset configuration",
        );
      }
    } catch (err) {
      const errorMsg = "Failed to reset configuration";
      setError(errorMsg);
      await utils.handleApiError(err, errorMsg);
      return false;
    } finally {
      setIsSaving(false);
    }
  }, []);

  // Update configuration partially
  const updateConfig = useCallback(
    async (updates: Partial<AppConfig>): Promise<boolean> => {
      if (!config) {
        setError("No configuration loaded");
        return false;
      }

      const updatedConfig = { ...config, ...updates };
      return await saveConfig(updatedConfig);
    },
    [config, saveConfig],
  );

  // Load system information
  const loadSystemInfo = useCallback(async () => {
    setError(null);
    try {
      const response = await settingsApi.getSystemInfo();
      if (response.success && response.data) {
        setSystemInfo(response.data as SystemInfo);
      } else {
        throw new Error(
          response.error?.message || "Failed to load system information",
        );
      }
    } catch (err) {
      const errorMsg = "Failed to load system information";
      setError(errorMsg);
      await utils.handleApiError(err, errorMsg);
    }
  }, []);

  // Load health status
  const loadHealthStatus = useCallback(async () => {
    setError(null);
    try {
      const response = await settingsApi.getHealthStatus();
      if (response.success && response.data) {
        setHealthStatus(response.data as HealthStatus);
        setIsOnline(true);
      } else {
        throw new Error(
          response.error?.message || "Failed to load health status",
        );
      }
    } catch (err) {
      setIsOnline(false);
      await utils.handleApiError(err, "Health check failed");
    }
  }, []);

  // Check connection
  const checkConnection = useCallback(async (): Promise<boolean> => {
    try {
      const response = await settingsApi.ping();
      const isConnected = response.success && response.data === "pong";
      setIsOnline(isConnected);
      return isConnected;
    } catch {
      setIsOnline(false);
      return false;
    }
  }, []);

  // Import configuration
  const importConfig = useCallback(async (): Promise<boolean> => {
    setError(null);
    try {
      const response = await dialogApi.importConfig();
      if (response.success && response.data) {
        try {
          const importedConfig = JSON.parse(response.data) as AppConfig;
          const saved = await saveConfig(importedConfig);
          if (saved) {
            await settingsApi.showNotification(
              "Configuration Imported",
              "Settings have been imported successfully.",
            );
          }
          return saved;
        } catch (parseErr) {
          throw new Error("Invalid configuration file format");
        }
      }
      return false;
    } catch (err) {
      const errorMsg = "Failed to import configuration";
      setError(errorMsg);
      await utils.handleApiError(err, errorMsg);
      return false;
    }
  }, [saveConfig]);

  // Export configuration
  const exportConfig = useCallback(
    async (format: ConfigFormat = "json"): Promise<boolean> => {
      if (!config) {
        setError("No configuration to export");
        return false;
      }

      setError(null);
      try {
        const content =
          format === "json"
            ? JSON.stringify(config, null, 2)
            : JSON.stringify(config, null, 2); // Could implement YAML here if needed

        const response = await dialogApi.exportConfig(content, format);
        if (response.success && response.data) {
          await settingsApi.showNotification(
            "Configuration Exported",
            "Settings have been exported successfully.",
          );
          return true;
        }
        return false;
      } catch (err) {
        const errorMsg = "Failed to export configuration";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
        return false;
      }
    },
    [config],
  );

  // Notification wrapper
  const showNotification = useCallback(
    async (title: string, body: string, icon?: string) => {
      try {
        await settingsApi.showNotification(title, body, icon);
      } catch (err) {
        await utils.handleApiError(err, "Failed to show notification");
      }
    },
    [],
  );

  // Logging wrappers
  const logError = useCallback(
    async (code: string, message: string, details?: string) => {
      try {
        await settingsApi.logError(code, message, details);
      } catch (err) {
        console.error("Failed to log error:", err);
      }
    },
    [],
  );

  const logWarning = useCallback(async (message: string, details?: string) => {
    try {
      await settingsApi.logWarning(message, details);
    } catch (err) {
      console.error("Failed to log warning:", err);
    }
  }, []);

  const logInfo = useCallback(async (message: string, details?: string) => {
    try {
      await settingsApi.logInfo(message, details);
    } catch (err) {
      console.error("Failed to log info:", err);
    }
  }, []);

  // Theme management
  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme);
  }, []);

  const toggleTheme = useCallback(() => {
    setThemeState((current) => {
      if (current === "light") return "dark";
      if (current === "dark") return "system";
      return "light";
    });
  }, []);

  // Utility actions
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const refresh = useCallback(async () => {
    await Promise.all([loadConfig(), loadSystemInfo(), loadHealthStatus()]);
  }, [loadConfig, loadSystemInfo, loadHealthStatus]);

  // Setup health check interval
  useEffect(() => {
    if (!enableHealthCheck) return;

    const runHealthCheck = async () => {
      await loadHealthStatus();
    };

    // Initial health check
    runHealthCheck();

    // Setup interval
    healthCheckInterval_ref.current = setInterval(
      runHealthCheck,
      healthCheckInterval,
    ) as any;

    return () => {
      if (healthCheckInterval_ref.current) {
        clearInterval(healthCheckInterval_ref.current);
      }
    };
  }, [enableHealthCheck, healthCheckInterval, loadHealthStatus]);

  // Auto-load data on mount
  useEffect(() => {
    if (autoLoad) {
      refresh();
    }
  }, [autoLoad, refresh]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (healthCheckInterval_ref.current) {
        clearInterval(healthCheckInterval_ref.current);
      }
    };
  }, []);

  const state: SettingsState = {
    config,
    systemInfo,
    healthStatus,
    isLoading,
    isSaving,
    error,
    isOnline,
    theme,
  };

  const actions: SettingsActions = {
    loadConfig,
    saveConfig,
    resetConfig,
    updateConfig,
    loadSystemInfo,
    loadHealthStatus,
    checkConnection,
    importConfig,
    exportConfig,
    showNotification,
    logError,
    logWarning,
    logInfo,
    setTheme,
    toggleTheme,
    clearError,
    refresh,
  };

  return [state, actions];
}

// Simplified hook for basic settings
export function useBasicSettings() {
  return useSettings({
    autoLoad: true,
    enableHealthCheck: false,
  });
}

// Hook for settings with health monitoring
export function useSettingsWithHealth() {
  return useSettings({
    autoLoad: true,
    enableHealthCheck: true,
    healthCheckInterval: 30000,
  });
}
