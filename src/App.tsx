import { useState, useEffect } from "react";
import {
  Menu,
  Settings as SettingsIcon,
  Plus,
  Wifi,
  WifiOff,
  MessageSquare,
} from "lucide-react";
import { cn } from "./utils/cn";
import { useChat } from "./hooks/useChat";
import { useSettings } from "./hooks/useSettings";
import { ConversationList } from "./components/ConversationList";
import { MessageList } from "./components/Message";
import { ChatInput } from "./components/ChatInput";
import { Settings } from "./components/Settings";
import "./App.css";

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);

  // Chat functionality
  const [chatState, chatActions] = useChat({
    autoLoadConversations: true,
    enableVoice: true,
    enableStreaming: true,
  });

  // Settings functionality
  const [settingsState, settingsActions] = useSettings({
    autoLoad: true,
    enableHealthCheck: true,
    healthCheckInterval: 30000,
  });

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth < 768) {
        setSidebarOpen(false);
      } else {
        setSidebarOpen(true);
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  // Error handling
  useEffect(() => {
    if (chatState.error) {
      settingsActions.showNotification("Chat Error", chatState.error, "error");
    }
  }, [chatState.error, settingsActions]);

  // Handle conversation selection
  const handleSelectConversation = async (conversationId: number) => {
    await chatActions.selectConversation(conversationId);
    // Close sidebar on mobile after selection
    if (window.innerWidth < 768) {
      setSidebarOpen(false);
    }
  };

  // Handle message sending
  const handleSendMessage = async (message: string) => {
    await chatActions.sendMessage(message);
  };

  // Handle voice input
  const handleStartVoice = async () => {
    if (!chatState.currentConversation) {
      // Create new conversation for voice input
      const newConv = await chatActions.createConversation("Voice Chat");
      if (newConv) {
        await chatActions.startVoiceInput(newConv.id);
      }
    } else {
      await chatActions.startVoiceInput(chatState.currentConversation.id);
    }
  };

  const handleStopVoice = () => {
    chatActions.stopVoiceInput();
  };

  // Handle conversation editing
  const handleEditConversation = async (id: number, newTitle: string) => {
    console.log("Edit conversation:", id, newTitle);
  };

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar */}
      <div
        className={cn(
          "transition-all duration-300 ease-in-out bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700",
          sidebarOpen ? "w-80" : "w-0",
          "lg:relative absolute inset-y-0 left-0 z-40 shadow-lg lg:shadow-none",
        )}
      >
        {sidebarOpen && (
          <ConversationList
            conversations={chatState.conversations}
            selectedConversationId={chatState.currentConversation?.id || null}
            onSelectConversation={handleSelectConversation}
            onCreateConversation={chatActions.createConversation}
            onDeleteConversation={chatActions.deleteConversation}
            onEditConversation={handleEditConversation}
            isLoading={chatState.isLoading}
            className="h-full"
          />
        )}
      </div>

      {/* Mobile overlay */}
      {sidebarOpen && (
        <div
          className="lg:hidden fixed inset-0 bg-black/50 z-30"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Main Content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <button
                onClick={() => setSidebarOpen(!sidebarOpen)}
                className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 lg:hidden"
              >
                <Menu size={20} />
              </button>

              <div className="flex items-center gap-3">
                <div className="w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center">
                  <MessageSquare size={16} className="text-white" />
                </div>
                <div>
                  <h1 className="font-semibold text-gray-900 dark:text-gray-100">
                    {chatState.currentConversation?.title || "Chat Box"}
                  </h1>
                  {chatState.currentConversation && (
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {chatState.messages.length} message
                      {chatState.messages.length !== 1 ? "s" : ""}
                    </p>
                  )}
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2">
              {/* Connection Status */}
              <div className="flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400">
                {settingsState.isOnline ? (
                  <Wifi size={16} className="text-green-500" />
                ) : (
                  <WifiOff size={16} className="text-red-500" />
                )}
                <span className="hidden sm:inline">
                  {settingsState.isOnline ? "Online" : "Offline"}
                </span>
              </div>

              {/* New Conversation Button */}
              <button
                onClick={() => chatActions.createConversation()}
                className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
                title="New conversation"
              >
                <Plus size={16} />
              </button>

              {/* Settings Button */}
              <button
                onClick={() => setSettingsOpen(true)}
                className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
                title="Settings"
              >
                <SettingsIcon size={16} />
              </button>
            </div>
          </div>
        </div>

        {/* Chat Area */}
        <div className="flex-1 flex flex-col min-h-0">
          {/* Welcome Screen or Messages */}
          {!chatState.currentConversation ? (
            <div className="flex-1 flex items-center justify-center p-8">
              <div className="text-center max-w-md">
                <div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <MessageSquare size={24} className="text-blue-500" />
                </div>
                <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
                  Welcome to Chat Box
                </h2>
                <p className="text-gray-500 dark:text-gray-400 mb-6">
                  Start a new conversation to begin chatting with AI.
                </p>
                <button
                  onClick={() => chatActions.createConversation()}
                  className="inline-flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
                >
                  <Plus size={16} />
                  New Conversation
                </button>
              </div>
            </div>
          ) : (
            <>
              {/* Messages */}
              <MessageList
                messages={chatState.messages}
                partialMessage={chatState.partialMessage}
                isGenerating={chatState.isGenerating}
                className="flex-1"
              />

              {/* Input */}
              <ChatInput
                onSendMessage={handleSendMessage}
                onStartVoice={handleStartVoice}
                onStopVoice={handleStopVoice}
                voiceStatus={chatState.voiceStatus}
                voiceTranscript={chatState.voiceTranscript}
                isGenerating={chatState.isGenerating}
                disabled={!settingsState.isOnline}
                placeholder="Type your message..."
              />
            </>
          )}
        </div>
      </div>

      {/* Settings Modal */}
      <Settings isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />

      {/* Global Loading Overlay */}
      {chatState.isLoading && chatState.conversations.length === 0 && (
        <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-sm mx-4">
            <div className="flex flex-col items-center gap-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Loading Chat Box...
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Error Toast */}
      {(chatState.error || settingsState.error) && (
        <div className="fixed bottom-4 right-4 z-50">
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 max-w-sm">
            <div className="flex items-start gap-3">
              <div className="text-red-500 mt-0.5">
                <WifiOff size={16} />
              </div>
              <div>
                <p className="text-sm font-medium text-red-700 dark:text-red-300">
                  {chatState.error || settingsState.error}
                </p>
                <button
                  onClick={() => {
                    chatActions.clearError();
                    settingsActions.clearError();
                  }}
                  className="text-xs text-red-600 dark:text-red-400 hover:underline mt-1"
                >
                  Dismiss
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
