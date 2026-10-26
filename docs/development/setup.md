# Windows development setup

## Prerequisites

Install Node.js LTS, Rust via rustup with the stable MSVC toolchain, Visual Studio Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2. Tauri's official prerequisites documentation has current installation details.

```powershell
winget install --id Rustlang.Rustup
rustup default stable-msvc
npm ci
```

## Commands

```powershell
npm run tauri:dev      # start the Windows app
npm run check          # Prettier, ESLint, TypeScript, frontend + adapter tests
npm run reference:test-native # offline adapter/native-validator integration
npm run rust:fmt       # check Rust formatting
npm run rust:check     # native type/build check
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri:build    # create a local NSIS installer
```

No release publishing or signing is configured. `npm ci` requires the committed lockfile.
