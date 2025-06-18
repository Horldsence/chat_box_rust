import React, { useState, useRef, useEffect } from "react";
import { Send, Mic, MicOff, Square, Loader2 } from "lucide-react";
import { cn } from "../utils/cn";
import type { VoiceStatus } from "../types";

interface ChatInputProps {
  onSendMessage: (message: string) => void;
  onStartVoice?: () => void;
  onStopVoice?: () => void;
  voiceStatus?: VoiceStatus;
  voiceTranscript?: string;
  isGenerating?: boolean;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
}

export function ChatInput({
  onSendMessage,
  onStartVoice,
  onStopVoice,
  voiceStatus = "idle",
  voiceTranscript = "",
  isGenerating = false,
  disabled = false,
  placeholder = "Type your message...",
  className,
}: ChatInputProps) {
  const [message, setMessage] = useState("");
  const [isVoiceMode, setIsVoiceMode] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const isVoiceActive =
    voiceStatus === "recording" || voiceStatus === "processing";
  const canSend =
    (message.trim() || voiceTranscript.trim()) && !isGenerating && !disabled;
  const hasVoiceSupport = onStartVoice && onStopVoice;

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
    }
  }, [message]);

  // Update message from voice transcript
  useEffect(() => {
    if (voiceTranscript && isVoiceMode) {
      setMessage(voiceTranscript);
    }
  }, [voiceTranscript, isVoiceMode]);

  // Clear voice mode when voice stops
  useEffect(() => {
    if (voiceStatus === "idle" && isVoiceMode) {
      setIsVoiceMode(false);
    }
  }, [voiceStatus, isVoiceMode]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const messageToSend = message.trim() || voiceTranscript.trim();
    if (!messageToSend || isGenerating || disabled) return;

    onSendMessage(messageToSend);
    setMessage("");
    setIsVoiceMode(false);

    // Focus back to textarea
    setTimeout(() => {
      textareaRef.current?.focus();
    }, 100);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleVoiceToggle = () => {
    if (isVoiceActive) {
      onStopVoice?.();
      setIsVoiceMode(false);
    } else {
      onStartVoice?.();
      setIsVoiceMode(true);
      setMessage("");
    }
  };

  const getVoiceButtonIcon = () => {
    switch (voiceStatus) {
      case "recording":
        return <MicOff size={18} className="text-red-500" />;
      case "processing":
        return <Loader2 size={18} className="animate-spin" />;
      default:
        return <Mic size={18} />;
    }
  };

  const getVoiceButtonTitle = () => {
    switch (voiceStatus) {
      case "recording":
        return "Stop recording";
      case "processing":
        return "Processing...";
      default:
        return "Start voice input";
    }
  };

  const displayMessage = isVoiceMode ? voiceTranscript : message;
  const showVoiceIndicator = isVoiceMode && voiceStatus === "recording";

  return (
    <div
      className={cn(
        "bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700",
        className,
      )}
    >
      <div className="p-4 max-w-4xl mx-auto">
        {/* Voice Status Indicator */}
        {showVoiceIndicator && (
          <div className="flex items-center gap-3 mb-4 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
            <div className="flex gap-1">
              <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:0ms]" />
              <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:150ms]" />
              <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:300ms]" />
            </div>
            <span className="text-sm font-medium text-red-600 dark:text-red-400">
              Recording...
            </span>
            <button
              onClick={handleVoiceToggle}
              className="ml-auto p-1 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400"
            >
              <Square size={14} />
            </button>
          </div>
        )}

        {/* Voice Transcript Display */}
        {isVoiceMode && voiceTranscript && voiceStatus !== "recording" && (
          <div className="mb-4 p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
            <div className="text-xs text-blue-600 dark:text-blue-400 mb-1 font-medium">
              Voice transcript:
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">
              {voiceTranscript}
            </div>
          </div>
        )}

        {/* Input Form */}
        <form onSubmit={handleSubmit}>
          <div className="flex items-end gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-xl border border-gray-200 dark:border-gray-600 focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500 transition-all">
            {/* Text Input */}
            <div className="flex-1">
              <textarea
                ref={textareaRef}
                value={displayMessage}
                onChange={(e) => !isVoiceMode && setMessage(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={
                  isVoiceMode ? "Voice input active..." : placeholder
                }
                disabled={disabled || isVoiceMode}
                className={cn(
                  "w-full resize-none border-none bg-transparent",
                  "text-sm leading-relaxed text-gray-900 dark:text-gray-100",
                  "placeholder:text-gray-500 dark:placeholder:text-gray-400",
                  "focus:outline-none focus:ring-0",
                  "min-h-[24px] max-h-[200px]",
                  disabled && "opacity-50 cursor-not-allowed",
                  isVoiceMode && "cursor-not-allowed opacity-75",
                )}
                rows={1}
              />
            </div>

            {/* Action Buttons */}
            <div className="flex items-center gap-2 flex-shrink-0">
              {/* Voice Button */}
              {hasVoiceSupport && (
                <button
                  type="button"
                  onClick={handleVoiceToggle}
                  disabled={disabled}
                  className={cn(
                    "p-2 rounded-lg transition-colors",
                    isVoiceActive
                      ? "bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50"
                      : "text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600 hover:text-gray-700 dark:hover:text-gray-300",
                    disabled && "opacity-50 cursor-not-allowed",
                  )}
                  title={getVoiceButtonTitle()}
                >
                  {getVoiceButtonIcon()}
                </button>
              )}

              {/* Send Button */}
              <button
                type="submit"
                disabled={!canSend}
                className={cn(
                  "p-2 rounded-lg transition-colors",
                  canSend
                    ? "bg-blue-500 text-white hover:bg-blue-600"
                    : "bg-gray-200 dark:bg-gray-600 text-gray-400 dark:text-gray-500 cursor-not-allowed",
                )}
                title="Send message"
              >
                {isGenerating ? (
                  <Loader2 size={18} className="animate-spin" />
                ) : (
                  <Send size={18} />
                )}
              </button>
            </div>
          </div>

          {/* Status Messages */}
          {(isGenerating || voiceStatus === "error") && (
            <div className="mt-2 text-xs">
              {isGenerating && (
                <div className="flex items-center gap-1 text-blue-600 dark:text-blue-400">
                  <Loader2 size={12} className="animate-spin" />
                  Generating response...
                </div>
              )}

              {voiceStatus === "error" && (
                <div className="text-red-500 dark:text-red-400">
                  Voice input failed. Please try again.
                </div>
              )}
            </div>
          )}

          {/* Help Text */}
          <div className="mt-2 text-xs text-gray-500 dark:text-gray-400 text-center">
            Press Enter to send, Shift+Enter for new line
            {hasVoiceSupport && " • Click mic for voice input"}
          </div>
        </form>
      </div>
    </div>
  );
}
