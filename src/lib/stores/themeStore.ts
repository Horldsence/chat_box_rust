import { writable, derived, type Writable } from "svelte/store";
import { browser } from "$app/environment";
import type { Theme } from "$lib/types";

// 主题类型定义
export type ThemeMode = "light" | "dark" | "auto";

interface ThemeState {
  mode: ThemeMode;
  current: "light" | "dark";
  systemPreference: "light" | "dark";
}

// 创建主题状态存储
function createThemeStore() {
  // 初始状态
  const initialState: ThemeState = {
    mode: "auto",
    current: "light",
    systemPreference: "light",
  };

  const { subscribe, set, update } = writable<ThemeState>(initialState);

  return {
    subscribe,

    // 设置主题模式
    setMode: (mode: ThemeMode) => {
      update((state) => {
        const newState = { ...state, mode };

        // 计算当前主题
        if (mode === "auto") {
          newState.current = state.systemPreference;
        } else {
          newState.current = mode;
        }

        // 保存到 localStorage
        if (browser) {
          localStorage.setItem("theme-mode", mode);
          document.documentElement.setAttribute("data-theme", newState.current);
        }

        return newState;
      });
    },

    // 切换主题
    toggle: () => {
      update((state) => {
        let newMode: ThemeMode;

        if (state.mode === "auto") {
          // 从自动模式切换到与系统偏好相反的主题
          newMode = state.systemPreference === "light" ? "dark" : "light";
        } else if (state.mode === "light") {
          newMode = "dark";
        } else {
          newMode = "light";
        }

        const newState = {
          ...state,
          mode: newMode,
          current: newMode,
        };

        // 保存到 localStorage
        if (browser) {
          localStorage.setItem("theme-mode", newMode);
          document.documentElement.setAttribute("data-theme", newState.current);
        }

        return newState;
      });
    },

    // 更新系统偏好
    updateSystemPreference: (preference: "light" | "dark") => {
      update((state) => {
        const newState = { ...state, systemPreference: preference };

        // 如果当前是自动模式，更新当前主题
        if (state.mode === "auto") {
          newState.current = preference;

          if (browser) {
            document.documentElement.setAttribute("data-theme", preference);
          }
        }

        return newState;
      });
    },

    // 初始化主题
    init: () => {
      if (!browser) return;

      // 检测系统主题偏好
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const systemPreference: "light" | "dark" = mediaQuery.matches ? "dark" : "light";

      // 从 localStorage 读取保存的主题模式
      const savedMode = localStorage.getItem("theme-mode") as ThemeMode | null;
      const mode = savedMode || "auto";

      // 计算当前主题
      const current = mode === "auto" ? systemPreference : mode;

      // 更新状态
      set({
        mode,
        current,
        systemPreference,
      });

      // 应用主题到文档
      document.documentElement.setAttribute("data-theme", current);

      // 监听系统主题变化
      const handleSystemThemeChange = (e: MediaQueryListEvent) => {
        const newSystemPreference: "light" | "dark" = e.matches ? "dark" : "light";
        update((state) => {
          const newState = { ...state, systemPreference: newSystemPreference };

          // 如果当前是自动模式，更新当前主题
          if (state.mode === "auto") {
            newState.current = newSystemPreference;
            document.documentElement.setAttribute("data-theme", newSystemPreference);
          }

          return newState;
        });
      };

      mediaQuery.addEventListener("change", handleSystemThemeChange);

      // 返回清理函数
      return () => {
        mediaQuery.removeEventListener("change", handleSystemThemeChange);
      };
    },

    // 重置为默认主题
    reset: () => {
      if (browser) {
        localStorage.removeItem("theme-mode");
      }

      const systemPreference = browser
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : "light";

      set({
        mode: "auto",
        current: systemPreference,
        systemPreference,
      });

      if (browser) {
        document.documentElement.setAttribute("data-theme", systemPreference);
      }
    },
  };
}

// 创建主题存储实例
export const themeStore = createThemeStore();

// 派生存储：仅当前主题
export const currentTheme = derived(themeStore, ($themeStore) => $themeStore.current);

// 派生存储：是否为暗色主题
export const isDark = derived(currentTheme, ($currentTheme) => $currentTheme === "dark");

// 派生存储：主题模式
export const themeMode = derived(themeStore, ($themeStore) => $themeStore.mode);

// 派生存储：系统偏好
export const systemPreference = derived(themeStore, ($themeStore) => $themeStore.systemPreference);

// 工具函数：获取主题类名
export const getThemeClass = (theme: "light" | "dark"): string => {
  return `theme-${theme}`;
};

// 工具函数：获取主题CSS变量
export const getThemeVariables = (theme: "light" | "dark") => {
  const lightTheme = {
    "--color-bg-primary": "#ffffff",
    "--color-bg-secondary": "#f7fafc",
    "--color-bg-tertiary": "#edf2f7",
    "--color-text-primary": "#2d3748",
    "--color-text-secondary": "#4a5568",
    "--color-text-muted": "#718096",
    "--color-border-light": "#e2e8f0",
    "--color-border-medium": "#cbd5e0",
    "--color-border-dark": "#a0aec0",
  };

  const darkTheme = {
    "--color-bg-primary": "#1a202c",
    "--color-bg-secondary": "#2d3748",
    "--color-bg-tertiary": "#4a5568",
    "--color-text-primary": "#f7fafc",
    "--color-text-secondary": "#e2e8f0",
    "--color-text-muted": "#a0aec0",
    "--color-border-light": "#4a5568",
    "--color-border-medium": "#718096",
    "--color-border-dark": "#a0aec0",
  };

  return theme === "dark" ? darkTheme : lightTheme;
};

// 工具函数：应用主题变量到元素
export const applyThemeVariables = (element: HTMLElement, theme: "light" | "dark") => {
  const variables = getThemeVariables(theme);
  Object.entries(variables).forEach(([key, value]) => {
    element.style.setProperty(key, value);
  });
};

// 工具函数：检测是否支持暗色主题
export const supportsDarkMode = (): boolean => {
  if (!browser) return false;
  return (
    window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches !== undefined
  );
};

// 主题配置选项
export const themeOptions: Array<{ value: ThemeMode; label: string; icon: string }> = [
  { value: "light", label: "浅色主题", icon: "☀️" },
  { value: "dark", label: "深色主题", icon: "🌙" },
  { value: "auto", label: "跟随系统", icon: "🔄" },
];

// 导出类型
export type { ThemeState };
