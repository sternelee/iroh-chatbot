# Iroh Chat - AI Chatbot Application

A modern chatbot application built with Tauri, Vue 3, and AI Elements Vue. This application demonstrates how to integrate AI-powered chat interfaces into desktop applications.

## Features

- **Modern Chat Interface**: Built with AI Elements Vue components for a professional chat experience
- **Real-time Communication**: Responsive messaging with typing indicators
- **Beautiful UI**: Gradient backgrounds, glass morphism effects, and smooth animations
- **Responsive Design**: Works seamlessly on desktop and mobile devices
- **TypeScript**: Full type safety throughout the application
- **Cross-platform**: Runs on Windows, macOS, and Linux via Tauri

## Technology Stack

### Frontend
- **Vue 3** - Progressive JavaScript framework with Composition API
- **TypeScript** - Type-safe JavaScript
- **AI Elements Vue** - Pre-built AI chat components
- **Tailwind CSS v4** - Utility-first CSS framework
- **shadcn-vue** - High-quality Vue component library
- **Vite** - Fast build tool and development server

### Backend (Tauri)
- **Rust** - Systems programming language for the desktop app backend
- **Tauri** - Framework for building cross-platform desktop applications

### AI Integration
- **AI SDK** - For seamless AI service integration
- **vue-stick-to-bottom** - Auto-scrolling chat functionality

## Project Structure

```
iroh-chat/
├── src/
│   ├── components/
│   │   ├── ai-elements/     # AI Elements Vue components
│   │   └── ui/              # shadcn-vue components
│   ├── lib/                 # Utility functions
│   ├── api/                 # API endpoints (for demonstration)
│   ├── App.vue              # Main chatbot interface
│   └── main.ts              # Vue app entry point
├── src-tauri/               # Tauri Rust backend
├── axum-app/                # Optional web server component
└── public/                  # Static assets
```

## Getting Started

### Prerequisites
- Node.js (v18 or higher)
- Rust and Cargo
- Tauri CLI

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd iroh-chat
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Add AI Elements Vue components**
   ```bash
   npx ai-elements-vue@latest add conversation
   npx ai-elements-vue@latest add message
   npx ai-elements-vue@latest add prompt-input
   ```

### Development

1. **Start the development server**
   ```bash
   npm run dev
   ```

2. **Run Tauri in development mode**
   ```bash
   npm run tauri dev
   ```

### Production Build

1. **Build the web application**
   ```bash
   npm run build
   ```

2. **Build the Tauri desktop application**
   ```bash
   npm run tauri build
   ```

## Key Components

### Conversation Components
- `Conversation` - Main chat container with auto-scroll
- `ConversationContent` - Message layout and spacing
- `ConversationEmptyState` - Empty chat state display
- `ConversationScrollButton` - Scroll-to-bottom button

### Message Components
- `Message` - Individual message wrapper
- `MessageContent` - Message content container
- `MessageResponse` - Response message display
- `MessageAvatar` - User/assistant avatars

### Input Components
- `PromptInput` - Chat input container
- `PromptInputTextarea` - Multi-line text input
- `PromptInputSubmit` - Send button with loading states

## Customization

### Styling
The chatbot uses a modern design with:
- Gradient backgrounds
- Glass morphism effects
- Smooth animations
- Responsive layouts

You can customize the appearance by modifying the CSS in `App.vue`.

### AI Integration
Currently uses a simple mock API. To integrate with real AI services:

1. Replace the mock responses in `getBotResponse()`
2. Set up proper API endpoints
3. Configure authentication keys
4. Handle streaming responses

Example with OpenAI:
```typescript
import { OpenAI } from 'openai'

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
})

async function getAIResponse(message: string) {
  const completion = await openai.chat.completions.create({
    messages: [{ role: "user", content: message }],
    model: "gpt-3.5-turbo",
  })

  return completion.choices[0]?.message?.content || "I'm sorry, I couldn't process that."
}
```

## Features Overview

### Chat Interface
- Clean, modern messaging interface
- Support for user and assistant messages
- Typing indicators and loading states
- Empty state with helpful messaging
- Responsive message layout

### Input Handling
- Multi-line text input with auto-resize
- Send button with loading states
- Keyboard shortcuts (Enter to send)
- Disabled state during processing

### Status Management
- Real-time connection status
- Error handling and display
- Loading states for better UX

### Responsive Design
- Mobile-friendly interface
- Adaptive layouts
- Touch-friendly interactions

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License.

## Acknowledgments

- [AI Elements Vue](https://github.com/vuepont/ai-elements-vue) - Vue port of AI Elements
- [Tauri](https://tauri.app/) - Cross-platform desktop app framework
- [Vue 3](https://vuejs.org/) - Progressive JavaScript framework
- [shadcn-vue](https://www.shadcn-vue.com/) - Vue component library