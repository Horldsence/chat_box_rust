import { useState, useEffect, useCallback, useRef } from "react";
import { conversationApi, messageApi, voiceApi, utils } from "../utils/api";
import type {
  Conversation,
  Message,
  MessageChunk,
  VoiceStatus,
} from "../types";

export interface UseChatOptions {
  autoLoadConversations?: boolean;
  enableVoice?: boolean;
  enableStreaming?: boolean;
}

export interface ChatState {
  conversations: Conversation[];
  messages: Message[];
  currentConversation: Conversation | null;
  isLoading: boolean;
  isGenerating: boolean;
  error: string | null;
  voiceStatus: VoiceStatus;
  voiceTranscript: string;
  partialMessage: string;
}

export interface ChatActions {
  // Conversation management
  loadConversations: () => Promise<void>;
  selectConversation: (conversationId: number) => Promise<void>;
  createConversation: (title?: string) => Promise<Conversation | null>;
  deleteConversation: (conversationId: number) => Promise<void>;

  // Message management
  sendMessage: (content: string, conversationId?: number) => Promise<void>;
  clearMessages: () => void;

  // Voice functionality
  startVoiceInput: (conversationId?: number) => Promise<void>;
  stopVoiceInput: () => void;

  // UI actions
  clearError: () => void;
  refreshData: () => Promise<void>;
}

