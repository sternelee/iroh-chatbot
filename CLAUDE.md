# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **iroh-chat**, a hybrid desktop application built with Tauri (Rust backend) and Vue 3 (TypeScript frontend). The project includes both a desktop app and a separate Axum web server component.

## Architecture

The project consists of three main components:

### 1. Tauri Desktop App (`src-tauri/`)
- **Backend**: Rust library (`iroh_chat_lib`) in `src-tauri/src/`
- **Frontend**: Vue 3 + TypeScript application in the root `src/`
- **Entry point**: `src-tauri/src/main.rs` calls `iroh_chat_lib::run()`

### 2. Vue Frontend (`src/`)
- **Framework**: Vue 3 with Composition API and `<script setup>`
- **Styling**: Tailwind CSS v4 with shadcn/ui components
- **Build tool**: Vite with Vue plugin
- **TypeScript**: Strict mode enabled with path aliases (`@/` → `src/`)

### 3. Axum Web Server (`axum-app/`)
- **Framework**: Axum web framework for Rust
- **Async runtime**: Tokio
- **Structure**: Separate binary (`main.rs`) and library (`lib.rs`)

## Common Development Commands

### Frontend Development
```bash
# Start development server
npm run dev

# Build for production
npm run build

# Type checking (fails on errors)
vue-tsc --noEmit

# Preview production build
npm run preview
```

### Tauri Development
```bash
# Run Tauri development mode
npm run tauri dev

# Build Tauri application
npm run tauri build
```

### Rust Components
```bash
# Build Tauri backend
cd src-tauri && cargo build

# Build Axum app
cd axum-app && cargo build

# Run Axum server
cd axum-app && cargo run
```

## Key Configuration

- **Vite config**: Port 1420, HMR on port 1421, ignores `src-tauri/` for watching
- **TypeScript**: Strict mode enabled, unused locals/parameters checking
- **Tauri development**: Uses `TAURI_DEV_HOST` environment variable for host configuration
- **Path aliases**: `@/` mapped to `src/` for imports

## Development Setup

The project uses:
- **Node.js**: Package management with npm/bun
- **Rust**: Cargo workspace with members `axum-app` and `src-tauri`
- **IDE**: Recommended VS Code with Vue - Official, Tauri, and rust-analyzer extensions

## Component Structure

Frontend components follow shadcn/ui patterns with:
- **Components**: `@/components` (including `@/components/ui`)
- **Utilities**: `@/lib/utils.ts` with `cn()` function for class merging
- **Composables**: `@/composables` for Vue composition functions
- **Assets**: Static assets in `src/assets/`

The application currently contains a basic Vue template with Tauri integration for demonstrating the framework setup.