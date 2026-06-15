<div align="center">
  <h1>🧠 Vimtelligence</h1>
  <p><strong>An ultra-high-performance modal terminal text editor with local AI acceleration.</strong></p>
  
  [![npm version](https://badge.fury.io/js/vimtelligence.svg)](https://badge.fury.io/js/vimtelligence)
  [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
</div>

---

**Vimtelligence** is a next-generation terminal text editor built in pure Rust. It marries the lightning-fast, modal editing paradigms of Vim with a locally hosted, offline-first Artificial Intelligence backend. No cloud APIs. No lag.

By embedding a quantized **Gemma 3 (270M parameters)** model natively via the [Candle](https://github.com/huggingface/candle) ML framework, Vimtelligence understands your natural language instructions and translates them instantly into structural Vim commands right within your editor.

## ✨ Features

- **Blazing Fast**: Written in pure Rust using `ratatui` for immediate-mode rendering and `ropey` for heavy-duty B-tree text buffer management.
- **True Modal Editing**: Supports classic Vim motions and structural operations like `ciw` (change inside word), `daw` (delete around word), and `dd`.
- **EZ Mode**: Press `Ctrl+W` to open the AI command overlay. Type natural language (e.g., *"delete this word"*), and the background AI thread will compute and seamlessly execute the correct macro instantly.
- **Offline & Private**: Inference runs 100% locally on your CPU/GPU using the quantized GGUF model. Your code never leaves your machine.
- **Asynchronous Engine**: Built on `tokio`, the editor's UI remains fully responsive at 60 FPS while the AI generates macros in the background.

## 🚀 Installation

You can install Vimtelligence globally via NPM. The postinstall script will automatically download the compiled Rust executable for your platform and the quantized AI model.

```bash
npm install -g vimtelligence
```

*Note: Currently, pre-compiled binaries are provided for Windows (`x86_64-pc-windows-msvc`). For other platforms, you will need to build from source.*

## 🛠️ Usage

Simply run Vimtelligence from your terminal, passing the file you wish to edit:

```bash
vimtelligence src/main.rs
```

### Core Keybindings
- `i` - Enter **Insert Mode**.
- `Esc` - Return to **Normal Mode**.
- `h`, `j`, `k`, `l` - Navigate cursor (Normal Mode).
- `s` - Save the current file.
- `q` - Quit the editor.

### EZ Mode (AI Assistant)
1. In Normal Mode, press `Ctrl+W` to open the AI ribbon.
2. Type a natural language command like *"delete the current line"*.
3. Press `Enter`. The model will execute the command dynamically and return you to Normal Mode.

## 🏗️ Building from Source

To build Vimtelligence locally, ensure you have [Rust](https://rustup.rs/) installed.

```bash
git clone https://github.com/turtle170/Vimtelligence.git
cd Vimtelligence
cargo run --release -- example.txt
```

To enable the AI capabilities locally, you will need to download the model into the expected directory:
```bash
node install.js
```

## 📜 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
