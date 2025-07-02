import {
  PlusOutlined,
  SettingOutlined,
  SmileOutlined,
  UserOutlined,
} from "@ant-design/icons";
import {
  Attachments,
  Bubble,
  Conversations,
  Prompts,
  Sender,
  Suggestion,
  ThoughtChain,
  Welcome,
} from "@ant-design/x";
import type { GetProp } from "antd";
import { App, Button, ConfigProvider, theme } from "antd";
import React, { useMemo } from "react";
import { useChat } from "../hooks/useChat";
import { useSettings } from "../hooks/useSettings";
import type {
  Conversation as AppConversation,
  Message as AppMessage,
} from "../types";

// Type definitions for Ant Design X
type ConversationsProps = GetProp<typeof Conversations, "items">;

interface AntdXChatProps {
  className?: string;
  onSettingsClick?: () => void;
  onAgentConfigClick?: () => void;
  onLive2DControlClick?: () => void;
}

export const AntdXChat: React.FC<AntdXChatProps> = ({
  className,
  onSettingsClick,
  onAgentConfigClick,
  onLive2DControlClick,
}) => {
  const { token } = theme.useToken();
  const [chatState, chatActions] = useChat({
    autoLoadConversations: true,
    enableVoice: true,
    enableStreaming: true,
  });
  const [settingsState] = useSettings({
    autoLoad: true,
    enableHealthCheck: true,
  });

  // Convert app conversations to Ant Design X format
  const conversationItems: ConversationsProps = useMemo(() => {
    return chatState.conversations.map((conv: AppConversation) => ({
      key: conv.id.toString(),
      label: conv.title,
      timestamp: conv.timestamp,
      description: conv.last_message,
    }));
  }, [chatState.conversations]);

  // Convert app messages to Ant Design X format
  const bubbleItems = useMemo(() => {
    const items = chatState.messages.map((msg: AppMessage) => ({
      key: msg.id.toString(),
      content: msg.content,
      role: msg.sender === "user" ? "user" : "assistant",
      timestamp: msg.timestamp,
    }));

    // Add partial message if generating
    if (chatState.isGenerating && chatState.partialMessage) {
      items.push({
        key: "partial",
        content: chatState.partialMessage,
        role: "assistant",
        timestamp: Date.now(),
      });
    }

    return items;
  }, [chatState.messages, chatState.partialMessage, chatState.isGenerating]);

  // Handle conversation selection
  const handleConversationChange = (activeKey: string) => {
    const conversationId = parseInt(activeKey);
    chatActions.selectConversation(conversationId);
  };

  // Handle new conversation
  const handleNewConversation = async () => {
    await chatActions.createConversation();
  };

  // Handle message send
  const handleSend = async (message: string) => {
    if (message.trim()) {
      await chatActions.sendMessage(message);
    }
  };

  // Welcome prompts
  const welcomePrompts = [
    {
      key: "chat",
      label: "开始对话",
      description: "与AI助手进行自然对话",
      icon: "💬",
    },
    {
      key: "help",
      label: "获取帮助",
      description: "了解如何使用聊天功能",
      icon: "❓",
    },
    {
      key: "settings",
      label: "配置设置",
      description: "自定义应用程序设置",
      icon: "⚙️",
    },
  ];

  // Quick suggestions
  const suggestions = [
    {
      key: "1",
      value: "解释一下这个概念",
      label: "解释一下这个概念",
      description: "请详细解释",
    },
    {
      key: "2",
      value: "写一段代码",
      label: "写一段代码",
      description: "帮我写代码",
    },
    {
      key: "3",
      value: "翻译文本",
      label: "翻译文本",
      description: "翻译成中文",
    },
    { key: "4", value: "总结内容", label: "总结内容", description: "简要总结" },
  ];

  // Handle prompt click
  const handlePromptClick = (key: string) => {
    switch (key) {
      case "chat":
        if (!chatState.currentConversation) {
          handleNewConversation();
        }
        break;
      case "help":
        handleSend("请介绍一下这个聊天应用的功能");
        break;
      case "settings":
        onSettingsClick?.();
        break;
    }
  };

  // Handle suggestion click
  const handleSuggestionClick = (suggestion: any) => {
    handleSend(suggestion.label);
  };

  return (
    <ConfigProvider
      theme={{
        algorithm:
          settingsState.theme === "dark"
            ? theme.darkAlgorithm
            : theme.defaultAlgorithm,
      }}
    >
      <App>
        <div
          className={`flex h-screen bg-gray-50 dark:bg-gray-900 ${className || ""}`}
          style={{ backgroundColor: token.colorBgContainer }}
        >
          {/* Sidebar with Conversations */}
          <div className="w-80 border-r border-gray-200 dark:border-gray-700">
            <div className="h-full flex flex-col">
              {/* Header */}
              <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                <div className="flex items-center justify-between mb-3">
                  <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                    对话列表
                  </h2>
                  <div className="flex gap-2">
                    <Button
                      type="text"
                      icon={<UserOutlined />}
                      size="small"
                      onClick={onAgentConfigClick}
                      title="智能体配置"
                    />
                    <Button
                      type="text"
                      icon={<SmileOutlined />}
                      size="small"
                      onClick={onLive2DControlClick}
                      title="Live2D控制"
                    />
                    <Button
                      type="text"
                      icon={<SettingOutlined />}
                      size="small"
                      onClick={onSettingsClick}
                      title="设置"
                    />
                  </div>
                </div>
                <Button
                  type="primary"
                  icon={<PlusOutlined />}
                  block
                  onClick={handleNewConversation}
                  loading={chatState.isLoading}
                >
                  新建对话
                </Button>
              </div>

              {/* Conversations List */}
              <div className="flex-1 overflow-hidden">
                <Conversations
                  items={conversationItems}
                  activeKey={chatState.currentConversation?.id.toString()}
                  onActiveChange={handleConversationChange}
                  style={{ height: "100%" }}
                />
              </div>
            </div>
          </div>

          {/* Main Chat Area */}
          <div className="flex-1 flex flex-col min-w-0">
            {!chatState.currentConversation ? (
              /* Welcome Screen */
              <div className="flex-1 flex items-center justify-center">
                <div className="max-w-md text-center">
                  <Welcome
                    title="欢迎使用 Chat Box"
                    description="选择一个对话开始聊天，或创建新的对话"
                    extra={
                      <Prompts
                        items={welcomePrompts}
                        onItemClick={(info) => handlePromptClick(info.data.key)}
                        style={{ marginTop: 24 }}
                      />
                    }
                  />
                </div>
              </div>
            ) : (
              <>
                {/* Chat Header */}
                <div className="border-b border-gray-200 dark:border-gray-700 px-6 py-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
                        {chatState.currentConversation.title}
                      </h1>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {chatState.messages.length} 条消息
                        {settingsState.isOnline ? " • 在线" : " • 离线"}
                      </p>
                    </div>
                  </div>
                </div>

                {/* Messages Area */}
                <div className="flex-1 overflow-hidden">
                  {bubbleItems.length === 0 ? (
                    <div className="h-full flex items-center justify-center">
                      <div className="text-center max-w-md">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                          开始新对话
                        </h3>
                        <p className="text-gray-500 dark:text-gray-400 mb-6">
                          发送消息开始与AI助手对话
                        </p>
                        <Suggestion
                          items={suggestions}
                          onSelect={handleSuggestionClick}
                        />
                      </div>
                    </div>
                  ) : (
                    <div className="h-full overflow-y-auto p-6">
                      <div className="max-w-4xl mx-auto space-y-4">
                        {bubbleItems.map((item: any) => (
                          <div
                            key={item.key}
                            className={`flex ${item.role === "user" ? "justify-end" : "justify-start"}`}
                          >
                            <div
                              className={`max-w-[80%] ${item.role === "user" ? "ml-12" : "mr-12"}`}
                            >
                              <Bubble
                                content={item.content}
                                avatar={
                                  item.role === "user"
                                    ? { icon: <UserOutlined /> }
                                    : undefined
                                }
                                placement={
                                  item.role === "user" ? "end" : "start"
                                }
                              />
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                {/* Input Area */}
                <div className="border-t border-gray-200 dark:border-gray-700 p-6">
                  <div className="max-w-4xl mx-auto">
                    <Sender
                      placeholder="输入消息..."
                      onSubmit={handleSend}
                      loading={chatState.isGenerating}
                      disabled={!settingsState.isOnline}
                      actions={
                        <div className="flex items-center gap-2">
                          <Attachments
                            beforeUpload={() => false}
                            accept=".txt,.md,.pdf"
                            maxCount={5}
                          />
                        </div>
                      }
                    />
                    {chatState.isGenerating && (
                      <div className="mt-2 text-center">
                        <ThoughtChain
                          items={[
                            { key: "1", title: "分析问题", status: "success" },
                            { key: "2", title: "生成回答", status: "pending" },
                            { key: "3", title: "优化回答", status: "pending" },
                          ]}
                        />
                      </div>
                    )}
                  </div>
                </div>
              </>
            )}
          </div>
        </div>
      </App>
    </ConfigProvider>
  );
};

export default AntdXChat;
