<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
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
import {
  Button,
  ButtonGroup,
} from '@/components/ui/button'
import {
  Input,
} from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'
import { Bot, Send, Plus, Search, Trash2, RotateCcw, MessageSquare, Brain, Cpu, Sparkles } from 'lucide-vue-next'

interface ChatMessage {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
  conversationId: string
}

interface Conversation {
  id: string
  title: string
  lastMessage: string
  timestamp: Date
  model: string
  messageCount: number
}

const input = ref('')
const messages = ref<ChatMessage[]>([])
const conversations = ref<Conversation[]>([])
const currentConversationId = ref<string>('')
const sidebarOpen = ref(false)
const searchQuery = ref('')
const status = ref<'idle' | 'loading' | 'error'>('idle')
const error = ref<string | null>(null)
const selectedModel = ref('gpt-3.5-turbo')
const newChatDialogOpen = ref(false)
const messagesContainer = ref<HTMLElement>()
const autoScrollEnabled = ref(true)

// Model options
const modelOptions = [
  {
    id: 'gpt-4',
    name: 'GPT-4',
    description: 'Most capable model for complex tasks',
    icon: Brain,
    color: 'text-purple-600'
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    description: 'Fast and efficient for most tasks',
    icon: Cpu,
    color: 'text-blue-600'
  },
  {
    id: 'claude-3',
    name: 'Claude 3',
    description: 'Advanced reasoning and analysis',
    icon: Sparkles,
    color: 'text-orange-600'
  }
]

// Filter conversations based on search
const filteredConversations = computed(() => {
  if (!searchQuery.value) return conversations.value
  return conversations.value.filter(conv =>
    conv.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  )
})

// Get current conversation
const currentConversation = computed(() => {
  return conversations.value.find(conv => conv.id === currentConversationId.value)
})

// Auto-scroll to bottom when new messages arrive
const scrollToBottom = async () => {
  if (!autoScrollEnabled.value || !messagesContainer.value) return
  await nextTick()
  messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
}

// Check if user is at bottom
const handleScroll = () => {
  if (!messagesContainer.value) return
  const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value
  autoScrollEnabled.value = scrollHeight - scrollTop <= clientHeight + 100
}

// Initialize with a sample conversation
function initializeConversations() {
  const now = new Date()

  // Create first conversation
  const conv1: Conversation = {
    id: '1',
    title: 'Welcome to Iroh Chat',
    lastMessage: 'Hello! How can I assist you today?',
    timestamp: now,
    model: 'gpt-3.5-turbo',
    messageCount: 2
  }

  // Add another sample conversation
  const conv2: Conversation = {
    id: '2',
    title: 'Project Discussion',
    lastMessage: 'Let me help you with that feature.',
    timestamp: new Date(now.getTime() - 3600000), // 1 hour ago
    model: 'gpt-4',
    messageCount: 5
  }

  conversations.value = [conv1, conv2]
  currentConversationId.value = conv1.id

  // Load messages for current conversation
  loadConversationMessages(conv1.id)
}

// Load messages for a specific conversation
function loadConversationMessages(conversationId: string) {
  currentConversationId.value = conversationId

  if (conversationId === '1') {
    messages.value = [
      {
        id: '1-1',
        role: 'assistant',
        content: 'Hello! I\'m Iroh Chat, your AI assistant. How can I help you today?',
        timestamp: new Date(),
        conversationId: '1'
      }
    ]
  } else if (conversationId === '2') {
    messages.value = [
      {
        id: '2-1',
        role: 'user',
        content: 'Can you help me implement this new feature?',
        timestamp: new Date(Date.now() - 300000),
        conversationId: '2'
      },
      {
        id: '2-2',
        role: 'assistant',
        content: 'I\'d be happy to help you with that feature. Let me break it down into manageable steps and guide you through the implementation process.',
        timestamp: new Date(Date.now() - 180000),
        conversationId: '2'
      }
    ]
  } else {
    messages.value = []
  }

  input.value = ''
  error.value = null
  status.value = 'idle'
}

// Create a new conversation
function createNewConversation() {
  const newId = Date.now().toString()
  const newConversation: Conversation = {
    id: newId,
    title: 'New Conversation',
    lastMessage: '',
    timestamp: new Date(),
    model: selectedModel.value,
    messageCount: 0
  }

  conversations.value.unshift(newConversation)
  currentConversationId.value = newId
  messages.value = []
  input.value = ''
  newChatDialogOpen.value = false

  // Focus on input
  setTimeout(() => {
    const textarea = document.querySelector('textarea')
    if (textarea) textarea.focus()
  }, 100)
}

