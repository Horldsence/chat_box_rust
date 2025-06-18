import React, { useState, useEffect } from "react";
import {
  X,
  Monitor,
  Sun,
  Moon,
  Save,
  RotateCcw,
  Settings as SettingsIcon,
  Database,
  Mic,
  Bot,
  Info,
  CheckCircle,
  XCircle,
} from "lucide-react";
import { cn } from "../utils/cn";
import { useSettings } from "../hooks/useSettings";
import type { Theme } from "../types";

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

interface SettingsTabProps {
  label: string;
  icon: React.ReactNode;
  isActive: boolean;
  onClick: () => void;
}

function SettingsTab({ label, icon, isActive, onClick }: SettingsTabProps) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-3 w-full px-3 py-2 rounded-lg text-left transition-colors",
        isActive
          ? "bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400"
          : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700",
      )}
    >
      {icon}
      <span className="text-sm font-medium">{label}</span>
    </button>
  );
}

export function Settings({ isOpen, onClose }: SettingsProps) {
  const [activeTab, setActiveTab] = useState("general");
  const [settings, settingsActions] = useSettings();
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [localConfig, setLocalConfig] = useState(settings.config);

  const tabs = [
    {
      id: "general",
      label: "General",
      icon: <SettingsIcon size={16} />,
    },
    {
      id: "voice",
      label: "Voice",
      icon: <Mic size={16} />,
    },
    {
      id: "ai",
      label: "AI Model",
      icon: <Bot size={16} />,
    },
    {
      id: "database",
      label: "Database",
      icon: <Database size={16} />,
    },
    {
      id: "about",
      label: "About",
      icon: <Info size={16} />,
    },
  ];

  // Update local config when settings config changes
  useEffect(() => {
    if (settings.config) {
      setLocalConfig(settings.config);
      setHasUnsavedChanges(false);
    }
  }, [settings.config]);

  // Close settings with escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
      return () => document.removeEventListener("keydown", handleEscape);
    }
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleConfigChange = (updates: any) => {
    setLocalConfig((prev) => ({ ...prev, ...updates }));
    setHasUnsavedChanges(true);
  };

  const handleSave = async () => {
    if (localConfig) {
      const success = await settingsActions.saveConfig(localConfig);
      if (success) {
        setHasUnsavedChanges(false);
      }
    }
  };

  const handleReset = async () => {
    const confirmed = window.confirm(
      "Are you sure you want to reset all settings to default values?",
    );
    if (confirmed) {
      await settingsActions.resetConfig();
      setHasUnsavedChanges(false);
    }
  };

  const handleThemeChange = (theme: Theme) => {
    handleConfigChange({ theme });
  };

  const getStatusIcon = (status: boolean) => {
    return status ? (
      <CheckCircle size={16} className="text-green-500" />
    ) : (
      <XCircle size={16} className="text-red-500" />
    );
  };

  const renderGeneralSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Appearance
        </h3>
        <div className="space-y-3">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Theme
          </label>
          <div className="flex gap-2">
            {[
              {
                value: "light" as Theme,
                icon: <Sun size={16} />,
                label: "Light",
              },
              {
                value: "dark" as Theme,
                icon: <Moon size={16} />,
                label: "Dark",
              },
              {
                value: "system" as Theme,
                icon: <Monitor size={16} />,
                label: "System",
              },
            ].map((option) => (
              <button
                key={option.value}
                onClick={() => handleThemeChange(option.value)}
                className={cn(
                  "flex items-center gap-2 px-3 py-2 rounded-lg border transition-colors",
                  localConfig?.theme === option.value
                    ? "bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800 text-blue-600 dark:text-blue-400"
                    : "border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700",
                )}
              >
                {option.icon}
                <span className="text-sm">{option.label}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Behavior
        </h3>
        <div className="space-y-4">
          <label className="flex items-center justify-between">
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Auto-save conversations
            </span>
            <input
              type="checkbox"
              checked={localConfig?.autoSave ?? true}
              onChange={(e) =>
                handleConfigChange({ autoSave: e.target.checked })
              }
              className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
          </label>
        </div>
      </div>
    </div>
  );

  const renderVoiceSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Voice Input
        </h3>
        <div className="space-y-4">
          <label className="flex items-center justify-between">
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Enable voice input
            </span>
            <input
              type="checkbox"
              checked={localConfig?.voiceEnabled ?? false}
              onChange={(e) =>
                handleConfigChange({ voiceEnabled: e.target.checked })
              }
              className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
          </label>
        </div>
      </div>
    </div>
  );

  const renderAISettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          AI Configuration
        </h3>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              API Endpoint
            </label>
            <input
              type="text"
              value={localConfig?.apiEndpoint || ""}
              onChange={(e) =>
                handleConfigChange({ apiEndpoint: e.target.value })
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Enter API endpoint URL"
            />
          </div>
        </div>
      </div>
    </div>
  );

  const renderDatabaseSettings = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Database Status
        </h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Connection Status
            </span>
            <div className="flex items-center gap-2">
              {getStatusIcon(settings.isOnline)}
              <span className="text-sm text-gray-900 dark:text-gray-100">
                {settings.isOnline ? "Connected" : "Disconnected"}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderAbout = () => (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          About Chat Box
        </h3>
        <div className="space-y-4">
          <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
              Version
            </h4>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Chat Box v1.0.0
            </p>
          </div>
          <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">
              Built with
            </h4>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              React, TypeScript, Tauri, and Rust
            </p>
          </div>
        </div>
      </div>
    </div>
  );

  const renderTabContent = () => {
    switch (activeTab) {
      case "general":
        return renderGeneralSettings();
      case "voice":
        return renderVoiceSettings();
      case "ai":
        return renderAISettings();
      case "database":
        return renderDatabaseSettings();
      case "about":
        return renderAbout();
      default:
        return renderGeneralSettings();
    }
  };

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black/50 z-50" onClick={onClose} />

      {/* Settings Modal */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
              Settings
            </h2>
            <button
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400"
            >
              <X size={20} />
            </button>
          </div>

          <div className="flex h-[600px]">
            {/* Sidebar */}
            <div className="w-64 border-r border-gray-200 dark:border-gray-700 p-4">
              <nav className="space-y-1">
                {tabs.map((tab) => (
                  <SettingsTab
                    key={tab.id}
                    label={tab.label}
                    icon={tab.icon}
                    isActive={activeTab === tab.id}
                    onClick={() => setActiveTab(tab.id)}
                  />
                ))}
              </nav>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto">
              <div className="p-6">{renderTabContent()}</div>
            </div>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-between p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/50">
            <div className="flex items-center gap-2">
              {hasUnsavedChanges && (
                <span className="text-sm text-amber-600 dark:text-amber-400">
                  You have unsaved changes
                </span>
              )}
            </div>

            <div className="flex items-center gap-3">
              <button
                onClick={handleReset}
                className="flex items-center gap-2 px-4 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 transition-colors"
              >
                <RotateCcw size={16} />
                Reset to Defaults
              </button>

              <button
                onClick={handleSave}
                disabled={!hasUnsavedChanges}
                className={cn(
                  "flex items-center gap-2 px-4 py-2 rounded-lg text-sm transition-colors",
                  hasUnsavedChanges
                    ? "bg-blue-500 text-white hover:bg-blue-600"
                    : "bg-gray-200 dark:bg-gray-600 text-gray-400 dark:text-gray-500 cursor-not-allowed",
                )}
              >
                <Save size={16} />
                Save Changes
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
