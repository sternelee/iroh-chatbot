<script setup lang="ts">
import { ref } from "vue";
import {
  Conversation,
  ConversationContent,
  ConversationScrollButton,
  ConversationEmptyState,
} from '@/components/ai-elements/conversation'
import {
  Message,
  MessageContent,
  MessageResponse,
} from '@/components/ai-elements/message'
import {
  PromptInput,
  PromptInputSubmit,
  PromptInputTextarea,
} from '@/components/ai-elements/prompt-input'
import { Bot } from 'lucide-vue-next'

interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
}

const input = ref('')
const messages = ref<ChatMessage[]>([])
const status = ref<'idle' | 'loading' | 'error'>('idle')
const error = ref<string | null>(null)

async function handleSubmit(e: Event) {
  e.preventDefault()
  if (!input.value.trim() || status.value === 'loading') return

  const userMessage = input.value.trim()
  input.value = ''

  // Add user message
  messages.value.push({
    id: Date.now().toString(),
    role: 'user',
    content: userMessage,
    timestamp: new Date()
  })

  // Set loading state
  status.value = 'loading'
  error.value = null

  try {
    // Simulate bot response (replace with actual API call)
    await new Promise(resolve => setTimeout(resolve, 1000))

    const botResponse = getBotResponse(userMessage)
    messages.value.push({
      id: (Date.now() + 1).toString(),
      role: 'assistant',
      content: botResponse,
      timestamp: new Date()
    })

    status.value = 'idle'
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'An error occurred'
    status.value = 'error'
  }
}

function getBotResponse(_userMessage: string): string {
  const responses = [
    "That's interesting! Tell me more about that.",
    "I understand. How can I help you with that?",
    "Thanks for sharing! What else would you like to discuss?",
    "I see your point. Let me think about that for a moment.",
    "That's a great question! Here's what I think about it.",
    "I appreciate you sharing that with me.",
    "That makes sense. What are your thoughts on this?",
    "Interesting perspective! Have you considered other angles?",
  ]
  return responses[Math.floor(Math.random() * responses.length)]
}
</script>

<template>
  <div
    class="flex flex-col h-screen font-sans bg-gradient-to-br from-indigo-500 to-purple-600"
    :style="{ fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, sans-serif' }"
  >
    <!-- Header -->
    <header class="bg-white/95 backdrop-blur-lg border-b border-white/20 shadow-lg px-8 py-4">
      <div class="max-w-screen-2xl mx-auto">
        <div class="flex items-center gap-4">
          <Bot class="w-10 h-10 text-indigo-500" />
          <h1 class="text-3xl font-bold text-gray-900 m-0">Iroh Chat</h1>
          <div
            class="ml-auto flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium transition-colors"
            :class="[
              status !== 'error'
                ? 'bg-green-100 text-green-800'
                : 'bg-red-100 text-red-800'
            ]"
          >
            <span
              class="w-2 h-2 rounded-full animate-pulse"
              :class="[
                status !== 'error' ? 'bg-green-500' : 'bg-red-500'
              ]"
            ></span>
            <span>{{ status === 'error' ? 'Offline' : 'Online' }}</span>
          </div>
        </div>
        <p class="text-gray-500 text-sm mt-1 ml-14">Your AI-powered chat assistant</p>
      </div>
    </header>

    <!-- Chat Area -->
    <div class="flex-1 overflow-hidden m-4 bg-white/95 backdrop-blur-lg rounded-2xl shadow-2xl">
      <Conversation class="h-full">
        <ConversationContent>
          <ConversationEmptyState
            v-if="messages.length === 0"
            title="Start a conversation"
            description="Type a message below to begin chatting with your AI assistant"
          >
            <Bot class="w-16 h-16 text-gray-400" />
          </ConversationEmptyState>

          <Message
            v-for="message in messages"
            :key="message.id"
            :from="message.role"
            class="max-w-[90%]"
          >
            <MessageContent>
              <MessageResponse>
                {{ message.content }}
              </MessageResponse>
            </MessageContent>
          </Message>
        </ConversationContent>

        <ConversationScrollButton />
      </Conversation>
    </div>

    <!-- Input Area -->
    <div class="bg-white/95 backdrop-blur-lg border-t border-white/20 px-8 py-4">
      <form @submit="handleSubmit" class="max-w-screen-2xl mx-auto">
        <PromptInput class="relative bg-white rounded-2xl border-2 border-gray-200 transition-all focus-within:border-indigo-500 focus-within:shadow-lg focus-within:shadow-indigo-500/10">
          <PromptInputTextarea
            v-model="input"
            placeholder="Type your message here..."
            class="px-6 pr-16 py-4 border-0 resize-none text-base leading-6 max-h-32 focus:ring-0"
            :disabled="status === 'loading'"
            @keydown.enter.prevent="handleSubmit"
          />

          <PromptInputSubmit
            :status="status === 'loading' ? 'streaming' : 'ready'"
            :disabled="!input.trim() || status === 'loading'"
            class="absolute bottom-3 right-3"
            @click="handleSubmit"
          />
        </PromptInput>
      </form>

      <!-- Error Display -->
      <div
        v-if="error"
        class="max-w-screen-2xl mx-auto mt-2 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm"
      >
        <p class="m-0">Error: {{ error }}</p>
      </div>
    </div>

    <!-- Footer -->
    <footer class="bg-gray-900/95 backdrop-blur-lg border-t border-white/10 px-8 py-4">
      <div class="max-w-screen-2xl mx-auto text-center">
        <p class="text-gray-400 text-sm m-0">
          Powered by AI Elements Vue • Built with Tauri + Vue 3
        </p>
      </div>
    </footer>
  </div>
</template>

<style scoped>
/* 移除自定义 CSS，全部使用 Tailwind CSS */
</style>
<style>
/* 全局样式重置 - 利用 Tailwind 的基础层 */
html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}

#app {
  height: 100%;
}

/* 确保字体渲染优化 */
:root {
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}
</style>