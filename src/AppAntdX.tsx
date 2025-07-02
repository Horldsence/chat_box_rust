import { useState, useEffect } from "react";
import { ConfigProvider, theme, App as AntdApp } from "antd";
import { XProvider } from "@ant-design/x";
import { AntdXChat } from "./components/AntdXChat";
import { Settings } from "./components/Settings";
import AgentConfig from "./components/AgentConfig";
import Live2DControl from "./components/Live2DControl";
import Live2DTest from "./components/Live2DTest";
import Live2DRender from "./components/Live2DRender";
import { useSettings } from "./hooks/useSettings";

import "./App.css";

function AppAntdX() {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [agentConfigOpen, setAgentConfigOpen] = useState(false);
  const [live2dControlOpen, setLive2dControlOpen] = useState(false);
  const [showLive2D, setShowLive2D] = useState(true);
  const [showTestPage, setShowTestPage] = useState(false);

  // Settings functionality
  const [settingsState, settingsActions] = useSettings({
    autoLoad: true,
    enableHealthCheck: true,
    healthCheckInterval: 30000,
  });

  // Handle window resize for responsive design
  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth < 1024) {
        setShowLive2D(false);
      } else {
        setShowLive2D(true);
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  // Theme configuration
  const themeConfig = {
    algorithm:
      settingsState.theme === "dark"
        ? theme.darkAlgorithm
        : theme.defaultAlgorithm,
    token: {
      colorPrimary: "#1677ff",
      borderRadius: 8,
    },
  };

  if (showTestPage) {
    return (
      <ConfigProvider theme={themeConfig}>
        <AntdApp>
          <div className="h-screen bg-gray-100 dark:bg-gray-900">
            <div className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 border-b">
              <h1 className="text-xl font-semibold">Test Page</h1>
              <button
                onClick={() => setShowTestPage(false)}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                Back to Chat
              </button>
            </div>
            <Live2DTest />
          </div>
        </AntdApp>
      </ConfigProvider>
    );
  }

  return (
    <ConfigProvider theme={themeConfig}>
      <XProvider>
        <AntdApp>
          <div className="h-screen flex bg-gray-50 dark:bg-gray-900">
            {/* Main Chat Interface */}
            <div className="flex-1 flex">
              <AntdXChat
                onSettingsClick={() => setSettingsOpen(true)}
                onAgentConfigClick={() => setAgentConfigOpen(true)}
                onLive2DControlClick={() => setLive2dControlOpen(true)}
              />

              {/* Live2D Panel */}
              {showLive2D && (
                <div className="w-80 border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
                  {/* Live2D Header */}
                  <div className="border-b border-gray-200 dark:border-gray-700 px-4 py-3">
                    <div className="flex items-center justify-between">
                      <h3 className="font-medium text-gray-900 dark:text-gray-100">
                        Live2D Character
                      </h3>
                      <div className="flex gap-2">
                        <button
                          onClick={() => setShowTestPage(true)}
                          className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400 text-sm"
                          title="Test Page"
                        >
                          Test
                        </button>
                        <button
                          onClick={() => setShowLive2D(false)}
                          className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400"
                          title="Hide Live2D"
                        >
                          ×
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Live2D Content */}
                  <div className="flex-1 relative">
                    <Live2DRender className="w-full h-full" />
                  </div>

                  {/* Live2D Status */}
                  <div className="border-t border-gray-200 dark:border-gray-700 px-4 py-2">
                    <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
                      <span>Status: Active</span>
                      <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Settings Modal */}
            {settingsOpen && (
              <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl max-h-[90vh] overflow-hidden">
                  <Settings
                    isOpen={settingsOpen}
                    onClose={() => setSettingsOpen(false)}
                  />
                </div>
              </div>
            )}

            {/* Agent Config Modal */}
            {agentConfigOpen && (
              <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl max-h-[90vh] overflow-hidden">
                  <AgentConfig />
                </div>
              </div>
            )}

            {/* Live2D Control Modal */}
            {live2dControlOpen && (
              <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg max-h-[90vh] overflow-hidden">
                  <Live2DControl />
                </div>
              </div>
            )}

            {/* Show/Hide Live2D Toggle (when hidden) */}
            {!showLive2D && (
              <button
                onClick={() => setShowLive2D(true)}
                className="fixed bottom-6 right-6 p-3 bg-blue-500 text-white rounded-full shadow-lg hover:bg-blue-600 z-40"
                title="Show Live2D"
              >
                <svg
                  className="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 19l-7-7 7-7"
                  />
                </svg>
              </button>
            )}

            {/* Connection Status Indicator */}
            <div className="fixed top-4 right-4 z-30">
              <div
                className={`flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium ${
                  settingsState.isOnline
                    ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                    : "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                }`}
              >
                <div
                  className={`w-2 h-2 rounded-full ${
                    settingsState.isOnline ? "bg-green-500" : "bg-red-500"
                  }`}
                />
                {settingsState.isOnline ? "Online" : "Offline"}
              </div>
            </div>

            {/* Global Error Display */}
            {settingsState.error && (
              <div className="fixed bottom-4 left-4 right-4 z-50">
                <div className="bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <svg
                      className="w-5 h-5 text-red-500"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                      />
                    </svg>
                    <span className="text-red-800 dark:text-red-400 text-sm font-medium">
                      {settingsState.error}
                    </span>
                  </div>
                  <button
                    onClick={() => settingsActions.clearError()}
                    className="text-red-500 hover:text-red-700 dark:hover:text-red-300"
                  >
                    <svg
                      className="w-4 h-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M6 18L18 6M6 6l12 12"
                      />
                    </svg>
                  </button>
                </div>
              </div>
            )}

            {/* Loading Overlay */}
            {settingsState.isLoading && (
              <div className="fixed inset-0 bg-black/20 z-40 flex items-center justify-center">
                <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow-xl">
                  <div className="flex items-center gap-3">
                    <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                    <span className="text-gray-900 dark:text-gray-100 font-medium">
                      Loading...
                    </span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </AntdApp>
      </XProvider>
    </ConfigProvider>
  );
}

export default AppAntdX;