// Delete conversation
function deleteConversation(conversationId: string, event: Event) {
  event.stopPropagation()

  const index = conversations.value.findIndex(conv => conv.id === conversationId)
  if (index > -1) {
    conversations.value.splice(index, 1)

    // If we deleted the current conversation, switch to another
    if (conversationId === currentConversationId.value) {
      if (conversations.value.length > 0) {
        loadConversationMessages(conversations.value[0].id)
      } else {
        // Create new conversation if none left
        createNewConversation()
      }
    }
  }
}

// Format timestamp
function formatTimestamp(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return 'Just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return date.toLocaleDateString()
}

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
    timestamp: new Date(),
    conversationId: currentConversationId.value
  })

  // Update conversation info
  const convIndex = conversations.value.findIndex(conv => conv.id === currentConversationId.value)
  if (convIndex > -1) {
    conversations.value[convIndex].lastMessage = userMessage
    conversations.value[convIndex].timestamp = new Date()
    conversations.value[convIndex].messageCount += 1
  }

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
      timestamp: new Date(),
      conversationId: currentConversationId.value
    })

    // Update conversation with bot response
    if (convIndex > -1) {
      conversations.value[convIndex].lastMessage = botResponse
      conversations.value[convIndex].timestamp = new Date()
    }

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
    "I'd love to help you explore that idea further.",
    "That's a fascinating topic! Let me share what I know about it.",
  ]
  return responses[Math.floor(Math.random() * responses.length)]
}

// Watch for message changes to trigger auto-scroll
watch(messages, async () => {
  await scrollToBottom()
}, { deep: true })

// Initialize on mount
if (typeof window !== 'undefined') {
  initializeConversations()
}

</script>

