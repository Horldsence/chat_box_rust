# Chat Box 前端项目结构

## 项目概述

这是一个基于 Tauri + Vue.js + TypeScript 的现代化 AI 聊天应用的前端部分。项目采用多页面应用架构，支持主应用、聊天窗口、设置页面和关于页面。

## 文件夹结构

```
src/
├── README.md                    # 项目结构说明
├── App.vue                      # Vue 主应用组件
├── vite-env.d.ts               # Vite 环境类型定义
│
├── api/                        # API 接口层
│   └── ...                     # API 相关文件
│
├── assets/                     # 静态资源
│   ├── styles/                 # 全局样式文件
│   └── ...                     # 图片、字体等资源
│
├── components/                 # Vue 组件
│   └── ...                     # 可复用的 Vue 组件
│
├── pages/                      # 页面文件
│   ├── main/                   # 主页面
│   │   ├── index.html          # 主页面 HTML
│   │   └── main.ts             # 主页面入口脚本
│   │
│   ├── chat/                   # 聊天页面
│   │   ├── chat.html           # 聊天页面 HTML
│   │   └── chat.ts             # 聊天页面脚本
│   │
│   ├── settings/               # 设置页面
│   │   ├── settings.html       # 设置页面 HTML
│   │   └── settings.ts         # 设置页面脚本
│   │
│   └── about/                  # 关于页面
│       ├── about.html          # 关于页面 HTML
│       └── about.ts            # 关于页面脚本
│
├── stores/                     # 状态管理
│   └── ...                     # Pinia stores
│
├── styles/                     # 样式文件
│   └── ...                     # 组件样式、主题等
│
├── types/                      # TypeScript 类型定义
│   └── ...                     # 接口和类型定义
│
└── utils/                      # 工具函数
    ├── clipboard.ts            # 剪贴板工具
    ├── markdown.ts             # Markdown 处理
    └── ...                     # 其他工具函数
```

## 页面说明

### 主页面 (main/)
- **index.html**: Vue 应用的主入口页面
- **main.ts**: Vue 应用的初始化脚本，加载主组件

### 聊天页面 (chat/)
- **chat.html**: 独立的聊天窗口页面
- **chat.ts**: 聊天功能的完整实现，包括消息发送、接收、语音输入等

### 设置页面 (settings/)
- **settings.html**: 应用设置界面
- **settings.ts**: 设置页面逻辑，包括 API 配置、参数调整等

### 关于页面 (about/)
- **about.html**: 应用信息展示页面
- **about.ts**: 关于页面交互逻辑，包括版本检查、链接跳转等

## 技术栈

- **Tauri**: 跨平台桌面应用框架
- **Vue 3**: 主应用框架
- **TypeScript**: 类型安全的 JavaScript
- **Element Plus**: UI 组件库
- **Vite**: 构建工具

## 开发规范

### 文件命名
- HTML 文件：使用页面名称，如 `chat.html`
- TypeScript 文件：使用页面名称，如 `chat.ts`
- 目录名称：使用小写字母，如 `settings/`

### 代码结构
1. **导入语句**: 按照外部库 -> 内部模块的顺序
2. **类型定义**: 在文件顶部定义接口和类型
3. **DOM 元素**: 集中获取 DOM 元素引用
4. **事件监听**: 统一在 `setupEventListeners` 函数中处理
5. **初始化**: 使用 `init` 函数作为入口点

### TypeScript 规范
- 严格类型检查，避免使用 `any`
- 为 DOM 元素添加类型断言
- 定义清晰的接口和类型
- 使用 async/await 处理异步操作

## 构建配置

项目使用 Vite 进行构建，配置文件为根目录的 `vite.config.ts`。多页面配置通过 `build.rollupOptions.input` 指定各页面的入口文件。

## 快捷键

### 聊天页面
- `Escape`: 关闭窗口
- `Ctrl + L`: 清空对话
- `Ctrl + E`: 导出对话
- `Ctrl + Enter`: 发送消息

### 设置页面
- `Escape`: 关闭窗口
- `Ctrl + S`: 保存设置

### 关于页面
- `Escape`: 关闭窗口
- `Ctrl + U`: 检查更新

## 注意事项

1. 每个页面都是独立的 HTML 文件，可以单独打开和运行
2. TypeScript 文件需要通过 `<script type="module">` 标签引入
3. 所有页面都支持 Tauri API 调用
4. 样式采用内联 CSS，保持页面的独立性
5. 国际化暂时使用中文，后续可扩展多语言支持