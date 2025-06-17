export interface Message {
  id: number;
  conversation_id: number;
  content: string;
  sender: "user" | "bot";
  timestamp: number;
}

export interface Conversation {
  id: number;
  title: string;
  last_message: string;
  timestamp: number;
}

export interface MessageChunk {
  conversation_id: number;
  content: string;
  is_complete: boolean;
}

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

export interface AppConfig {
  config_path: string;
  ai_model: {
    model_type: string;
    model_name: string;
    server_url: string;
    server_port: string;
    system_prompt: string;
    candle_model_id?: string;
    candle_revision?: string;
    candle_use_flash_attn: boolean;
  };
  voice: {
    enabled: boolean;
    model_path: string;
    timeout_seconds: number;
  };
  ui: {
    theme: string;
    language: string;
  };
  database: {
    enabled: boolean;
    path: string;
  };
  app_behavior: {
    log_level: string;
    default_conversation_title: string;
    welcome_message: string;
    message_chunk_buffer_size: number;
    message_chunk_send_interval_ms: number;
    show_error_dialogs: boolean;
    auto_retry_failed_init: boolean;
  };
}

export interface SystemInfo {
  os: string;
  arch: string;
  app_version: string;
  rust_version: string;
  timestamp: number;
}

export interface HealthStatus {
  config_loaded: boolean;
  database_connected: boolean;
  llm_available: boolean;
  voice_recognition_available: boolean;
  conversation_count: number;
  message_count: number;
  uptime_ms: number;
}
