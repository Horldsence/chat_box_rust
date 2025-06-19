// Live2D 动作类型
export type Live2DActionType =
  | "Speaking"
  | "Thinking"
  | "Happy"
  | "Confused"
  | "Surprised"
  | "Greeting"
  | "Farewell"
  | "Idle"
  | "Typing"
  | "Listening"
  | { Custom: string };

// Live2D 动作配置
export interface Live2DAction {
  action_type: Live2DActionType;
  motion_group: string;
  motion_index?: number;
  expression?: string;
  duration?: number;
  priority: number;
}

// Live2D 事件
export interface Live2DEvent {
  event_type: string;
  action: Live2DAction;
  timestamp: number;
  metadata: Record<string, string>;
}

// Live2D 配置
export interface Live2DConfig {
  model_path: string;
  scale: number;
  position: [number, number];
  auto_blink: boolean;
  auto_breath: boolean;
  default_actions: Record<string, Live2DAction>;
  text_triggers: Record<string, string>;
}

// Live2D 状态
export interface Live2DState {
  current_action?: Live2DAction;
  action_queue: Live2DAction[];
  is_speaking: boolean;
  last_action_time: number;
}

// Agent 相关类型定义
export type AgentRole =
  | "Assistant"
  | "Friend"
  | "Mentor"
  | { Expert: string }
  | "Entertainment"
  | { Custom: string };

export interface AgentPersonality {
  friendliness: number;
  professionalism: number;
  humor: number;
  patience: number;
  creativity: number;
  expression_style: string;
  language_preference: string;
}

export type ResponseLength = "Brief" | "Moderate" | "Detailed" | "Adaptive";

export interface AgentBehavior {
  response_length: ResponseLength;
  use_emojis: boolean;
  ask_questions: boolean;
  offer_suggestions: boolean;
  remember_context: boolean;
  personalized_responses: boolean;
}

export interface AgentLive2DIntegration {
  enabled: boolean;
  emotion_mapping: Record<string, string>;
  action_triggers: Record<string, string>;
  auto_expression: boolean;
  speaking_actions: string[];
  thinking_actions: string[];
}

export interface AgentConfig {
  name: string;
  role: AgentRole;
  system_prompt: string;
  personality: AgentPersonality;
  behavior: AgentBehavior;
  live2d_integration: AgentLive2DIntegration;
  preset_responses: Record<string, string>;
  knowledge_domains: string[];
  created_at: number;
  updated_at: number;
}

export interface AgentTemplate {
  id: string;
  name: string;
  description: string;
  config: AgentConfig;
  preview_image?: string;
  tags: string[];
}

export interface AgentState {
  current_config: AgentConfig;
  conversation_context: string[];
  user_preferences: Record<string, string>;
  session_start_time: number;
  interaction_count: number;
}

// Live2D 组件相关类型
export interface Live2DProps {
  modelPath?: string;
  scale?: number;
  position?: [number, number];
  className?: string;
  onActionChange?: (action: Live2DAction) => void;
  onExpressionChange?: (expression: string) => void;
}

export interface Live2DControlProps {
  className?: string;
  onActionExecuted?: (action: string) => void;
}

export interface AgentConfigProps {
  onConfigChange?: (config: AgentConfig) => void;
  className?: string;
}

// Tauri 命令相关类型
export interface TauriResponse<T> {
  data?: T;
  error?: string;
}

// 事件监听器类型
export type Live2DEventListener = (event: Live2DEvent) => void;
export type AgentEventListener = (event: any) => void;

// 预定义动作配置
export interface PredefinedAction {
  id: string;
  label: string;
  icon: any; // React component
  color: string;
}

// 表情配置
export type ExpressionType =
  | "default"
  | "happy"
  | "sad"
  | "angry"
  | "surprised"
  | "confused"
  | "thinking"
  | "excited"
  | "sleepy"
  | "focused"
  | "smile"
  | "wink";

// 错误类型
export interface Live2DError {
  code: string;
  message: string;
  details?: any;
}

export interface AgentError {
  code: string;
  message: string;
  details?: any;
}

// 配置更新类型
export interface ConfigUpdate<T> {
  field: keyof T;
  value: any;
}

// 状态更新类型
export interface StateUpdate<T> {
  field: keyof T;
  value: any;
}

// 通知类型
export interface NotificationMessage {
  id: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  message: string;
  duration?: number;
}

// Live2D 模型加载状态
export interface ModelLoadState {
  isLoading: boolean;
  isLoaded: boolean;
  error?: string;
  progress?: number;
}

// Agent 模板分类
export interface TemplateCategory {
  id: string;
  name: string;
  description: string;
  templates: AgentTemplate[];
}

// Live2D 性能统计
export interface Live2DPerformance {
  fps: number;
  memory_usage: number;
  render_time: number;
  last_update: number;
}

// Agent 对话统计
export interface AgentStats {
  total_interactions: number;
  average_response_time: number;
  most_used_actions: string[];
  session_duration: number;
}
