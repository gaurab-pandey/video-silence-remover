<div align="center">

  <img src="public/img/1.png" alt="Video Silence Remover Logo" width="120" height="120" />

  # Video Silence Remover

  **Automatically detect and remove silence from your videos with native performance.**

  [![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
  [![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
  [![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)](https://react.js.org/)
  [![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
  [![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
  [![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](LICENSE)

  <p align="center">
    <a href="#features">✨ Features</a> •
    <a href="#installation">⬇️ Installation</a> •
    <a href="#development-setup">🛠️ Development</a> •
    <a href="#contributing">🤝 Contributing</a>
  </p>
</div>

---

## ✨ Features

- **🔇 Smart Silence Detection**  
  Automatically analyzes video audio to detect silent segments with precision.

- **🎞️ Interactive Timeline**  
  Visual waveform representation with a professional Premiere-style timeline for easy navigation.

- **👁️ Segment Review**  
  Fine-tune the results by reviewing, merging, or excluding detected segments before exporting.

- **⚡ Instant Preview**  
  Real-time video preview as you scrub through the timeline—no rendering required.

- **🚀 Native Performance**  
  Powered by **Rust** and **FFmpeg** for blazing fast processing and rendering.

---

## ⬇️ Installation

The application is distributed as a **standalone executable**. 

> [!IMPORTANT]  
> You do **not** need Rust or Node.js installed to run the app!

1. Download the latest `.exe` or `.msi` from the **[Releases Page](#)**.
2. Run the installer.
3. Enjoy!

---

## 🛠️ Development Setup

If you want to build the application from source or contribute code, follow these steps.

### Prerequisites (For Development Only)
- **Node.js**: [Download Node.js](https://nodejs.org/)
- **Rust**: [Install Rust](https://rustup.rs/)

### 🏗️ Build from Source

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/video-silence-remover.git
   cd video-silence-remover
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **FFmpeg Setup**  
   ⚠️ Ensure `ffmpeg` and `ffprobe` binaries are placed in `src-tauri/binaries` with the correct architecture suffix (e.g., `ffmpeg-x86_64-pc-windows-msvc.exe`).

### 🏃 Run Locally

Start the development server with hot-reload:
```bash
npm run tauri dev
```

Build for production:
```bash
npm run tauri build
```

---

## 🤝 Contributing

We welcome contributions from the community! 

1. **🍴 Fork** the project.
2. **🌿 Create** a feature branch (`git checkout -b feature/AmazingFeature`).
3. **💻 Commit** your changes (`git commit -m 'Add some AmazingFeature'`).
4. **🚀 Push** to the branch (`git push origin feature/AmazingFeature`).
5. **📥 Open** a Pull Request.

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

<div align="center">
  <sub>Built with ❤️ by <a href="https://x.com/snc0x">snc0x</a></sub>
</div>
