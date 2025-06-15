# Chat Box - AI 聊天助手

<div align="center">

![Chat Box Logo](./static/favicon.svg)

**基于 SvelteKit + Rust/Tauri 构建的现代化 AI 聊天桌面应用**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Svelte](https://img.shields.io/badge/Svelte-4A4A55?logo=svelte&logoColor=FF3E00)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)

[功能特性](#features) • [快速开始](#quick-start) • [开发指南](#development) • [构建部署](#build) • [贡献指南](#contributing)

</div>

![](/image/present.png)

## 📖 项目简介

Chat Box 是一款基于 SvelteKit 和 Rust/Tauri 构建的现代化 AI 聊天桌面应用。它提供了简洁美观的用户界面，强大的性能，以及丰富的功能特性，让您能够轻松地与 AI 助手进行对话。

## ✨ 功能特性 {#features}

### 🎯 核心功能
- **智能对话** - 支持多轮对话，上下文理解
- **对话管理** - 创建、删除、导出对话记录
- **实时响应** - 流式输出，即时显示 AI 回复
- **本地存储** - 使用 SQLite 本地数据库，数据安全可靠

### 🎨 用户界面
- **现代设计** - 简洁美观的 UI 设计
- **响应式布局** - 适配不同屏幕尺寸
- **主题切换** - 支持明暗主题自动切换
- **动画效果** - 流畅的过渡动画和交互效果

### 🛠️ 技术特性
- **跨平台** - 支持 Windows、macOS、Linux
- **高性能** - Rust 后端，极速响应
- **类型安全** - 全程 TypeScript 支持
- **模块化** - 组件化架构，易于扩展

### 🔧 开发特性
- **热重载** - 开发时自动刷新
- **代码检查** - ESLint + Prettier 代码规范
- **类型检查** - 完整的 TypeScript 类型系统
- **测试支持** - Vitest 单元测试框架

## 🚀 技术栈

### 前端
- **[SvelteKit](https://kit.svelte.dev/)** - 现代 Web 应用框架
- **[TypeScript](https://www.typescriptlang.org/)** - 类型安全的 JavaScript
- **[Vite](https://vitejs.dev/)** - 快速的构建工具
- **[Tauri API](https://tauri.app/)** - 桌面应用集成

### 后端
- **[Rust](https://www.rust-lang.org/)** - 系统级编程语言
- **[Tauri](https://tauri.app/)** - 跨平台桌面应用框架
- **[SQLite](https://www.sqlite.org/)** - 轻量级数据库
- **[Tokio](https://tokio.rs/)** - 异步运行时

### 开发工具
- **[ESLint](https://eslint.org/)** - 代码检查工具
- **[Prettier](https://prettier.io/)** - 代码格式化工具
- **[Vitest](https://vitest.dev/)** - 单元测试框架

## 📋 系统要求

### 开发环境
- **Node.js** >= 18.0.0
- **npm** >= 8.0.0 或 **pnpm** >= 8.0.0
- **Rust** >= 1.70.0
- **Git** 版本控制

### 运行环境
- **Windows** 10+ (x64)
- **macOS** 10.15+ (Intel/Apple Silicon)
- **Linux** (x64) - Ubuntu 18.04+, Debian 10+, CentOS 8+

## 🏁 快速开始 {#quick-start}

### 1. 克隆项目
```bash
git clone https://github.com/yourusername/chat-box-svelte.git
cd chat-box-svelte
```

### 2. 安装依赖
```bash
# 使用 npm
npm install

# 或使用 pnpm (推荐)
pnpm install
```

### 3. 配置环境
```bash
# 复制环境配置文件
cp .env.example .env

# 编辑配置文件 (可选)
# vim .env
```

### 4. 启动开发服务器
```bash
# 启动 Tauri 开发模式
npm run tauri:dev

# 或者单独启动前端开发服务器
npm run dev
```

### 5. 构建应用
```bash
# 构建生产版本
npm run tauri:build

# 构建调试版本
npm run tauri:build:debug
```

## 🔧 开发指南 {#development}

### 项目结构
```
chat-box-svelte/
├── src/                          # 前端源码
│   ├── lib/                      # 库文件
│   │   ├── components/           # Svelte 组件
│   │   │   ├── ui/              # UI 基础组件
│   │   │   └── Notification.svelte
│   │   ├── services/            # 服务层
│   │   │   ├── ErrorService.ts  # 错误处理服务
│   │   │   └── TauriService.ts  # Tauri API 服务
│   │   ├── stores/              # 状态管理
│   │   │   ├── chatStore.ts     # 聊天状态
│   │   │   └── themeStore.ts    # 主题状态
│   │   └── types/               # 类型定义
│   │       └── index.ts
│   ├── routes/                   # 路由页面
│   │   ├── +layout.svelte       # 布局组件
│   │   ├── +layout.ts           # 布局配置
│   │   └── +page.svelte         # 主页面
│   ├── app.css                  # 全局样式
│   └── app.html                 # HTML 模板
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   └── lib.rs               # 主要逻辑
│   ├── Cargo.toml               # Rust 依赖配置
│   └── tauri.conf.json          # Tauri 配置
├── static/                       # 静态资源
├── tests/                        # 测试文件
├── package.json                  # 项目配置
├── vite.config.js               # Vite 配置
├── svelte.config.js             # Svelte 配置
├── tsconfig.json                # TypeScript 配置
├── eslint.config.js             # ESLint 配置
└── .prettierrc                  # Prettier 配置
```

### 开发命令

```bash
# 开发
npm run dev                    # 启动前端开发服务器
npm run tauri:dev             # 启动 Tauri 开发模式

# 代码检查
npm run lint                  # 运行 ESLint
npm run format                # 格式化代码
npm run check                 # 类型检查
npm run check:watch           # 监听模式类型检查

# 测试
npm run test                  # 运行测试
npm run test:unit             # 运行单元测试
npm run test:watch            # 监听模式测试

# 构建
npm run build                 # 构建前端
npm run tauri:build           # 构建桌面应用
npm run preview               # 预览构建结果

# 清理
npm run clean                 # 清理构建文件
```

### 组件开发

#### 创建新组件
```typescript
// src/lib/components/ui/NewComponent.svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let prop1: string = '';
  export let prop2: number = 0;
  
  const dispatch = createEventDispatcher();
  
  function handleClick() {
    dispatch('click', { data: 'value' });
  }
</script>

<div class="new-component">
  <button on:click={handleClick}>
    {prop1}
  </button>
</div>

<style>
  .new-component {
    /* 样式 */
  }
</style>
```

#### 使用组件
```typescript
// 在其他组件中使用
import NewComponent from '$lib/components/ui/NewComponent.svelte';

<NewComponent 
  prop1="Hello" 
  prop2={42} 
  on:click={handleComponentClick} 
/>
```

### 状态管理

```typescript
// src/lib/stores/newStore.ts
import { writable, derived } from 'svelte/store';

interface NewState {
  data: string[];
  loading: boolean;
}

function createNewStore() {
  const { subscribe, set, update } = writable<NewState>({
    data: [],
    loading: false
  });

  return {
    subscribe,
    
    async loadData() {
      update(state => ({ ...state, loading: true }));
      try {
        const data = await fetchData();
        set({ data, loading: false });
      } catch (error) {
        update(state => ({ ...state, loading: false }));
        throw error;
      }
    },
    
    addItem(item: string) {
      update(state => ({
        ...state,
        data: [...state.data, item]
      }));
    }
  };
}

export const newStore = createNewStore();
export const isLoading = derived(newStore, $store => $store.loading);
```

### Tauri 集成

```typescript
// 调用 Rust 后端
import { invoke } from '@tauri-apps/api/core';
import type { ApiResponse } from '$lib/types';

export async function callRustFunction(param: string): Promise<string> {
  try {
    const response: ApiResponse<string> = await invoke('rust_function_name', {
      parameter: param
    });
    
    if (response.success && response.data) {
      return response.data;
    } else {
      throw new Error(response.error?.message || 'Unknown error');
    }
  } catch (error) {
    console.error('Failed to call Rust function:', error);
    throw error;
  }
}
```

## 🏗️ 构建部署 {#build}

### 开发构建
```bash
# 构建前端
npm run build

# 构建带调试信息的桌面应用
npm run tauri:build:debug
```

### 生产构建
```bash
# 构建优化版本
npm run tauri:build
```

构建产物将生成在 `src-tauri/target/release/bundle/` 目录下。

### 构建配置

#### Tauri 配置 (`src-tauri/tauri.conf.json`)
```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:1420",
    "distDir": "../build"
  },
  "package": {
    "productName": "Chat Box",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "copyright": "© 2024 Chat Box Team",
      "deb": {
        "depends": []
      },
      "externalBin": [],
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "com.chatbox.app",
      "longDescription": "A modern AI chat application built with SvelteKit and Rust/Tauri",
      "macOS": {
        "entitlements": null,
        "exceptionDomain": "",
        "frameworks": [],
        "providerShortName": null,
        "signingIdentity": null
      },
      "resources": [],
      "shortDescription": "AI Chat Assistant",
      "targets": "all",
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    }
  }
}
```

## ⚙️ 配置说明

### 环境变量
参考 `.env.example` 文件配置环境变量：

```bash
# 基本配置
NODE_ENV=development
TAURI_DEBUG=true

# AI 服务配置
OPENAI_API_KEY=your_key_here
OLLAMA_BASE_URL=http://localhost:11434

# 功能开关
FEATURE_VOICE_RECOGNITION=false
FEATURE_TEXT_TO_SPEECH=false
```

### TypeScript 配置
项目使用严格的 TypeScript 配置，确保类型安全：

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

## 🧪 测试

### 运行测试
```bash
# 运行所有测试
npm run test

# 运行单元测试
npm run test:unit

# 监听模式
npm run test:watch

# 测试覆盖率
npm run test:coverage
```

### 测试示例
```typescript
// tests/components/Button.test.ts
import { render, fireEvent } from '@testing-library/svelte';
import Button from '$lib/components/ui/Button.svelte';

describe('Button Component', () => {
  test('renders with correct text', () => {
    const { getByRole } = render(Button, { props: { text: 'Click me' } });
    const button = getByRole('button');
    expect(button).toHaveTextContent('Click me');
  });

  test('emits click event', async () => {
    const { getByRole, component } = render(Button);
    const button = getByRole('button');
    
    let clicked = false;
    component.$on('click', () => { clicked = true; });
    
    await fireEvent.click(button);
    expect(clicked).toBe(true);
  });
});
```

## 🤝 贡献指南 {#contributing}

我们欢迎任何形式的贡献！请查看以下指南：

### 开发流程
1. **Fork** 项目
2. **创建**特性分支 (`git checkout -b feature/amazing-feature`)
3. **提交**更改 (`git commit -m 'Add amazing feature'`)
4. **推送**到分支 (`git push origin feature/amazing-feature`)
5. **创建** Pull Request

### 代码规范
- 使用 **TypeScript** 编写代码
- 遵循 **ESLint** 和 **Prettier** 规则
- 编写**单元测试**
- 添加适当的**文档注释**

### 提交规范
使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
feat: 添加新功能
fix: 修复问题
docs: 更新文档
style: 代码格式调整
refactor: 代码重构
test: 添加测试
chore: 构建配置更新
```

### 问题报告
在报告问题时，请提供：
- **操作系统**和版本
- **Node.js** 和 **Rust** 版本
- **错误信息**和堆栈跟踪
- **重现步骤**

## 📚 更多资源

### 官方文档
- [SvelteKit 文档](https://kit.svelte.dev/docs)
- [Tauri 文档](https://tauri.app/v1/guides/)
- [Rust 文档](https://doc.rust-lang.org/)

### 社区资源
- [Svelte 中文文档](https://svelte.dev/)
- [Tauri 中文社区](https://tauri.app/)
- [Rust 中文社区](https://rust-lang.org/)

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 🙏 致谢

感谢以下开源项目和社区：
- [SvelteKit](https://kit.svelte.dev/) - 现代 Web 应用框架
- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Rust 社区](https://www.rust-lang.org/community) - 优秀的系统编程语言
- 所有贡献者和用户的支持

---

<div align="center">

**[⬆ 回到顶部](#chat-box---ai-聊天助手)**

Made with ❤️ by [Chat Box Team](https://github.com/yourusername)

</div>
