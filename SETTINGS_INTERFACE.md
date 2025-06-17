# 设置界面文档

## 概述

全新的设置界面充分利用了后端 Rust API，提供了完整的可视化配置文件编辑功能。该界面使用现代化的 SvelteKit 组件构建，支持实时配置管理、验证和持久化。

## 核心功能

### 🔧 后端 API 集成
- **获取配置**: 使用 `get_app_config` API 从后端加载完整配置
- **保存配置**: 使用 `save_app_config` API 实时保存配置更改
- **重置配置**: 使用 `reset_app_config` API 恢复默认设置
- **实时同步**: 配置更改立即反映到应用行为

### 📱 响应式界面设计
- **移动端适配**: 完全响应式设计，支持各种屏幕尺寸
- **深色主题**: 支持浅色/深色主题切换
- **无障碍**: 完整的键盘导航和屏幕阅读器支持
- **动画效果**: 流畅的过渡动画和状态指示

### ⚡ 实时配置管理
- **变更检测**: 实时检测配置修改，显示未保存状态
- **自动验证**: 即时验证配置值的有效性
- **错误处理**: 详细的错误信息和修复建议
- **回滚功能**: 支持取消更改和恢复原始配置

## 界面布局

### 导航栏
左侧导航栏提供以下配置分类：

1. **🏠 通用设置** - 基本应用配置
2. **🤖 AI 模型** - AI 模型和推理设置
3. **🎤 语音识别** - 语音输入和识别配置
4. **🎨 界面设置** - 主题和界面偏好
5. **🗄️ 数据存储** - 数据库和存储设置
6. **🔧 高级设置** - 应用行为和性能调优
7. **📁 导入导出** - 配置文件管理

### 配置字段组件

#### ConfigFieldGroup
组织相关配置项的容器组件：
- 支持折叠/展开功能
- 统一的错误和警告显示
- 可选的必填标识
- 清晰的分组标题和描述

#### ConfigField
通用配置字段组件，支持多种输入类型：
- **文本输入**: 单行和多行文本
- **数字输入**: 数值输入带范围验证
- **选择器**: 下拉菜单选择
- **开关**: 布尔值切换
- **滑块**: 数值范围选择
- **文件选择**: 文件路径选择

## 配置结构

### AI 模型配置 (`ai_model`)
```typescript
{
  model_type: "candle" | "ollama",           // 推理引擎类型
  model_name: string,                        // 模型名称
  server_url: string,                        // 服务器地址
  server_port: string,                       // 端口号
  system_prompt: string,                     // 系统提示词
  candle_model_id?: string,                  // Candle 模型 ID
  candle_revision?: string,                  // 模型版本
  candle_use_flash_attn: boolean            // Flash Attention
}
```

### 语音配置 (`voice`)
```typescript
{
  enabled: boolean,                          // 启用语音识别
  model_path: string,                        // Vosk 模型路径
  timeout_seconds: number                    // 识别超时时间
}
```

### 界面配置 (`ui`)
```typescript
{
  theme: "light" | "dark" | "auto",         // 主题模式
  language: "zh-CN" | "en-US" | "ja-JP"    // 界面语言
}
```

### 数据库配置 (`database`)
```typescript
{
  enabled: boolean,                          // 启用数据库
  path: string                              // SQLite 文件路径
}
```

### 应用行为配置 (`app_behavior`)
```typescript
{
  log_level: string,                         // 日志级别
  default_conversation_title: string,       // 默认对话标题
  welcome_message: string,                   // 欢迎消息
  message_chunk_buffer_size: number,        // 消息缓冲区大小
  message_chunk_send_interval_ms: number,   // 发送间隔
  show_error_dialogs: boolean,              // 显示错误对话框
  auto_retry_failed_init: boolean           // 自动重试失败初始化
}
```

## 技术实现

### 配置服务 (`ConfigService`)
```typescript
// 获取配置
const config = await configService.getConfig();

// 保存配置
await configService.saveConfig(config);

// 重置配置
const defaultConfig = await configService.resetConfig();

// 监听配置变更
const unsubscribe = configService.addListener((event) => {
  console.log('配置变更:', event);
});
```

### 状态管理
- **响应式状态**: 使用 Svelte 的响应式系统管理配置状态
- **变更检测**: 自动检测配置修改并更新界面状态
- **错误处理**: 统一的错误处理和用户反馈机制

### 表单验证
内置完整的表单验证系统：
- **即时验证**: 用户输入时立即验证
- **自定义规则**: 针对不同字段类型的专门验证
- **错误提示**: 清晰的错误信息和修复建议
- **视觉反馈**: 错误状态的视觉高亮

## 用户体验特性

### 🎯 智能提示
- 配置字段提供详细的描述和帮助信息
- 推荐值和示例输入
- 上下文相关的提示信息

### 🔄 实时预览
- 主题更改立即生效
- 语言切换即时反映
- 配置更改状态实时显示

### 💾 数据持久化
- 自动保存到后端配置文件
- 支持配置的导入和导出
- 安全的配置重置功能

### 🛡️ 错误恢复
- 配置验证失败时的回滚机制
- 网络错误时的重试逻辑
- 详细的错误日志和用户提示

## 键盘快捷键

- `Ctrl/Cmd + S`: 保存当前配置
- `Ctrl/Cmd + R`: 重置配置（需确认）
- `Esc`: 取消当前更改
- `Tab/Shift+Tab`: 在配置字段间导航

## 无障碍支持

- **屏幕阅读器**: 完整的 ARIA 标签和语义化 HTML
- **键盘导航**: 所有功能均可通过键盘访问
- **高对比度**: 支持高对比度模式
- **字体缩放**: 响应系统字体大小设置

## 导入导出功能

### 配置导出
- 支持将当前配置导出为 JSON 文件
- 包含时间戳的文件命名
- 完整的配置结构保存

### 配置导入
- 支持从 JSON 文件导入配置
- 导入前的配置验证
- 导入失败时的错误提示和回滚

## 开发者指南

### 添加新配置字段
1. 在 `AppConfig` 接口中添加字段定义
2. 在后端 Rust 代码中更新对应结构
3. 在设置界面中添加对应的 `ConfigField` 组件
4. 添加必要的验证规则

### 自定义组件
所有配置组件都是模块化设计，可以轻松扩展：
- `ConfigFieldGroup`: 配置分组容器
- `ConfigField`: 通用配置字段
- `StatusIndicator`: 状态指示器

### 主题定制
界面使用 Tailwind CSS 构建，支持：
- 自定义颜色方案
- 响应式断点调整
- 深色模式适配

## 故障排除

### 常见问题

**配置无法保存**
- 检查后端服务是否正常运行
- 验证配置文件权限
- 查看控制台错误信息

**配置重置失败**
- 确认文件系统写入权限
- 检查配置文件路径是否正确
- 重启应用后重试

**界面显示异常**
- 清除浏览器缓存
- 检查网络连接
- 查看开发者工具中的错误信息

### 调试模式
在开发环境中启用详细日志：
```typescript
// 在 ConfigService 中启用调试模式
const config = await configService.getConfig();
console.log('当前配置:', config);
```

## 更新日志

### v1.0.0 (当前版本)
- ✅ 完整的后端 API 集成
- ✅ 响应式界面设计
- ✅ 实时配置验证
- ✅ 导入导出功能
- ✅ 无障碍支持
- ✅ 主题切换
- ✅ 多语言支持

### 计划功能
- 🔄 配置历史记录
- 🔄 批量配置操作
- 🔄 配置模板系统
- 🔄 高级搜索功能

## 许可证

本项目基于 MIT 许可证开源。