export function useChat(
  options: UseChatOptions = {},
): [ChatState, ChatActions] {
  const {
    autoLoadConversations = true,
    enableVoice = true,
    enableStreaming = true,
  } = options;

  // State
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [currentConversation, setCurrentConversation] =
    useState<Conversation | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [voiceStatus, setVoiceStatus] = useState<VoiceStatus>("idle");
  const [voiceTranscript, setVoiceTranscript] = useState("");
  const [partialMessage, setPartialMessage] = useState("");

  // Refs for cleanup
  const messageChunkUnlisten = useRef<(() => void) | null>(null);
  const voiceStatusUnlisten = useRef<(() => void) | null>(null);
  const voicePartialUnlisten = useRef<(() => void) | null>(null);

  // Load conversations
  const loadConversations = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const convs = await conversationApi.getAll();
      setConversations(convs.sort((a, b) => b.timestamp - a.timestamp));
    } catch (err) {
      const errorMsg = "Failed to load conversations";
      setError(errorMsg);
      await utils.handleApiError(err, errorMsg);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Select conversation and load its messages
  const selectConversation = useCallback(
    async (conversationId: number) => {
      setIsLoading(true);
      setError(null);
      try {
        const conversation = conversations.find((c) => c.id === conversationId);
        if (!conversation) {
          throw new Error("Conversation not found");
        }

        const msgs = await conversationApi.getMessages(conversationId);
        setMessages(msgs.sort((a, b) => a.timestamp - b.timestamp));
        setCurrentConversation(conversation);
      } catch (err) {
        const errorMsg = "Failed to load conversation messages";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
      } finally {
        setIsLoading(false);
      }
    },
    [conversations],
  );

  // Create new conversation
  const createConversation = useCallback(
    async (title?: string): Promise<Conversation | null> => {
      setIsLoading(true);
      setError(null);
      try {
        const defaultTitle =
          title || `New Chat ${new Date().toLocaleDateString()}`;
        const newConversation = await conversationApi.create(defaultTitle);

        setConversations((prev) => [newConversation, ...prev]);
        setCurrentConversation(newConversation);
        setMessages([]);

        return newConversation;
      } catch (err) {
        const errorMsg = "Failed to create conversation";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [],
  );

  // Delete conversation
  const deleteConversation = useCallback(
    async (conversationId: number) => {
      setIsLoading(true);
      setError(null);
      try {
        await conversationApi.delete(conversationId);

        setConversations((prev) => prev.filter((c) => c.id !== conversationId));

        if (currentConversation?.id === conversationId) {
          setCurrentConversation(null);
          setMessages([]);
        }
      } catch (err) {
        const errorMsg = "Failed to delete conversation";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
      } finally {
        setIsLoading(false);
      }
    },
    [currentConversation],
  );

  // Send message
  const sendMessage = useCallback(
    async (content: string, conversationId?: number) => {
      if (!content.trim()) return;

      let targetConversationId = conversationId || currentConversation?.id;

      // Create new conversation if none exists
      if (!targetConversationId) {
        const title = utils.generateConversationTitle(content);
        const newConv = await createConversation(title);
        if (!newConv) return;
        targetConversationId = newConv.id;
      }

      setIsGenerating(true);
      setError(null);
      setPartialMessage("");

      try {
        // Send user message
        const userMessage = await messageApi.sendUserMessage(
          content,
          targetConversationId,
        );
        setMessages((prev) => [...prev, userMessage]);

        // Update conversation's last message and timestamp
        setConversations((prev) =>
          prev.map((conv) =>
            conv.id === targetConversationId
              ? {
                  ...conv,
                  last_message: content,
                  timestamp: userMessage.timestamp,
                }
              : conv,
          ),
        );

        // Generate AI response
        await messageApi.generateAIResponse(content, targetConversationId);

        // Note: isGenerating will be set to false by the message chunk listener
        // when the AI response is complete
      } catch (err) {
        const errorMsg = "Failed to send message";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
        setIsGenerating(false);
      }
    },
    [currentConversation, createConversation],
  );

  // Voice input
  const startVoiceInput = useCallback(
    async (conversationId?: number) => {
      if (!enableVoice) {
        setError("Voice input is disabled");
        return;
      }

      const targetConversationId = conversationId || currentConversation?.id;
      if (!targetConversationId) {
        setError("No conversation selected for voice input");
        return;
      }

      setVoiceStatus("recording");
      setVoiceTranscript("");
      setError(null);

      try {
        const transcript = await voiceApi.startVoiceInput(targetConversationId);
        if (transcript && transcript.trim()) {
          setVoiceTranscript(transcript);
          await sendMessage(transcript, targetConversationId);
        }
      } catch (err) {
        const errorMsg = "Voice input failed";
        setError(errorMsg);
        await utils.handleApiError(err, errorMsg);
      } finally {
        setVoiceStatus("idle");
      }
    },
    [currentConversation, enableVoice, sendMessage],
  );

  const stopVoiceInput = useCallback(() => {
    setVoiceStatus("idle");
    setVoiceTranscript("");
  }, []);

  // Clear functions
  const clearMessages = useCallback(() => {
    setMessages([]);
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const refreshData = useCallback(async () => {
    await loadConversations();
    if (currentConversation) {
      await selectConversation(currentConversation.id);
    }
  }, [loadConversations, selectConversation, currentConversation]);

  // Setup event listeners
  useEffect(() => {
    if (!enableStreaming) return;

    // Listen for message chunks
    const setupMessageChunkListener = async () => {
      if (messageChunkUnlisten.current) {
        messageChunkUnlisten.current();
      }

      messageChunkUnlisten.current = await messageApi.onMessageChunk(
        (chunk: MessageChunk) => {
          if (chunk.is_complete) {
            setIsGenerating(false);
            setPartialMessage("");

            // Refresh messages to get the final AI response from database
            setTimeout(async () => {
              if (currentConversation) {
                try {
                  const msgs = await conversationApi.getMessages(
                    currentConversation.id,
                  );
                  setMessages(msgs.sort((a, b) => a.timestamp - b.timestamp));

                  // Also refresh conversations to update last_message
                  await loadConversations();
                } catch (err) {
                  console.error("Failed to refresh messages:", err);
                }
              }
            }, 1000);
          } else {
            setPartialMessage((prev) => prev + chunk.content);
          }
        },
      );
    };

    setupMessageChunkListener();

    return () => {
      if (messageChunkUnlisten.current) {
        messageChunkUnlisten.current();
      }
    };
  }, [
    enableStreaming,
    currentConversation,
    selectConversation,
    loadConversations,
  ]);

  // Setup voice event listeners
  useEffect(() => {
    if (!enableVoice) return;

    const setupVoiceListeners = async () => {
      // Clean up existing listeners
      if (voiceStatusUnlisten.current) {
        voiceStatusUnlisten.current();
      }
      if (voicePartialUnlisten.current) {
        voicePartialUnlisten.current();
      }

      // Setup new listeners
      voiceStatusUnlisten.current = await voiceApi.onVoiceStatus(
        (status: VoiceStatus) => {
          setVoiceStatus(status);
        },
      );

      voicePartialUnlisten.current = await voiceApi.onVoicePartial(
        (text: string) => {
          setVoiceTranscript(text);
        },
      );
    };

    setupVoiceListeners();

    return () => {
      if (voiceStatusUnlisten.current) {
        voiceStatusUnlisten.current();
      }
      if (voicePartialUnlisten.current) {
        voicePartialUnlisten.current();
      }
    };
  }, [enableVoice]);

  // Auto-load conversations on mount
  useEffect(() => {
    if (autoLoadConversations) {
      loadConversations();
    }
  }, [autoLoadConversations, loadConversations]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (messageChunkUnlisten.current) {
        messageChunkUnlisten.current();
      }
      if (voiceStatusUnlisten.current) {
        voiceStatusUnlisten.current();
      }
      if (voicePartialUnlisten.current) {
        voicePartialUnlisten.current();
      }
    };
  }, []);

  const state: ChatState = {
    conversations,
    messages,
    currentConversation,
    isLoading,
    isGenerating,
    error,
    voiceStatus,
    voiceTranscript,
    partialMessage,
  };

  const actions: ChatActions = {
    loadConversations,
    selectConversation,
    createConversation,
    deleteConversation,
    sendMessage,
    clearMessages,
    startVoiceInput,
    stopVoiceInput,
    clearError,
    refreshData,
  };

  return [state, actions];
}

// Simplified hook for basic chat functionality
export function useSimpleChat() {
  return useChat({
    autoLoadConversations: true,
    enableVoice: false,
    enableStreaming: true,
  });
}

// Hook specifically for voice-enabled chat
export function useVoiceChat() {
  return useChat({
    autoLoadConversations: true,
    enableVoice: true,
    enableStreaming: true,
  });
}
