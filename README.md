# Chat Box Rust
# 聊天盒子

*[English](README.md) | [中文](README-CN.md)*

<!-- PROJECT SHIELDS -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Horldsence/chat_box_rust)

<br />

<p align="center">
  <a href="https://github.com/horldsence/chat_box_rust/">
    <img src="src-tauri/icons/128x128@2x.png" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Chat_Box_Rust</h3>
  <p align="center">
    A cross-platform high-performance chat application built with Rust
    <br />
    <a href="https://github.com/horldsence/chat_box_rust"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/horldsence/chat_box_rust">View Demo</a>
    ·
    <a href="https://github.com/horldsence/chat_box_rust/issues">Report Bug</a>
    ·
    <a href="https://github.com/horldsence/chat_box_rust/issues">Request Feature</a>
  </p>
</p>

> This README.md is intended for developers

![Demo Image](image/present.png)

## Project Overview

Chat Box is a local-first AI conversation app designed to provide secure, efficient, and user-friendly interactions. By integrating large language models, users can enjoy natural language dialogues with a modern interface. The project is optimized for RISC-V architecture, offering native support for next-generation platforms.

## Features

- 🌟 **NEW** Candle support for on-device AI inference  
- 📝 Multi-conversation management with history  
- 🔊 Voice input and real-time speech-to-text  
- 🖥️ Material You–inspired modern UI  
- 📊 Markdown rendering with code highlighting  
- 🏎️ Streaming responses for live updates  
- 🛡️ Local-first to safeguard data privacy  
- 🌍 Cross-platform support, including RISC-V  
- 🔧 Dual AI backends (Ollama & Candle)  
- 💾 SQLite for persistent local storage  
- 🎨 Element Plus UI components with Vue 3

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite  
- **Backend**: Rust + Tauri  
- **Storage**: SQLite  
- **AI Models**: Ollama framework support

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 16.0.0  
- [Rust](https://www.rust-lang.org/) ≥ 1.60.0  
- Tauri development environment ([guide](https://tauri.app/v1/guides/getting-started/prerequisites))

### Standard Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/horldsence/chat_box_rust.git
   cd chat_box_rust
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Install Ollama (x86/AMD64):
   ```bash
   curl -fsSL https://ollama.com/install.sh | sh
   ```
4. Run in development mode:
   ```bash
   npm run tauri dev
   ```
5. Build the application:
   ```bash
   npm run tauri build
   ```

### RISC-V Environment Setup

To support RISC-V, follow these steps:

1. System requirements:
   - RISC-V 64 CPU  
   - ≥ 4 GB RAM  
   - Linux distribution with RISC-V support

2. Install Rust toolchain:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. Install frontend dependencies:
   ```bash
   npm install
   ```

4. Build Ollama from source:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential git cmake golang-go
   git clone https://github.com/ollama/ollama.git
   cd ollama
   go build -o ollama cmd/ollama/main.go
   sudo cp ollama /usr/local/bin/
   ```

5. Add RISC-V Rust configuration:
   ```bash
   cat > .cargo/config.toml << EOF
   [target.riscv64gc-unknown-linux-gnu]
   rustflags = ["-C", "target-cpu=native"]
   EOF
   ```

6. Build and run:
   ```bash
   npm run tauri build -- --target riscv64gc-unknown-linux-gnu
   ./src-tauri/target/riscv64gc-unknown-linux-gnu/release/chat-box
   ```

## Usage Guide

1. **New Conversation**: Click the "+" in the sidebar  
2. **Chat**: Type your message and press Enter or Send  
3. **Voice Input**: Click the microphone icon  
4. **Manage Conversations**: Rename or delete from the list  
5. **Settings**: Configure AI model, UI, and voice options  

Refer to the full documentation for more advanced features.

## Project Structure

```
chat_box_rust/
├── src/                  # Frontend code
│   ├── components/       # Vue components
│   ├── pages/            # Application pages
│   ├── services/         # Frontend services
│   ├── stores/           # Pinia state management
│   ├── styles/           # Stylesheets
│   ├── types/            # TypeScript definitions
│   └── utils/            # Utility functions
├── src-tauri/            # Tauri backend
│   ├── crates/           # Rust workspace crates
│   │   ├── agent/        # AI agent implementation
│   │   └── initialize/   # Initialization logic
│   ├── database/         # Database files
│   └── src/              # Rust source code
│       ├── commands/     # Tauri commands
│       ├── services/     # Backend services
│       └── utils/        # Helper modules
├── config.yaml           # Application configuration
└── package.json          # Node.js dependencies
```

## Configuration

Edit `config.yaml` to customize settings:

```yaml
ai_model:
  model_type: "candle"    # "ollama" or "candle"
  model_name: "qwen2.5:0.5b"
  server_url: "http://localhost"
  server_port: 11434
  system_prompt: "You are a friendly and helpful AI assistant."

voice:
  enabled: false
  model_path: "model/vosk-model-small-cn-0.22"
  timeout_seconds: 15

ui:
  theme: "light"
  language: "zh-CN"       # switch to "en-US" for English UI

database:
  enabled: true
  path: "database/chat_database.db"
```

### AI Model Modes

- **Ollama**: External Ollama server  
- **Candle**: In-process Rust AI inference

## Multilingual Support

Documentation is available in multiple languages:

- [English (README.md)](README.md)  
- [中文 (README-CN.md)](README-CN.md)

Switch UI language in `config.yaml` under `ui.language`.

## Roadmap

- [ ] Support more LLMs  
- [ ] RISC-V performance tuning  
- [ ] Text-to-speech feature  
- [ ] Improved offline mode  
- [ ] Plugin ecosystem  

See [open issues](https://github.com/horldsence/chat_box_rust/issues) for details.

## Contributing

We welcome your contributions!

1. Fork the project  
2. Create a feature branch (`git checkout -b feature/YourFeature`)  
3. Commit changes (`git commit -m 'Add YourFeature'`)  
4. Push to your branch (`git push origin feature/YourFeature`)  
5. Open a Pull Request

## License

This project is licensed under the MIT License – see the [LICENSE](LICENSE) file for details.

## Contact

Maintainer: Peng – horldjason@outlook.com  
Repository: https://github.com/horldsence/chat_box_rust

## Acknowledgements

- [Tauri](https://tauri.app/)  
- [Vue 3](https://vuejs.org/)  
- [Rust](https://www.rust-lang.org/)  
- [Ollama](https://ollama.com/)  
- [RISC-V Foundation](https://riscv.org/)

<!-- links -->
[your-project-path]: horldsence/chat_box_rust
[contributors-shield]: https://img.shields.io/github/contributors/horldsence/chat_box_rust.svg?style=flat-square
[contributors-url]: https://github.com/horldsence/chat_box_rust/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/horldsence/chat_box_rust.svg?style=flat-square
[forks-url]: https://github.com/horldsence/chat_box_rust/network/members
[stars-shield]: https://img.shields.io/github/stars/horldsence/chat_box_rust.svg?style=flat-square
[stars-url]: https://github.com/horldsence/chat_box_rust/stargazers
[issues-shield]: https://img.shields.io/github/issues/horldsence/chat_box_rust.svg?style=flat-square
[issues-url]: https://github.com/horldsence/chat_box_rust/issues
[license-shield]: https://img.shields.io/github/license/horldsence/chat_box_rust.svg?style=flat-square
[license-url]: https://github.com/horldsence/chat_box_rust/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=flat-square&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/shaojintian