<template>
  <div
    class="flex h-screen font-sans bg-gradient-to-br from-indigo-500 to-purple-600"
    :style="{ fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, sans-serif' }"
  >
    <!-- Mobile Menu Button -->
    <Button
      v-if="!sidebarOpen"
      @click="sidebarOpen = true"
      class="lg:hidden fixed top-4 left-4 z-50 bg-white/90 backdrop-blur-lg rounded-full p-2 shadow-lg"
      variant="outline"
    >
      <MessageSquare class="w-5 h-5" />
    </Button>

    <!-- Sidebar -->
    <div
      class="fixed inset-y-0 left-0 z-50 bg-white/95 backdrop-blur-lg border-r border-white/20 shadow-2xl transform transition-transform duration-300 ease-in-out"
      :class="[
        'w-80',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full',
        'lg:translate-x-0'
      ]"
    >
      <!-- Sidebar Header -->
      <div class="p-4 border-b border-white/20">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Bot class="w-8 h-8 text-indigo-500" />
            <h2 class="text-xl font-bold text-gray-900">Conversations</h2>
          </div>
          <Button
            variant="outline"
            size="icon"
            @click="sidebarOpen = false"
            class="lg:hidden"
          >
            <RotateCcw class="w-4 h-4" />
          </Button>
        </div>

        <!-- Search -->
        <div class="mt-4">
          <Input
            v-model="searchQuery"
            placeholder="Search conversations..."
            class="bg-white/80"
          >
            <template #prefix>
              <Search class="w-4 h-4 text-gray-400" />
            </template>
          </Input>
        </div>

        <!-- New Chat Button -->
        <Button
          @click="createNewConversation"
          class="w-full mt-4 bg-indigo-500 hover:bg-indigo-600 text-white"
        >
          <Plus class="w-4 h-4 mr-2" />
          New Chat
        </Button>
      </div>

      <!-- Conversation List -->
      <div class="flex-1 overflow-y-auto p-2">
        <div
          v-for="conversation in filteredConversations"
          :key="conversation.id"
          class="group relative p-3 mb-2 rounded-lg cursor-pointer transition-all hover:bg-gray-100"
          :class="[
            currentConversationId === conversation.id
              ? 'bg-indigo-100 border-l-4 border-indigo-500'
              : 'hover:bg-gray-100'
          ]"
          @click="loadConversationMessages(conversation.id)"
        >
          <!-- Delete Button -->
          <Button
            variant="ghost"
            size="icon"
            @click="deleteConversation(conversation.id, $event)"
            class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <Trash2 class="w-4 h-4 text-red-500 hover:text-red-700" />
          </Button>

          <!-- Conversation Info -->
          <div class="pr-8">
            <div class="font-medium text-gray-900 truncate">
              {{ conversation.title }}
            </div>
            <div class="text-sm text-gray-500 truncate">
              {{ conversation.lastMessage || 'No messages yet' }}
            </div>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-gray-400">
                {{ formatTimestamp(conversation.timestamp) }}
              </span>
              <span class="text-xs bg-gray-200 rounded px-2 py-1">
                {{ conversation.messageCount }} messages
              </span>
              <span
                v-if="conversation.model"
                class="text-xs px-2 py-1 rounded"
                :class="[
                  conversation.model === 'gpt-4' ? 'bg-purple-100 text-purple-700' :
                  conversation.model === 'claude-3' ? 'bg-orange-100 text-orange-700' :
                  'bg-blue-100 text-blue-700'
                ]"
              >
                {{ modelOptions.find(m => m.id === conversation.model)?.name || conversation.model }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Sidebar Footer -->
      <div class="p-4 border-t border-white/20">
        <!-- Model Selector -->
        <div class="mb-4">
          <label class="text-sm font-medium text-gray-700 mb-2">Default Model</label>
          <Select v-model="selectedModel">
            <SelectTrigger class="bg-white/80">
              <SelectValue placeholder="Select model" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="model in modelOptions"
                :key="model.id"
                :value="model.id"
              >
                <div class="flex items-center gap-2">
                  <component :is="model.icon" :class="`w-4 h-4 ${model.color}`" />
                  <div>
                    <div class="font-medium">{{ model.name }}</div>
                    <div class="text-xs text-gray-500">{{ model.description }}</div>
                  </div>
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- Footer Info -->
        <div class="text-center">
          <p class="text-xs text-gray-500">
            Current: {{ filteredConversations.length }} conversations
          </p>
        </div>
      </div>
    </div>

    <!-- Mobile Overlay -->
    <div
      v-if="sidebarOpen"
      class="lg:hidden fixed inset-0 z-40 bg-black/50"
      @click="sidebarOpen = false"
    ></div>

    <!-- Main Content -->
    <div class="flex flex-col h-full lg:ml-80">
      <!-- Header -->
      <header class="bg-white/95 backdrop-blur-lg border-b border-white/20 shadow-lg px-4 lg:px-8 py-4">
        <div class="max-w-screen-2xl mx-auto">
          <div class="flex items-center gap-4">
            <Button
              variant="ghost"
              @click="sidebarOpen = true"
              class="lg:hidden"
            >
              <MessageSquare class="w-5 h-5" />
            </Button>

            <Bot class="w-8 h-8 text-indigo-500" />
            <div class="flex-1">
              <h1 class="text-2xl lg:text-3xl font-bold text-gray-900 m-0">
                {{ currentConversation?.title || 'Chat' }}
              </h1>
              <p class="text-gray-500 text-sm hidden lg:block">
                {{ currentConversation?.model ? `Powered by ${modelOptions.find(m => m.id === currentConversation.model)?.name}` : 'Select a model' }}
              </p>
            </div>

            <!-- Status Indicator -->
            <div
              class="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium transition-colors"
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
              <span class="hidden sm:inline">{{ status === 'error' ? 'Error' : 'Online' }}</span>
            </div>
          </div>
        </div>
      </header>

      <!-- Chat Area -->
      <div class="flex-1 overflow-hidden">
        <div class="h-full flex flex-col">
          <!-- Messages Container -->
          <div
            ref="messagesContainer"
            class="flex-1 overflow-y-auto px-4 py-4"
            @scroll="handleScroll"
          >
            <div class="max-w-4xl mx-auto">
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
                    class="max-w-[90%] mb-6"
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
          </div>
        </div>
      </div>

      <!-- Input Area -->
      <div class="bg-white/95 backdrop-blur-lg border-t border-white/20 px-4 lg:px-8 py-4">
        <div class="max-w-screen-2xl mx-auto">
          <form @submit="handleSubmit">
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
      </div>

      <!-- Footer -->
      <footer class="bg-gray-900/95 backdrop-blur-lg border-t border-white/10 px-4 lg:px-8 py-4">
        <div class="max-w-screen-2xl mx-auto text-center">
          <p class="text-gray-400 text-sm m-0">
            Powered by AI Elements Vue • Built with Tauri + Vue 3
          </p>
        </div>
      </footer>
    </div>

    <!-- New Chat Dialog -->
    <Dialog v-model:open="newChatDialogOpen">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New Conversation</DialogTitle>
          <DialogDescription>
            Choose a model for your new conversation. This will create a fresh chat session.
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium text-gray-700">Select Model</label>
            <Select v-model="selectedModel">
              <SelectTrigger>
                <SelectValue placeholder="Choose a model" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="model in modelOptions"
                  :key="model.id"
                  :value="model.id"
                >
                  <div class="flex items-center gap-3">
                    <component :is="model.icon" :class="`w-5 h-5 ${model.color}`" />
                    <div>
                      <div class="font-medium">{{ model.name }}</div>
                      <div class="text-sm text-gray-500">{{ model.description }}</div>
                    </div>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            @click="newChatDialogOpen = false"
          >
            Cancel
          </Button>
          <Button @click="createNewConversation">
            Create Chat
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
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