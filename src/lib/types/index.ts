// ===== 核心数据类型 =====

export interface Conversation {
  id: number;
  title: string;
  last_message: string;
  timestamp: number;
}

export interface Message {
  id: number;
  conversation_id: number;
  content: string;
  sender: string;
  timestamp: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
}

export interface ApiError {
  code: string;
  message: string;
  details?: string;
  timestamp: string;
}

// ===== 用户界面相关类型 =====

export type Theme = "light" | "dark" | "auto";

export interface AppSettings {
  theme: Theme;
  language: string;
  fontSize: "small" | "medium" | "large";
  autoSave: boolean;
  soundEnabled: boolean;
  notificationsEnabled: boolean;
}

export interface ChatSettings {
  maxMessages: number;
  showTimestamps: boolean;
  enableMarkdown: boolean;
  autoScroll: boolean;
  typingIndicator: boolean;
}

// ===== 通知系统 =====

export type NotificationType = "success" | "error" | "warning" | "info";

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  duration?: number;
  actions?: NotificationAction[];
  timestamp: number;
}

export interface NotificationAction {
  label: string;
  action: () => void;
  type: "primary" | "secondary";
}

// ===== 错误处理 =====

export interface ErrorInfo {
  id: string;
  code: string;
  title: string;
  message: string;
  details?: string;
  timestamp: number;
  type: "error" | "warning" | "info" | "success";
  category: "network" | "business" | "system" | "validation" | "unknown";
  severity: "low" | "medium" | "high" | "critical";
  source?: string;
  retryable?: boolean;
  userAction?: string;
}

export interface ErrorHandlerOptions {
  showNotification?: boolean;
  showDialog?: boolean;
  logToConsole?: boolean;
  logToBackend?: boolean;
  autoRetry?: boolean;
  retryCount?: number;
  retryDelay?: number;
}

// ===== 语音识别 =====

export interface VoiceRecognition {
  isSupported: boolean;
  isRecording: boolean;
  isProcessing: boolean;
  result?: string;
  confidence?: number;
  error?: string;
}

export interface VoiceSettings {
  enabled: boolean;
  language: string;
  continuous: boolean;
  interimResults: boolean;
  maxAlternatives: number;
}

// ===== 聊天相关 =====

export interface ChatState {
  currentConversation: Conversation | null;
  conversations: Conversation[];
  messages: Message[];
  isLoading: boolean;
  isTyping: boolean;
  error: string | null;
}

export interface SendMessageOptions {
  conversationId: string;
  content: string;
  sender: "user" | "assistant";
  metadata?: Record<string, any>;
}

export interface MessageFilter {
  conversationId?: string;
  sender?: "user" | "assistant";
  dateFrom?: string;
  dateTo?: string;
  searchText?: string;
}

// ===== 文件导入导出 =====

export interface ExportOptions {
  format: "txt" | "json" | "markdown" | "html";
  includeMetadata: boolean;
  dateRange?: {
    from: string;
    to: string;
  };
  conversations?: string[];
}

export interface ImportResult {
  success: boolean;
  conversationsImported: number;
  messagesImported: number;
  errors: string[];
}

// ===== 快捷操作 =====

export interface QuickAction {
  id: string;
  label: string;
  text: string;
  icon?: string;
  category: string;
  order: number;
  enabled: boolean;
}

export interface QuickActionCategory {
  id: string;
  name: string;
  icon?: string;
  order: number;
  actions: QuickAction[];
}

// ===== 搜索 =====

export interface SearchOptions {
  query: string;
  conversationId?: string;
  sender?: "user" | "assistant";
  caseSensitive?: boolean;
  wholeWords?: boolean;
  regex?: boolean;
}

export interface SearchResult {
  message: Message;
  conversation: Conversation;
  highlights: {
    start: number;
    end: number;
  }[];
  score: number;
}

export interface SearchState {
  isSearching: boolean;
  query: string;
  results: SearchResult[];
  currentIndex: number;
  total: number;
  error: string | null;
}

// ===== 统计信息 =====

export interface ChatStats {
  totalConversations: number;
  totalMessages: number;
  totalUserMessages: number;
  totalAssistantMessages: number;
  averageMessagesPerConversation: number;
  mostActiveDay: string;
  firstMessageDate: string;
  lastMessageDate: string;
}

export interface MessageStats {
  wordCount: number;
  characterCount: number;
  averageWordsPerMessage: number;
  longestMessage: Message;
  shortestMessage: Message;
}

// ===== 窗口状态 =====

export interface WindowState {
  isMaximized: boolean;
  isMinimized: boolean;
  isFullscreen: boolean;
  size: {
    width: number;
    height: number;
  };
  position: {
    x: number;
    y: number;
  };
}

// ===== 键盘快捷键 =====

export interface KeyboardShortcut {
  id: string;
  name: string;
  description: string;
  keys: string[];
  action: () => void;
  enabled: boolean;
  global?: boolean;
}

// ===== 插件系统 =====

export interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  config?: Record<string, any>;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  main: string;
  permissions: string[];
  dependencies?: Record<string, string>;
}

// ===== 用户偏好 =====

export interface UserPreferences {
  app: AppSettings;
  chat: ChatSettings;
  voice: VoiceSettings;
  shortcuts: KeyboardShortcut[];
  quickActions: QuickAction[];
  plugins: Plugin[];
}

// ===== 应用状态 =====

export interface AppState {
  initialized: boolean;
  loading: boolean;
  error: string | null;
  settings: UserPreferences;
  chat: ChatState;
  search: SearchState;
  voice: VoiceRecognition;
  window: WindowState;
  notifications: Notification[];
}

// ===== 事件类型 =====

export interface ChatEvent {
  type:
    | "message_sent"
    | "message_received"
    | "conversation_created"
    | "conversation_deleted"
    | "typing_start"
    | "typing_stop";
  payload: any;
  timestamp: number;
}

export interface SystemEvent {
  type: "app_ready" | "window_focused" | "window_blurred" | "theme_changed" | "settings_updated";
  payload?: any;
  timestamp: number;
}

// ===== API 相关 =====

export interface TauriCommand {
  command: string;
  args?: Record<string, any>;
  timeout?: number;
}

export interface TauriEventPayload<T = any> {
  event: string;
  payload: T;
  windowLabel?: string;
}

// ===== 工具函数类型 =====

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type RequiredFields<T, K extends keyof T> = T & Required<Pick<T, K>>;

// ===== 数据库相关 =====

export interface DatabaseConnection {
  connected: boolean;
  version: string;
  path: string;
  size: number;
  lastBackup?: string;
}

export interface BackupOptions {
  includeSettings: boolean;
  includeConversations: boolean;
  compress: boolean;
  encryption: boolean;
  destination: string;
}

export interface RestoreOptions {
  backupPath: string;
  overwriteExisting: boolean;
  restoreSettings: boolean;
  restoreConversations: boolean;
}

// ===== 辅助类型 =====

export type Prettify<T> = {
  [K in keyof T]: T[K];
} & {};

export type ValueOf<T> = T[keyof T];

export type ArrayElement<T> = T extends readonly (infer U)[] ? U : never;

export type NonNullable<T> = T extends null | undefined ? never : T;

export type Awaited<T> = T extends Promise<infer U> ? U : T;
