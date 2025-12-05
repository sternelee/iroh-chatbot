# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **iroh-chat**, a hybrid desktop application built with Tauri (Rust backend) and Vue 3 (TypeScript frontend). The project includes both a desktop app and a separate Axum web server component.

## Architecture

The project uses an integrated hybrid architecture with three main components:

### 1. Tauri Desktop App (`src-tauri/`)
- **Backend**: Rust library (`iroh_chat_lib`) with embedded Axum server
- **Frontend**: Vue 3 + TypeScript application in the root `src/`
- **Integration**: Uses `tauri-axum` for seamless Tauri-Axum integration
- **Entry point**: `src-tauri/src/main.rs` calls `iroh_chat_lib::run()`
- **State Management**: Shared app state with async router handling via `Arc<Mutex<Router>>`

### 2. Vue Frontend (`src/`)
- **Framework**: Vue 3 with Composition API and `<script setup>`
- **Styling**: Tailwind CSS v4 with shadcn/ui components
- **Build tool**: Vite with Vue plugin
- **TypeScript**: Strict mode enabled with path aliases (`@/` → `src/`)

### 3. Axum Web Server (`axum-app/`)
- **Framework**: Axum web framework for Rust
- **Async runtime**: Tokio
- **Structure**: Library (`lib.rs`) used by Tauri, standalone binary (`main.rs`) for development
- **Integration**: Exported as crate dependency to Tauri backend

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
# Build entire workspace (Tauri + Axum)
cargo build

# Build Tauri backend specifically
cd src-tauri && cargo build

# Build Axum app specifically
cd axum-app && cargo build

# Run standalone Axum server (for development/testing)
cd axum-app && cargo run

# Run integrated Tauri app with embedded Axum
npm run tauri dev
```

### Code Quality
```bash
# Lint and fix code
npm run lint

# Check linting without fixing
npm run lint:check

# Format code with Prettier
npm run format

# Check formatting
npm run format:check

# Run both linting and formatting
npm run lint:format

# Type check only
npm run type-check
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

## Application Features

### Core Chat Interface
- **AI Chat Interface**: Full-featured chat with conversation management, message history, and real-time responses
- **Multiple AI Models**: Support for GPT-4, GPT-3.5 Turbo, Claude 3, and other models with model-specific badges
- **Message Enhancements**: Like/dislike feedback, copy functionality, response regeneration, and message branching
- **File Attachments**: Support for file uploads and attachments in conversations

### User Interface
- **Sidebar Navigation**: Collapsible sidebar with conversation list, search functionality, and settings
- **Responsive Design**: Mobile-friendly interface with collapsible sidebars and adaptive layouts
- **Dark/Light Mode**: Theme switching with persistent settings
- **Advanced Components**: Uses shadcn/ui components including Cards, Badges, Tabs, Accordion, Switch, and ScrollArea

### Input System
- **Cursor-Style Input**: Advanced prompt input with file attachment support, command palette, and tabs
- **Suggestion Cards**: Interactive suggestion cards for common use cases (code writing, explanations, brainstorming)
- **Model Selection**: Dynamic model switching with detailed model information and capabilities

### Settings & Configuration
- **Comprehensive Settings**: Tabbed settings interface with General, AI Models, and Appearance sections
- **User Profile**: Customizable user profile with avatar, name, and email
- **Preferences**: Notification settings, sound effects, auto-save conversations

## Component Structure

Frontend components follow a sophisticated architecture:

### Core Components (`@/components/`)
- **AI Elements**: Advanced chat components (`@/components/ai-elements/`)
  - `conversation/`: Conversation management, content display, scroll controls
  - `message/`: Message rendering, actions, attachments, branching, toolbar
  - `prompt-input/`: Advanced input system with file handling, commands, tabs
- **UI Components**: Extensive shadcn/ui component library (`@/components/ui/`)
  - Standard components: Button, Input, Avatar, Dialog, Dropdown Menu, Tooltip
  - Advanced components: Sidebar, Card, Badge, Tabs, Accordion, Switch, ScrollArea

### Key Architectural Patterns
- **AI SDK Compatibility**: Message structures compatible with AI SDK standards
- **Composition API**: Extensive use of Vue 3 Composition API with TypeScript
- **Reactive State Management**: Complex reactive state for conversations, messages, and UI
- **Component Composition**: Highly composable component architecture with props and events

### API Integration
- **Mock Chat API**: `src/api/chat.ts` with POST endpoint for AI responses
- **Response Simulation**: Realistic response timing and content variation
- **Error Handling**: Comprehensive error handling and user feedback

## Tauri-Axum Integration

### Architecture Pattern
- **Shared Router**: Axum router is created in `axum-app/lib.rs` and embedded in Tauri app state
- **Async Bridge**: Uses `tauri-axum` crate to bridge Tauri frontend to Axum backend
- **Request Handling**: `local_app_request` command forwards frontend requests to embedded Axum router
- **State Management**: `Arc<Mutex<Router>>` provides thread-safe access to the Axum router

### Development Workflow
- **Standalone Mode**: Run Axum server separately (`cd axum-app && cargo run`) for API development
- **Integrated Mode**: Run Tauri app (`npm run tauri dev`) for full desktop experience
- **Shared Code**: Axum routes and handlers are shared between standalone and integrated modes
- **Hot Reload**: Both frontend (Vite) and backend (cargo watch) support hot reloading in development

### Communication Flow
1. Frontend (Vue) makes HTTP requests to local endpoints
2. Tauri captures requests via `local_app_request` command
3. Requests are forwarded to embedded Axum router
4. Axum processes requests and returns responses
5. Responses flow back through Tauri to the frontend

## Data Structures

### Core Types
- **ChatMessage**: AI SDK compatible message structure with id, role, content, createdAt, attachments, metadata
- **ConversationData**: Complete conversation structure with messages array, timestamps, model info
- **EnhancedMessage**: Advanced message format with versioning, attachments, and branching support
- **PromptInputMessage**: Input format for the advanced prompt system with text and files

### State Management
- **Conversations**: Reactive array of conversation data with search and filtering
- **Messages**: Separate reactive arrays for current messages and enhanced messages
- **UI State**: Reactive state for dark mode, loading status, error handling, settings
- **User Preferences**: Settings for notifications, sound effects, auto-save, theme selection

## Development Patterns

### Vue 3 Patterns
- **Composition API**: All components use `<script setup>` with TypeScript
- **Reactive References**: Extensive use of `ref()` and `computed()` for state management
- **Watchers**: Strategic use of `watch()` for side effects and auto-scrolling
- **Component Composition**: Heavy use of props, events, and slots for component communication

### Styling Approach
- **Tailwind CSS**: Comprehensive utility-first styling with custom configurations
- **shadcn/ui**: Professional component library with consistent design system
- **Responsive Design**: Mobile-first approach with breakpoint-specific layouts
- **Theme Support**: Dynamic dark/light mode with CSS custom properties

### TypeScript Usage
- **Strict Mode**: Full TypeScript strict mode with comprehensive type checking
- **Interface Definitions**: Detailed interfaces for all data structures and API responses
- **Type Safety**: Strong typing throughout with proper generic usage
- **Import Aliases**: Path aliases (`@/`) for clean import statements

The application is a production-ready AI chat interface with advanced features comparable to modern AI chat platforms.