import React, { useState, useRef, useEffect } from "react";
import { Sender, Attachments } from "@ant-design/x";
import { Button, Tooltip } from "antd";
import {
  AudioOutlined,
  StopOutlined,
  PaperClipOutlined,
  LoadingOutlined,
} from "@ant-design/icons";
import type { UploadFile } from "antd";
import type { VoiceStatus } from "../types";

interface AntdXChatInputProps {
  onSendMessage: (message: string, attachments?: UploadFile[]) => void;
  onStartVoice?: () => void;
  onStopVoice?: () => void;
  voiceStatus?: VoiceStatus;
  voiceTranscript?: string;
  isGenerating?: boolean;
  disabled?: boolean;
  placeholder?: string;
  maxLength?: number;
  allowAttachments?: boolean;
  supportedFileTypes?: string[];
  maxAttachments?: number;
  className?: string;
}

export const AntdXChatInput: React.FC<AntdXChatInputProps> = ({
  onSendMessage,
  onStartVoice,
  onStopVoice,
  voiceStatus = "idle",
  voiceTranscript = "",
  isGenerating = false,
  disabled = false,
  placeholder = "输入消息...",
  allowAttachments = true,
  supportedFileTypes = [
    ".txt",
    ".md",
    ".pdf",
    ".doc",
    ".docx",
    ".jpg",
    ".png",
    ".gif",
  ],
  maxAttachments = 5,
  className,
}) => {
  const [message, setMessage] = useState("");
  const [attachments, setAttachments] = useState<UploadFile[]>([]);
  const [isVoiceMode, setIsVoiceMode] = useState(false);
  const inputRef = useRef<any>(null);

  const hasVoiceSupport = onStartVoice && onStopVoice;
  const isVoiceActive =
    voiceStatus === "recording" || voiceStatus === "processing";
  // const canSend = (message.trim() || voiceTranscript.trim() || attachments.length > 0) && !disabled;

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

  // Handle message send
  const handleSend = () => {
    const messageToSend = message.trim() || voiceTranscript.trim();
    if (!messageToSend && attachments.length === 0) return;

    onSendMessage(messageToSend, attachments);
    setMessage("");
    setAttachments([]);
    setIsVoiceMode(false);

    // Focus back to input
    setTimeout(() => {
      inputRef.current?.focus();
    }, 100);
  };

  // Handle voice toggle
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

  // Handle file upload
  const handleFileChange = (fileList: UploadFile[]) => {
    // Validate file types
    const validFiles = fileList.filter((file) => {
      const fileName = file.name || "";
      const isValidType = supportedFileTypes.some((type) =>
        fileName.toLowerCase().endsWith(type.toLowerCase()),
      );
      if (!isValidType) {
        console.error(`不支持的文件类型: ${fileName}`);
        return false;
      }
      return true;
    });

    // Check file count limit
    if (validFiles.length > maxAttachments) {
      console.error(`最多只能上传 ${maxAttachments} 个文件`);
      setAttachments(validFiles.slice(0, maxAttachments));
    } else {
      setAttachments(validFiles);
    }
  };

  // Get voice button icon
  const getVoiceIcon = () => {
    switch (voiceStatus) {
      case "recording":
        return <StopOutlined className="text-red-500" />;
      case "processing":
        return <LoadingOutlined className="animate-spin" />;
      default:
        return <AudioOutlined />;
    }
  };

  // Get voice button tooltip
  const getVoiceTooltip = () => {
    switch (voiceStatus) {
      case "recording":
        return "停止录音";
      case "processing":
        return "处理中...";
      default:
        return "语音输入";
    }
  };

  // Voice status indicator
  const renderVoiceIndicator = () => {
    if (!isVoiceMode || voiceStatus !== "recording") return null;

    return (
      <div className="flex items-center gap-3 mb-3 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
        <div className="flex gap-1">
          <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:0ms]" />
          <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:150ms]" />
          <div className="w-2 h-4 bg-red-500 rounded-full animate-pulse [animation-delay:300ms]" />
        </div>
        <span className="text-sm font-medium text-red-600 dark:text-red-400">
          正在录音...
        </span>
        <Button
          type="text"
          size="small"
          icon={<StopOutlined />}
          onClick={handleVoiceToggle}
          className="ml-auto text-red-600 dark:text-red-400"
        />
      </div>
    );
  };

  // Voice transcript display
  const renderVoiceTranscript = () => {
    if (!isVoiceMode || !voiceTranscript || voiceStatus === "recording")
      return null;

    return (
      <div className="mb-3 p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
        <div className="text-xs text-blue-600 dark:text-blue-400 mb-1 font-medium">
          语音识别结果:
        </div>
        <div className="text-sm text-gray-900 dark:text-gray-100">
          {voiceTranscript}
        </div>
      </div>
    );
  };

  // Attachment preview
  const renderAttachments = () => {
    if (attachments.length === 0) return null;

    return (
      <div className="mb-3">
        <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">
          附件 ({attachments.length}/{maxAttachments}):
        </div>
        <div className="flex flex-wrap gap-2">
          {attachments.map((file, index) => (
            <div
              key={index}
              className="flex items-center gap-2 px-3 py-1 bg-gray-100 dark:bg-gray-700 rounded-md text-sm"
            >
              <PaperClipOutlined className="text-gray-400" />
              <span className="text-gray-700 dark:text-gray-300 truncate max-w-32">
                {file.name}
              </span>
              <Button
                type="text"
                size="small"
                onClick={() =>
                  setAttachments((prev) => prev.filter((_, i) => i !== index))
                }
                className="p-0 h-auto text-gray-400 hover:text-red-500"
              >
                ×
              </Button>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const displayMessage = isVoiceMode ? voiceTranscript : message;

  return (
    <div
      className={`bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 ${className || ""}`}
    >
      <div className="p-4 max-w-4xl mx-auto">
        {/* Voice Status Indicator */}
        {renderVoiceIndicator()}

        {/* Voice Transcript Display */}
        {renderVoiceTranscript()}

        {/* Attachment Preview */}
        {renderAttachments()}

        {/* Input Area */}
        <Sender
          ref={inputRef}
          value={displayMessage}
          onChange={isVoiceMode ? undefined : setMessage}
          placeholder={isVoiceMode ? "语音输入激活中..." : placeholder}
          onSubmit={handleSend}
          loading={isGenerating}
          disabled={disabled || isVoiceMode}
          autoSize={{ minRows: 1, maxRows: 6 }}
          actions={
            <div className="flex items-center gap-2">
              {/* Attachments */}
              {allowAttachments && (
                <Attachments
                  beforeUpload={() => false}
                  onChange={({ fileList }) => handleFileChange(fileList)}
                  accept={supportedFileTypes.join(",")}
                  maxCount={maxAttachments}
                  multiple
                >
                  <Tooltip title="添加附件">
                    <Button
                      type="text"
                      icon={<PaperClipOutlined />}
                      disabled={disabled}
                      className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                    />
                  </Tooltip>
                </Attachments>
              )}

              {/* Voice Button */}
              {hasVoiceSupport && (
                <Tooltip title={getVoiceTooltip()}>
                  <Button
                    type="text"
                    icon={getVoiceIcon()}
                    onClick={handleVoiceToggle}
                    disabled={disabled}
                    className={`${
                      isVoiceActive
                        ? "text-red-500 hover:text-red-600"
                        : "text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                    }`}
                  />
                </Tooltip>
              )}
            </div>
          }
          styles={{
            input: {
              fontSize: "14px",
              lineHeight: "1.5",
            },
          }}
        />

        {/* Status Messages */}
        {(isGenerating || voiceStatus === "error") && (
          <div className="mt-2 text-xs flex items-center justify-center gap-2">
            {isGenerating && (
              <div className="flex items-center gap-1 text-blue-600 dark:text-blue-400">
                <LoadingOutlined className="text-xs" />
                正在生成回复...
              </div>
            )}

            {voiceStatus === "error" && (
              <div className="text-red-500 dark:text-red-400">
                语音输入失败，请重试
              </div>
            )}
          </div>
        )}

        {/* Help Text */}
        <div className="mt-2 text-xs text-gray-500 dark:text-gray-400 text-center">
          按 Enter 发送，Shift+Enter 换行
          {hasVoiceSupport && " • 支持语音输入"}
          {allowAttachments &&
            ` • 支持文件上传 (${supportedFileTypes.join(", ")})`}
        </div>
      </div>
    </div>
  );
};

export default AntdXChatInput;
