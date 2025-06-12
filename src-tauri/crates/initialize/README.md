# Initialize Crate

这个 crate 负责 ChatBox 应用的完整初始化流程，包括 AI 模型、语音识别、数据库等组件的初始化。

## 功能特性

### 1. 智能模型管理
- **Ollama 支持**: 自动检测 Ollama 服务可用性和模型存在性
- **Candle 支持**: 自动下载和初始化本地 Candle 模型
- **模型验证**: 确保模型文件完整性和配置正确性

### 2. 错误处理系统
- **交互式错误处理**: 在初始化失败时显示用户友好的错误对话框
- **用户选择**: 提供忽略、重试、退出三种处理选项
- **组件状态跟踪**: 记录每个组件的初始化状态

### 3. 渐进式初始化
- **模块化初始化**: 每个组件独立初始化，失败不影响其他组件
- **状态通知**: 实时通知前端初始化进度和状态
- **忽略继续**: 允许忽略非关键组件的初始化失败

## 配置选项

### AI 模型配置
```yaml
ai_model:
  model_type: "ollama"  # "ollama" 或 "candle"
  model_name: "qwen2.5:0.5b"
  server_url: "http://localhost"
  server_port: 11434
  system_prompt: "你是一个友好、乐于助人的AI助手"
  
  # Candle 特定设置
  candle_model_id: "microsoft/DialoGPT-medium"
  candle_revision: "main"
  candle_use_flash_attn: false
```

### 错误处理配置
```yaml
app_behavior:
  show_error_dialogs: true        # 是否显示错误对话框
  auto_retry_failed_init: false   # 是否自动重试失败的初始化
```

## 使用方法

### 基本使用
```rust
use initialize::{initialize_app, InitConfig};

// 加载配置
let config = InitConfig::new(config_path).load_config();

// 初始化应用
let result = initialize_app(config, app_handle).await?;

// 检查结果
if !result.success {
    println!("部分组件初始化失败: {:?}", result.failed_components);
    println!("被忽略的组件: {:?}", result.ignored_components);
}
```

### 错误处理
```rust
use initialize::{ErrorHandler, InitError, UserAction};

let error_handler = ErrorHandler::new(app_handle, true);
let action = error_handler.handle_error(&error, "AI模型").await?;

match action {
    UserAction::Retry => { /* 重试逻辑 */ },
    UserAction::Ignore => { /* 忽略并继续 */ },
    UserAction::Exit => { /* 退出应用 */ },
}
```

## 初始化流程

### 1. 配置验证
- 检查配置文件存在性和格式
- 验证必需配置项
- 创建缺失的目录结构

### 2. AI 模型初始化

#### Ollama 模式
1. 检查 Ollama 服务连接
2. 验证指定模型是否存在
3. 测试模型响应能力

#### Candle 模式
1. 检查本地模型文件
2. 从 Hugging Face 下载缺失模型
3. 验证模型文件完整性
4. 初始化推理引擎

### 3. 语音识别初始化
- 检查 Vosk 模型文件
- 验证 Python 环境
- 初始化语音识别引擎

### 4. 数据库初始化
- 创建数据库目录
- 初始化数据库连接
- 创建必要的表结构

## 错误类型

```rust
pub enum InitError {
    ModelUnavailable(String),    // 模型不可用
    ConfigLoadFailed(String),    // 配置加载失败
    DatabaseInitFailed(String),  // 数据库初始化失败
    VoiceInitFailed(String),     // 语音初始化失败
    NetworkError(String),        // 网络错误
    FileSystemError(String),     // 文件系统错误
}
```

## 组件状态

每个组件的初始化状态包括：
- `initialized`: 是否成功初始化
- `ignored`: 是否被用户忽略
- `error`: 错误信息（如果有）

## 前端集成

### 事件监听
```javascript
// 监听错误对话框事件
await listen('show_error_dialog', (event) => {
    // 显示错误对话框并返回用户选择
});

// 监听组件忽略通知
await listen('component_ignored', (event) => {
    // 显示组件忽略通知
});

// 监听初始化完成事件
await listen('initialization_complete', (event) => {
    // 显示初始化结果摘要
});
```

## 开发指南

### 添加新组件
1. 在 `AppInitializer` 中添加初始化方法
2. 在 `initialize_all` 中调用新方法
3. 定义相应的错误类型
4. 添加配置选项（如需要）

### 自定义错误处理
继承 `ErrorHandler` 并重写 `handle_error` 方法：
```rust
impl ErrorHandler {
    async fn handle_custom_error(&self, error: &MyError) -> Result<UserAction> {
        // 自定义错误处理逻辑
    }
}
```

## 最佳实践

1. **渐进增强**: 允许应用在部分组件失败时继续运行
2. **用户体验**: 提供清晰的错误信息和处理选项
3. **资源管理**: 合理处理下载、缓存和清理
4. **配置验证**: 在运行时验证配置的正确性
5. **状态通知**: 及时通知用户初始化进度和结果

## 依赖项

- `tauri`: Tauri 应用框架
- `tokio`: 异步运行时
- `serde`: 序列化/反序列化
- `log`: 日志记录
- `anyhow`: 错误处理
- `ollama-rs`: Ollama 客户端
- `candle-*`: Candle ML 框架
- `hf-hub`: Hugging Face Hub 客户端
- `rusqlite`: SQLite 数据库

## 许可证

MIT License