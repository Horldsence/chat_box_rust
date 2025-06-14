# Chat Box Rust

*[English](README.md) | [中文](README-CN.md)*

<!-- PROJECT SHIELDS -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Horldsence/chat_box_rust)

<!-- PROJECT LOGO -->
<br />

<p align="center">
  <a href="https://github.com/horldsence/chat_box_rust/">
    <img src="src-tauri/icons/128x128@2x.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Chat Box Rust</h3>
  <p align="center">
    A cross-platform high-performance AI chat application built with Rust
    <br />
    <a href="https://github.com/horldsence/chat_box_rust"><strong>Explore the documentation »</strong></a>
    <br />
    <br />
    <a href="https://github.com/horldsence/chat_box_rust">View Demo</a>
    ·
    <a href="https://github.com/horldsence/chat_box_rust/issues">Report Bug</a>
    ·
    <a href="https://github.com/horldsence/chat_box_rust/issues">Request Feature</a>
  </p>

</p>

> This README is designed for developers

A modern desktop AI chat application built with Tauri, Vue 3, and Rust, supporting multi-platform deployment including RISC-V architecture.

![Demo Screenshot](image/present.png)

## Table of Contents

- [Project Overview](#project-overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Standard Installation](#standard-installation)
  - [RISC-V Environment Setup](#risc-v-environment-setup)
- [Usage Guide](#usage-guide)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Language Support](#language-support)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Acknowledgments](#acknowledgments)

## Project Overview

Chat Box Rust is a local-first AI conversation application designed to provide secure, efficient, and user-friendly human-AI interaction. By integrating large language models, users can engage in natural language conversations while enjoying a modern interface and smooth interactions. The project is specially optimized for performance on RISC-V architecture, providing native support for next-generation computing platforms.

## Features

- 🌟 **NEW** Added Candle support for local AI inference
- 📝 Multi-conversation management with chat history
- 🔊 Voice input with real-time speech-to-text
- 🖥️ Modern Material You design interface
- 📊 Markdown format support with code highlighting
- 🏎️ Streaming responses with real-time AI replies
- 🛡️ Local-first approach protecting data privacy
- 🌍 Cross-platform support including RISC-V architecture
- 🔧 Dual AI backend support (Ollama & Candle)
- 💾 SQLite database for conversation storage
- 🎨 Element Plus UI components with Vue 3

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Element Plus
- **Backend**: Rust + Tauri
- **Data Storage**: SQLite
- **AI Frameworks**:
  - Ollama (for remote/local model serving)
  - Candle (for native Rust AI inference)
- **Voice Processing**: Vosk (speech recognition)
- **Audio**: Rodio (audio playback)

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (>= 16.0.0)
- [Rust](https://www.rust-lang.org/) (>= 1.60.0)
- [Tauri Development Environment](https://tauri.app/v1/guides/getting-started/prerequisites)

### Standard Installation

1. Clone the repository

```bash
git clone https://github.com/horldsence/chat_box_rust.git
cd chat_box_rust
```

2. Install dependencies

```bash
npm install
```

3. Install Ollama (for x86/AMD64 environments)

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

4. Run in development mode

```bash
npm run tauri dev
```

5. Build the application

```bash
npm run tauri build
```

### RISC-V Environment Setup

For RISC-V architecture, please note the following considerations:

#### 1. System Requirements

- RISC-V 64 architecture processor
- At least 4GB RAM
- RISC-V-compatible Linux distribution (e.g., Debian riscv64, Fedora riscv64)

#### 2. Rust Toolchain Installation

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Select default installation options
source $HOME/.cargo/env
```

#### 3. Frontend Dependencies Installation

```bash
# Ensure Node.js RISC-V version is installed
npm install
```

#### 4. Manual Ollama Compilation (RISC-V Specific)

Since Ollama doesn't provide official RISC-V precompiled packages, source compilation is required:

```bash
# Install necessary dependencies
sudo apt-get update
sudo apt-get install -y build-essential git cmake golang-go

# Clone Ollama source code
git clone https://github.com/ollama/ollama.git
cd ollama

# Compile Ollama (may need to adjust GOARCH parameter)
go build -o ollama cmd/ollama/main.go

# Install to system path
sudo cp ollama /usr/local/bin/
```

#### 5. RISC-V Specific Optimizations

```bash
# Create RISC-V optimization configuration in project directory
cat > .cargo/config.toml << EOF
[target.riscv64gc-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]
EOF
```

#### 6. Build and Run

```bash
# Build for RISC-V environment
npm run tauri build -- --target riscv64gc-unknown-linux-gnu

# Run the built application
./src-tauri/target/riscv64gc-unknown-linux-gnu/release/chat-box
```

## Usage Guide

1. **Create New Conversation**: Click the "+" button in the left panel to create a new conversation
2. **Message Interaction**: Type your question in the input box, then press send button or Enter key
3. **Voice Input**: Click the microphone icon to start voice input
4. **Manage Conversations**: Select, rename, or delete conversations in the left panel
5. **Configure Settings**: Adjust AI models, interface, and voice parameters through the settings panel

See the complete documentation for more features.

## Project Structure

```
chat_box_rust/
├── src/                       # Frontend code
│   ├── components/           # Vue components
│   ├── pages/                # Application pages
│   ├── services/             # Frontend services
│   ├── stores/               # Pinia state management
│   ├── styles/               # Stylesheets
│   ├── types/                # TypeScript type definitions
│   └── utils/                # Utility functions
├── src-tauri/                # Tauri backend code
│   ├── crates/               # Rust workspace crates
│   │   ├── agent/           # AI agent implementation
│   │   └── initialize/      # Initialization logic
│   ├── database/             # Database files
│   └── src/                  # Main Rust source code
│       ├── commands/         # Tauri commands
│       ├── services/         # Backend services
│       └── utils/            # Utility modules
├── config.yaml              # Application configuration
└── package.json              # Node.js dependencies
```

## Configuration

The application uses a `config.yaml` file for configuration:

```yaml
ai_model:
  model_type: "candle"  # "ollama" or "candle"
  model_name: "qwen2.5:0.5b"
  server_url: "http://localhost"
  server_port: 11434
  system_prompt: "You are a helpful AI assistant."

voice:
  enabled: false
  model_path: "model/vosk-model-small-en-us-0.15"
  timeout_seconds: 15

ui:
  theme: "light"
  language: "en-US"

database:
  enabled: true
  path: "database/chat_database.db"
```

### AI Model Configuration

- **Ollama Mode**: Uses external Ollama server for model inference
- **Candle Mode**: Uses built-in Rust-native AI inference with Hugging Face models

## Language Support

Chat Box Rust provides documentation in multiple languages:

- [English (README.md)](README.md)
- [中文 (README-CN.md)](README-CN.md)

The application interface also supports multiple languages which can be configured in the `config.yaml` file:

```yaml
ui:
  language: "en-US"  # Change to "zh-CN" for Chinese interface
```

## Roadmap

- [ ] Support for more large language models
- [ ] Improve RISC-V architecture performance optimization
- [ ] Add text-to-speech functionality
- [ ] Enhance offline mode experience
- [ ] Implement plugin system
- [ ] Multi-language interface support
- [ ] Cloud synchronization capabilities

See [open issues](https://github.com/horldsence/chat_box_rust/issues) for more information.

## Contributing

Contributions make the open-source community an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

Project Maintainer: Peng - horldjason@outlook.com

Project Link: [https://github.com/horldsence/chat_box_rust](https://github.com/horldsence/chat_box_rust)

## Acknowledgments

- [Tauri](https://tauri.app/) - For the excellent desktop app framework
- [Vue.js](https://vuejs.org/) - For the reactive frontend framework
- [Rust](https://www.rust-lang.org/) - For the powerful systems programming language
- [Ollama](https://ollama.com/) - For local LLM serving
- [Candle](https://github.com/huggingface/candle) - For Rust-native machine learning
- [Element Plus](https://element-plus.org/) - For beautiful UI components
- [RISC-V Foundation](https://riscv.org/) - For the open instruction set architecture

<!-- links -->
[your-project-path]:horldsence/chat_box_rust
[contributors-shield]: https://img.shields.io/github/contributors/horldsence/chat_box_rust.svg?style=flat-square
[contributors-url]: https://github.com/horldsence/chat_box_rust/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/horldsence/chat_box_rust.svg?style=flat-square
[forks-url]: https://github.com/horldsence/chat_box_rust/network/members
[stars-shield]: https://img.shields.io/github/stars/horldsence/chat_box_rust.svg?style=flat-square
[stars-url]: https://github.com/horldsence/chat_box_rust/stargazers
[issues-shield]: https://img.shields.io/github/issues/horldsence/chat_box_rust.svg?style=flat-square
[issues-url]: https://img.shields.io/github/issues/horldsence/chat_box_rust.svg
[license-shield]: https://img.shields.io/github/license/horldsence/chat_box_rust.svg?style=flat-square
[license-url]: https://github.com/horldsence/chat_box_rust/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=flat-square&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/shaojintian
