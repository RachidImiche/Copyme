<div align="center">
  <img src="app-icon.svg" width="128" height="128" alt="CopyMe Logo">
  <h1>CopyMe</h1>
  <p>A lightning-fast, highly aesthetic clipboard history manager built with Tauri 2.</p>

  <br>

  <a href="https://github.com/RachidImiche/Copyme/releases/latest">
    <img src="https://img.shields.io/badge/Windows-Download_Latest-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Download for Windows" />
  </a>
  <a href="https://github.com/RachidImiche/Copyme/releases/latest">
    <img src="https://img.shields.io/badge/macOS-Download_Latest-000000?style=for-the-badge&logo=apple&logoColor=white" alt="Download for macOS" />
  </a>
  <a href="https://github.com/RachidImiche/Copyme/releases/latest">
    <img src="https://img.shields.io/badge/Linux-Download_Latest-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Download for Linux" />
  </a>

  <br><br>
</div>

---

## Overview

[![Build Status](https://img.shields.io/github/actions/workflow/status/RachidImiche/Copyme/release.yml?style=for-the-badge)](https://github.com/RachidImiche/Copyme/actions/workflows/release.yml)

**CopyMe** is a lightweight, cross-platform clipboard history application designed to be a modern, drop-in replacement for native clipboard managers. Built with **Tauri 2**, **Rust**, and **Vanilla web technologies**, it offers a blazing-fast background listener, local SQLite persistence, and a nice UI.

## Features

- **Global Shortcuts**: Instantly summon the clipboard anywhere using `Win + V` or `Ctrl + Shift + V`.
- **Always-on-Top Floating UI**: A non-intrusive panel that appears right at your mouse cursor.
- **Smart Background Monitoring**: Silently monitors clipboard changes, ignores duplicates, and safely limits history to 500 items.
- **Local Persistence**: All your history and pinned items are stored safely on your machine using a local SQLite database.
- **Instant Paste Simulation**: Click any item (or use keyboard navigation) to instantly paste the text directly into your previously focused application.
- **Search & Pin**: Quickly filter through your history in real-time, and pin your most-used snippets so they never get deleted.
- **Full Text Preview**: Easily preview massive blocks of copied text.

## Technology Stack

- **Framework**: [Tauri 2](https://v2.tauri.app/) (Minimal footprint, secure, native integration)
- **Backend**: Rust
  - `rusqlite`: Database management and persistence
  - `arboard`: Cross-platform background clipboard access
  - `enigo`: Native keyboard simulation for seamless pasting
  - `tauri-plugin-global-shortcut`: System-wide hotkey registration
- **Frontend**: Vanilla HTML, CSS (Custom CSS Variables & Backdrop Filters), and JavaScript

## Installation & Build

### Prerequisites
Make sure you have installed the necessary dependencies for Tauri development:
- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- Standard [Tauri Prerequisites for Windows](https://v2.tauri.app/start/prerequisites/) (Visual Studio C++ Build Tools).

### Running Locally
1. Clone the repository:
   ```bash
   git clone https://github.com/RachidImiche/Copyme.git
   cd Copyme
   ```
2. Install Node dependencies:
   ```bash
   npm install
   ```
3. Run the development server:
   ```bash
   npm run tauri dev
   ```

### Building for Production
To create a standalone executable and an `.msi` / `.exe` installer:
```bash
npm run tauri build
```
The generated installers will be located in `src-tauri/target/release/bundle/`.


## 📄 License
This project is licensed under the MIT License.
