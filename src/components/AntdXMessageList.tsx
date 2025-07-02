import React, { useRef, useEffect, useState } from "react";
import { Bubble, ThoughtChain } from "@ant-design/x";
import { Button, Avatar, Tooltip, Dropdown, message as antMessage } from "antd";
import {
  UserOutlined,
  RobotOutlined,
  CopyOutlined,
  ReloadOutlined,
  MoreOutlined,
  CheckOutlined,
  ClockCircleOutlined,
  SoundOutlined,
} from "@ant-design/icons";
import type { MenuProps } from "antd";
import type { Message } from "../types";
import { utils } from "../utils/api";

interface AntdXMessageListProps {
  messages: Message[];
  partialMessage?: string;
  isGenerating?: boolean;
  onRetryMessage?: (messageId: number) => void;
  onCopyMessage?: (content: string) => void;
  onSpeakMessage?: (content: string) => void;
  enableTTS?: boolean;
  className?: string;
}

interface MessageItemProps {
  message: Message;
  isPartial?: boolean;
  onRetry?: () => void;
  onCopy?: () => void;
  onSpeak?: () => void;
  enableTTS?: boolean;
}

const MessageItem: React.FC<MessageItemProps> = ({
  message,
  isPartial = false,
  onRetry,
  onCopy,
  onSpeak,
  enableTTS = false,
}) => {
  const [copied, setCopied] = useState(false);
  const [speaking, setSpeaking] = useState(false);
  const isUser = message.sender === "user";

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(message.content);
      setCopied(true);
      onCopy?.();
      antMessage.success("已复制到剪贴板");
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      antMessage.error("复制失败");
    }
  };

  const handleSpeak = () => {
    if (speaking) {
      speechSynthesis.cancel();
      setSpeaking(false);
    } else {
      const utterance = new SpeechSynthesisUtterance(message.content);
      utterance.onstart = () => setSpeaking(true);
      utterance.onend = () => setSpeaking(false);
      utterance.onerror = () => setSpeaking(false);
      speechSynthesis.speak(utterance);
      onSpeak?.();
    }
  };

  const menuItems: MenuProps["items"] = [
    {
      key: "copy",
      label: copied ? "已复制" : "复制内容",
      icon: copied ? <CheckOutlined /> : <CopyOutlined />,
      onClick: handleCopy,
    },
    ...(enableTTS
      ? [
          {
            key: "speak",
            label: speaking ? "停止朗读" : "朗读内容",
            icon: <SoundOutlined />,
            onClick: handleSpeak,
          },
        ]
      : []),
    ...(!isUser && onRetry
      ? [
          {
            key: "retry",
            label: "重新生成",
            icon: <ReloadOutlined />,
            onClick: onRetry,
          },
        ]
      : []),
  ];

  const renderContent = () => {
    if (!message.content && isPartial) {
      return (
        <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400">
          <div className="flex gap-1">
            <div className="w-2 h-2 bg-current rounded-full animate-bounce [animation-delay:-0.3s]" />
            <div className="w-2 h-2 bg-current rounded-full animate-bounce [animation-delay:-0.15s]" />
            <div className="w-2 h-2 bg-current rounded-full animate-bounce" />
          </div>
          <span className="text-sm">正在思考...</span>
        </div>
      );
    }

    return (
      <div className="prose prose-sm max-w-none dark:prose-invert">
        <div className="whitespace-pre-wrap break-words leading-relaxed">
          {message.content}
          {isPartial && (
            <span className="inline-block w-2 h-4 bg-blue-500 ml-1 animate-pulse" />
          )}
        </div>
      </div>
    );
  };

  return (
    <div
      className={`flex gap-4 p-4 group ${isUser ? "flex-row-reverse" : "flex-row"}`}
    >
      {/* Avatar */}
      <div className="flex-shrink-0">
        <Avatar
          size={40}
          icon={isUser ? <UserOutlined /> : <RobotOutlined />}
          style={{
            backgroundColor: isUser ? "#1677ff" : "#f0f0f0",
            color: isUser ? "#fff" : "#666",
          }}
        />
      </div>

      {/* Message Bubble */}
      <div
        className={`flex-1 min-w-0 max-w-[80%] ${isUser ? "flex justify-end" : "flex justify-start"}`}
      >
        <div className="relative">
          <Bubble
            content={renderContent()}
            className={`${isUser ? "bubble-user" : "bubble-assistant"}`}
            style={{
              backgroundColor: isUser ? "#1677ff" : "#f8f9fa",
              color: isUser ? "#fff" : "#333",
              borderRadius: "12px",
              padding: "12px 16px",
              maxWidth: "100%",
              wordBreak: "break-word",
            }}
          />

          {/* Message Actions */}
          <div
            className={`absolute top-2 opacity-0 group-hover:opacity-100 transition-opacity ${
              isUser ? "left-2" : "right-2"
            }`}
          >
            <Dropdown
              menu={{ items: menuItems }}
              trigger={["click"]}
              placement={isUser ? "bottomLeft" : "bottomRight"}
            >
              <Button
                type="text"
                size="small"
                icon={<MoreOutlined />}
                className="bg-white/90 dark:bg-gray-800/90 shadow-sm"
              />
            </Dropdown>
          </div>

          {/* Timestamp */}
          <div
            className={`mt-1 text-xs text-gray-400 dark:text-gray-500 flex items-center gap-1 ${
              isUser ? "justify-end" : "justify-start"
            }`}
          >
            <ClockCircleOutlined className="text-xs" />
            <Tooltip title={utils.formatTimestamp(message.timestamp)}>
              <span>{utils.formatRelativeTime(message.timestamp)}</span>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>
  );
};

