// Message types
export interface Message {
  id: number;
  content: string;
  sender: "user" | "bot";
  timestamp: number;
  conversation_id: number;
}

export interface MessageChunk {
  conversation_id: number;
  content: string;
  is_complete: boolean;
}

// Conversation types
export interface Conversation {
  id: number;
  title: string;
  last_message: string;
  timestamp: number;
}

// API Response types
export interface ErrorInfo {
  code: string;
  message: string;
  details?: string;
  timestamp: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ErrorInfo;
}

// Configuration types
export interface AppBehavior {
  message_chunk_buffer_size: number;
  message_chunk_send_interval_ms: number;
}

export interface AppConfig {
  app_behavior: AppBehavior;
  theme?: Theme;
  autoSave?: boolean;
  voiceEnabled?: boolean;
  apiEndpoint?: string;
}

// Voice recognition types
export type VoiceStatus =
  | "idle"
  | "recording"
  | "processing"
  | "completed"
  | "error";

export interface VoiceEvent {
  status: VoiceStatus;
  transcript?: string;
  partial?: string;
}

// Dialog types
export interface DialogFilter {
  name: string;
  extensions: string[];
}

export interface FileDialogOptions {
  title?: string;
  filters?: DialogFilter[];
  multiple?: boolean;
  defaultName?: string;
}

// System info types
export interface SystemInfo {
  os: string;
  arch: string;
  app_version: string;
  rust_version: string;
  timestamp: number;
}

// Health status types
export interface HealthStatus {
  config_loaded: boolean;
  database_connected: boolean;
  llm_available: boolean;
  voice_recognition_available: boolean;
  conversation_count: number;
  message_count: number;
  uptime_ms: number;
}

// UI State types
export interface UIState {
  selectedConversationId: number | null;
  isVoiceRecording: boolean;
  isGeneratingResponse: boolean;
  sidebarOpen: boolean;
  settingsOpen: boolean;
}

// Theme types
export type Theme = "light" | "dark" | "system";

// Chat input types
export interface ChatInputState {
  message: string;
  isSubmitting: boolean;
  voiceMode: boolean;
}

// Settings panel types
export interface SettingsTab {
  id: string;
  label: string;
  icon: string;
}

// Export utility types
export type ExportFormat = "json" | "txt" | "md";
export type ConfigFormat = "json" | "yaml";
