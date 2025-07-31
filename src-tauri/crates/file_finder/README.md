# File Finder

一个用于 Tauri 应用的高性能文件搜索和索引库，支持正则表达式搜索和内容匹配。

## 功能特性

- **快速文件索引**: 在应用启动时建立文件索引，提供快速搜索
- **正则表达式支持**: 支持复杂的正则表达式模式匹配
- **内容搜索**: 可以在文件内容中搜索指定模式
- **多种搜索选项**: 支持隐藏文件、文件类型过滤、结果数量限制等
- **Home目录权限**: 具有访问用户主目录的完整权限
- **异步操作**: 全异步设计，不阻塞UI线程
- **自动索引更新**: 支持检查和刷新文件索引

## 安装和设置

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
file_finder = { path = "crates/file_finder" }
```

### 2. 初始化插件

在 Tauri 应用的 `lib.rs` 中：

```rust
tauri::Builder::default()
    .plugin(file_finder::init())
    // ... 其他插件
```

### 3. 配置权限

在 `capabilities/default.json` 中添加必要权限：

```json
{
  "permissions": [
    "fs:default",
    "fs:allow-home-read-recursive",
    "fs:allow-home-meta-recursive",
    "fs:scope-home-recursive",
    {
      "identifier": "fs:allow-read-file",
      "allow": [{ "path": "$HOME/**" }]
    },
    {
      "identifier": "fs:allow-read-dir", 
      "allow": [{ "path": "$HOME/**" }]
    },
    {
      "identifier": "fs:allow-exists",
      "allow": [{ "path": "$HOME/**" }]
    },
    {
      "identifier": "fs:allow-stat",
      "allow": [{ "path": "$HOME/**" }]
    }
  ]
}
```

## API 使用

### Tauri 命令

以下命令可以从前端 JavaScript/TypeScript 调用：

#### 1. 搜索文件

```javascript
import { invoke } from '@tauri-apps/api/core';

// 基本搜索
const results = await invoke('search_files', {
  options: {
    pattern: 'config',
    use_regex: false,
    include_hidden: false,
    max_results: 10
  }
});

// 正则表达式搜索
const regexResults = await invoke('search_files', {
  options: {
    pattern: '\\.txt$',
    use_regex: true,
    max_results: 20
  }
});
```

#### 2. 内容搜索

```javascript
// 在文件内容中搜索
const contentResults = await invoke('search_files_with_content', {
  options: {
    pattern: 'fn main',
    search_content: true,
    max_results: 5
  }
});
```

#### 3. 刷新索引

```javascript
// 手动刷新文件索引
await invoke('refresh_file_index');
```

#### 4. 获取文件信息

```javascript
// 获取特定文件的详细信息
const fileInfo = await invoke('get_file_info', {
  path: '/path/to/file.txt'
});
```

#### 5. 获取索引统计

```javascript
// 获取索引状态
const stats = await invoke('get_index_stats');
console.log(`索引了 ${stats.file_count} 个文件`);
console.log(`需要更新: ${stats.needs_update}`);
```

### 搜索选项

`SearchOptions` 结构体支持以下选项：

```rust
pub struct SearchOptions {
    pub pattern: String,           // 搜索模式
    pub use_regex: bool,          // 是否使用正则表达式
    pub include_hidden: bool,     // 是否包含隐藏文件
    pub max_depth: Option<usize>, // 最大搜索深度
    pub file_types: Option<Vec<String>>, // 文件类型过滤
    pub max_results: Option<usize>,      // 最大结果数量
    pub search_content: bool,     // 是否搜索文件内容
}
```

### 文件信息结构

```rust
pub struct FileInfo {
    pub path: PathBuf,         // 文件路径
    pub name: String,          // 文件名
    pub size: u64,            // 文件大小（字节）
    pub modified: u64,        // 修改时间（Unix时间戳）
    pub is_dir: bool,         // 是否为目录
    pub extension: Option<String>, // 文件扩展名
}
```

## 使用示例

### 1. 基本文件名搜索

```javascript
const results = await invoke('search_files', {
  options: {
    pattern: 'config',
    use_regex: false,
    include_hidden: false,
    max_results: 10
  }
});

results.forEach(file => {
  console.log(`${file.name} - ${file.size} bytes`);
});
```

### 2. 正则表达式搜索

```javascript
// 搜索所有 .log 文件
const logFiles = await invoke('search_files', {
  options: {
    pattern: '\\.log$',
    use_regex: true,
    max_results: 50
  }
});

// 搜索以数字开头的文件
const numberedFiles = await invoke('search_files', {
  options: {
    pattern: '^\\d+',
    use_regex: true
  }
});
```

### 3. 按文件类型搜索

```javascript
// 搜索所有图片文件
const imageFiles = await invoke('search_files', {
  options: {
    pattern: '',
    file_types: ['jpg', 'jpeg', 'png', 'gif', 'svg'],
    max_results: 100
  }
});

// 搜索代码文件
const codeFiles = await invoke('search_files', {
  options: {
    pattern: '',
    file_types: ['rs', 'js', 'ts', 'py', 'java', 'cpp'],
    max_results: 200
  }
});
```

### 4. 内容搜索

```javascript
// 在文件内容中搜索特定文本
const results = await invoke('search_files_with_content', {
  options: {
    pattern: 'TODO',
    search_content: true,
    max_results: 20
  }
});

// 使用正则表达式搜索内容
const emailResults = await invoke('search_files_with_content', {
  options: {
    pattern: '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}',
    use_regex: true,
    search_content: true,
    max_results: 10
  }
});
```

## 性能考虑

- **索引构建**: 初次启动时会建立完整的文件索引，可能需要几秒到几分钟时间
- **内存使用**: 索引会占用一定内存，大约每10万个文件占用50-100MB
- **搜索速度**: 基于索引的搜索非常快，通常在毫秒级别
- **内容搜索**: 内容搜索较慢，建议限制结果数量和搜索范围

## 安全性

- 遵循 Tauri 的安全模型，只能访问已授权的目录
- 默认配置为访问用户主目录
- 支持 `.gitignore` 规则，自动跳过版本控制忽略的文件
- 不会访问系统敏感目录

## 错误处理

所有 API 调用都返回 `Result` 类型，需要适当处理错误：

```javascript
try {
  const results = await invoke('search_files', { options });
  // 处理结果
} catch (error) {
  console.error('搜索失败:', error);
}
```

## 日志和调试

库使用 `log` crate 记录调试信息：

- `INFO`: 索引构建和重要操作
- `WARN`: 非致命错误和警告
- `ERROR`: 严重错误

## 开发和测试

运行测试：

```bash
cargo test -p file_finder
```

运行示例：

```bash
cargo run --example usage -p file_finder
```

## 许可证

MIT License