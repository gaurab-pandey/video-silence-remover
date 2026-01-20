<div align="center">

  <img src="public/img/1.png" alt="Video Silence Remover Logo" width="120" height="120" />

  # Video Silence Remover

  **A high-performance, native desktop application to automatically detect and eliminate silent gaps from video footage.**

  [![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
  [![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
  [![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)](https://react.js.org/)
  [![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
  [![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
  [![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

  <p align="center">
    <a href="#-overview">📖 Overview</a> •
    <a href="#-features">✨ Features</a> •
    <a href="#-installation">⬇️ Installation</a> •
    <a href="#-development-setup">🛠️ Development</a> •
    <a href="#-contributing">🤝 Contributing</a>
  </p>
</div>

---

## 📖 Overview

Video Silence Remover is designed for content creators, podcasters, and video editors who want to speed up their workflow. By leveraging the power of **Rust** for core processing and **FFmpeg** for media analysis, it identifies silent segments and allows for surgical removal, leaving you with a clean, punchy final cut.

Unlike web-based tools, this is a **native application** that runs locally on your machine—ensuring maximum privacy, speed, and support for large video files.

---

## ✨ Features

- **🔇 Intelligent Silence Detection**  
  Scan your videos for silent intervals using customizable threshold and duration parameters. Perfect for jump-cut style editing.

- **🎞️ Professional Timeline Workflow**  
  A fully interactive, Premiere-style timeline. Visualize the audio waveform, scrub the playhead, and see exactly where the cuts will happen.

- **👁️ Surgical Segment Review**  
  Choose exactly what to keep and what to cut. Review detected segments, merge them, or exclude specific parts before the final export.

- **⚡ Blazing Fast Preview**  
  Real-time preview mechanism. Scrub through the timeline and see the video frame update instantly without pre-rendering.

- **🚀 Native Performance & Privacy**  
  Processing happens entirely on your machine. No cloud uploads, no subscription fees, just raw native speed.

---

## ⬇️ Installation

The application is distributed as a portable standalone executable or a standard installer.

> [!IMPORTANT]  
> End-users do **not** need to install Rust, Node.js, or FFmpeg manually. Everything is bundled!

1. Head over to the **[Releases Page](https://github.com/dietcokezerosugar/video-silence-remover/releases)**.
2. Download the latest `.msi` (installer) or `.exe` (portable).
3. Run the application and start editing.

---

## 🛠️ Development Setup

Follow these steps if you want to contribute to the project or build a custom version.

### Prerequisites
- **Node.js** (v18+)
- **Rust** (Stable)
- **Git LFS** (Required to fetch bundled FFmpeg binaries)

### 🏗️ Build from Source

1. **Clone the repository and fetch LFS objects**
   ```bash
   git clone https://github.com/dietcokezerosugar/video-silence-remover.git
   cd video-silence-remover
   git lfs install
   git lfs pull
   ```

2. **Install frontend dependencies**
   ```bash
   npm install
   ```

3. **FFmpeg Sidecars**  
   The project uses sidecar binaries for FFmpeg. If `git lfs pull` was successful, you should see them in `src-tauri/binaries/`. If you are adding new binaries, ensure they follow the Tauri sidecar naming convention (e.g., `ffmpeg-x86_64-pc-windows-msvc.exe`).

### 🏃 Running the App

Start the development environment:
```bash
npm run tauri dev
```

Build a production release:
```bash
npm run tauri build
```

---

## 🤝 Contributing

Contributions are what make the open-source community an amazing place to learn, inspire, and create.

1. **Fork** the Project.
2. Create your **Feature Branch** (`git checkout -b feature/AmazingFeature`).
3. **Commit** your Changes (`git commit -m 'Add some AmazingFeature'`).
4. **Push** to the Branch (`git push origin feature/AmazingFeature`).
5. Open a **Pull Request**.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

<div align="center">
  <sub>Built with ❤️ by <a href="https://x.com/snc0x">snc0x</a></sub>
</div>
