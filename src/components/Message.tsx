import React, { useState } from "react";
import { Copy, User, Bot, Clock, Check } from "lucide-react";
import { cn } from "../utils/cn";
import { utils } from "../utils/api";
import type { Message as MessageType } from "../types";

interface MessageProps {
  message: MessageType;
  isPartial?: boolean;
  className?: string;
}

export function Message({
  message,
  isPartial = false,
  className,
}: MessageProps) {
  const [copied, setCopied] = useState(false);
  const isUser = message.sender === "user";

  const handleCopy = async () => {
    const success = await utils.copyToClipboard(message.content);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return utils.formatRelativeTime(timestamp);
  };

  const renderContent = () => {
    if (!message.content && isPartial) {
      return (
        <div className="flex items-center gap-1 text-gray-500 dark:text-gray-400">
          <div className="w-2 h-2 bg-current rounded-full animate-bounce [animation-delay:-0.3s]" />
          <div className="w-2 h-2 bg-current rounded-full animate-bounce [animation-delay:-0.15s]" />
          <div className="w-2 h-2 bg-current rounded-full animate-bounce" />
        </div>
      );
    }

    return (
      <div className="prose prose-sm max-w-none dark:prose-invert">
        <p className="whitespace-pre-wrap break-words m-0 leading-relaxed text-gray-900 dark:text-gray-100">
          {message.content}
          {isPartial && (
            <span className="inline-block w-2 h-4 bg-blue-500 ml-1 animate-pulse" />
          )}
        </p>
      </div>
    );
  };

  return (
    <div
      className={cn(
        "group flex gap-4 p-4 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors",
        isUser && "bg-blue-50 dark:bg-blue-900/10",
        className,
      )}
    >
      {/* Avatar */}
      <div
        className={cn(
          "flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center",
          isUser
            ? "bg-blue-500 text-white"
            : "bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
        )}
      >
        {isUser ? <User size={16} /> : <Bot size={16} />}
      </div>

      {/* Message Content */}
      <div className="flex-1 min-w-0">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <span className="font-medium text-sm text-gray-900 dark:text-gray-100">
              {isUser ? "You" : "Assistant"}
            </span>
            <span className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
              <Clock size={12} />
              {formatTimestamp(message.timestamp)}
            </span>
          </div>

          {/* Actions */}
          <button
            onClick={handleCopy}
            className={cn(
              "opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700",
              "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300",
            )}
            title="Copy message"
          >
            {copied ? (
              <Check size={14} className="text-green-500" />
            ) : (
              <Copy size={14} />
            )}
          </button>
        </div>

        {/* Content */}
        <div className="text-sm">{renderContent()}</div>

        {/* Status indicators */}
        {isPartial && (
          <div className="flex items-center gap-2 mt-2 text-xs text-gray-500 dark:text-gray-400">
            <div className="flex items-center gap-1">
              <div className="w-1 h-1 bg-blue-500 rounded-full animate-pulse" />
              Generating response...
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface MessageListProps {
  messages: MessageType[];
  partialMessage?: string;
  isGenerating?: boolean;
  className?: string;
}

export function MessageList({
  messages,
  partialMessage,
  isGenerating = false,
  className,
}: MessageListProps) {
  const scrollRef = React.useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  React.useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, partialMessage]);

  const shouldShowPartial = isGenerating && partialMessage;
  const partialMessageObj: MessageType = {
    id: -1,
    content: partialMessage || "",
    sender: "bot",
    timestamp: Date.now(),
    conversation_id: messages[0]?.conversation_id || 0,
  };

  return (
    <div
      ref={scrollRef}
      className={cn(
        "flex-1 overflow-y-auto bg-white dark:bg-gray-900",
        "scrollbar-thin scrollbar-track-transparent scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600",
        className,
      )}
    >
      <div className="max-w-4xl mx-auto">
        {messages.length === 0 && !shouldShowPartial && (
          <div className="flex flex-col items-center justify-center h-full text-center p-8">
            <div className="w-16 h-16 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center mb-4">
              <Bot size={24} className="text-gray-400 dark:text-gray-500" />
            </div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
              Ready to chat
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 max-w-md">
              Send a message to begin your conversation with the AI assistant.
            </p>
          </div>
        )}

        {messages.map((message) => (
          <Message key={message.id} message={message} />
        ))}

        {shouldShowPartial && (
          <Message message={partialMessageObj} isPartial={true} />
        )}

        {isGenerating && !partialMessage && (
          <Message message={partialMessageObj} isPartial={true} />
        )}
      </div>
    </div>
  );
}