export const AntdXMessageList: React.FC<AntdXMessageListProps> = ({
  messages,
  partialMessage,
  isGenerating = false,
  onRetryMessage,
  onCopyMessage,
  onSpeakMessage,
  enableTTS = false,
  className,
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      const scrollElement = scrollRef.current;
      scrollElement.scrollTop = scrollElement.scrollHeight;
    }
  }, [messages, partialMessage, autoScroll]);

  // Handle scroll to detect if user manually scrolled up
  const handleScroll = () => {
    if (scrollRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
      const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
      setAutoScroll(isAtBottom);
    }
  };

  const shouldShowPartial = isGenerating && partialMessage;
  const partialMessageObj: Message = {
    id: -1,
    content: partialMessage || "",
    sender: "bot",
    timestamp: Date.now(),
    conversation_id: messages[0]?.conversation_id || 0,
  };

  const handleRetryMessage = (messageId: number) => {
    onRetryMessage?.(messageId);
  };

  const handleCopyMessage = (content: string) => {
    onCopyMessage?.(content);
  };

  const handleSpeakMessage = (content: string) => {
    onSpeakMessage?.(content);
  };

  return (
    <div className={`flex-1 flex flex-col min-h-0 ${className || ""}`}>
      {/* Messages Container */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900 scrollbar-thin scrollbar-track-transparent scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600"
        onScroll={handleScroll}
      >
        <div className="max-w-4xl mx-auto py-4">
          {messages.length === 0 && !shouldShowPartial && (
            <div className="flex flex-col items-center justify-center h-full text-center p-8 min-h-[400px]">
              <div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/20 rounded-full flex items-center justify-center mb-4">
                <RobotOutlined className="text-2xl text-blue-500" />
              </div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                准备开始对话
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 max-w-md">
                发送消息开始与 AI 助手的对话。您可以询问任何问题或寻求帮助。
              </p>
            </div>
          )}

          {messages.map((message) => (
            <MessageItem
              key={message.id}
              message={message}
              onRetry={() => handleRetryMessage(message.id)}
              onCopy={() => handleCopyMessage(message.content)}
              onSpeak={() => handleSpeakMessage(message.content)}
              enableTTS={enableTTS}
            />
          ))}

          {shouldShowPartial && (
            <MessageItem
              key="partial"
              message={partialMessageObj}
              isPartial={true}
              enableTTS={enableTTS}
            />
          )}

          {isGenerating && !partialMessage && (
            <MessageItem
              key="generating"
              message={partialMessageObj}
              isPartial={true}
              enableTTS={enableTTS}
            />
          )}
        </div>
      </div>

      {/* Thought Chain for AI Processing */}
      {isGenerating && (
        <div className="border-t border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-800">
          <div className="max-w-4xl mx-auto">
            <ThoughtChain
              items={[
                { key: "1", title: "理解问题", status: "success" },
                { key: "2", title: "检索知识", status: "success" },
                {
                  key: "2",
                  title: "生成回答",
                  status: partialMessage ? "success" : "pending",
                },
                { key: "4", title: "优化表达", status: "pending" },
              ]}
            />
          </div>
        </div>
      )}

      {/* Scroll to Bottom Button */}
      {!autoScroll && (
        <div className="absolute bottom-20 right-6">
          <Button
            type="primary"
            shape="circle"
            size="large"
            onClick={() => {
              setAutoScroll(true);
              if (scrollRef.current) {
                scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
              }
            }}
            className="shadow-lg"
          >
            ↓
          </Button>
        </div>
      )}

      <style>{`
        .bubble-user {
          background: linear-gradient(135deg, #1677ff 0%, #69c0ff 100%);
          color: white;
        }

        .bubble-assistant {
          background: linear-gradient(135deg, #f8f9fa 0%, #ffffff 100%);
          border: 1px solid #e8e8e8;
        }

        .dark .bubble-assistant {
          background: linear-gradient(135deg, #1f1f1f 0%, #2d2d2d 100%);
          border-color: #404040;
          color: #fff;
        }

        .scrollbar-thin::-webkit-scrollbar {
          width: 6px;
        }

        .scrollbar-thin::-webkit-scrollbar-track {
          background: transparent;
        }

        .scrollbar-thin::-webkit-scrollbar-thumb {
          background: #d1d5db;
          border-radius: 3px;
        }

        .dark .scrollbar-thin::-webkit-scrollbar-thumb {
          background: #4b5563;
        }
      `}</style>
    </div>
  );
};

export default AntdXMessageList;
