<script setup lang="ts">
  import { ref, computed, watch, nextTick, defineComponent } from 'vue'
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
    MessageAvatar,
    MessageActions,
    MessageAction
  } from '@/components/ai-elements/message'
  import {
    PromptInput,
    PromptInputSubmit,
    PromptInputTextarea,
    PromptInputAttachments,
    PromptInputActionMenu,
    PromptInputActionMenuTrigger,
    PromptInputActionMenuContent,
    PromptInputActionMenuItem,
  } from '@/components/ai-elements/prompt-input'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Textarea } from '@/components/ui/textarea'
  import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
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
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
  } from '@/components/ui/tooltip'
  import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupAction,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarInput,
    SidebarInset,
    SidebarMenu,
    SidebarMenuAction,
    SidebarMenuBadge,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubButton,
    SidebarMenuSubItem,
    SidebarProvider,
    SidebarRail,
    SidebarSeparator,
    SidebarTrigger,
    useSidebar,
  } from '@/components/ui/sidebar'
  import {
    Bot,
    Send,
    Plus,
    Search,
    Trash2,
    RotateCcw,
    MessageSquare,
    Brain,
    Cpu,
    Sparkles,
    MoreVertical,
    Copy,
    Edit,
    RefreshCw,
    Share2,
    Settings,
    Moon,
    Sun,
    Paperclip,
    Mic,
    Square,
    ChevronDown,
    ChevronRight,
    History,
    MessageCircle,
  } from 'lucide-vue-next'

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
  const darkMode = ref(false)
  const isRecording = ref(false)
  const attachments = ref<File[]>([])

  // Mobile overlay component with sidebar context
  const MobileOverlay = defineComponent({
    setup() {
      const { open, setOpen } = useSidebar()
      return { open, setOpen }
    },
    template: `
      <div
        v-if="open"
        class="lg:hidden fixed inset-0 z-40 bg-black/50"
        @click="setOpen(false)"
      ></div>
    `
  })

  // Model options
  const modelOptions = [
    {
      id: 'gpt-4',
      name: 'GPT-4',
      description: 'Most capable model for complex tasks',
      icon: Brain,
      color: 'text-purple-600',
    },
    {
      id: 'gpt-3.5-turbo',
      name: 'GPT-3.5 Turbo',
      description: 'Fast and efficient for most tasks',
      icon: Cpu,
      color: 'text-blue-600',
    },
    {
      id: 'claude-3',
      name: 'Claude 3',
      description: 'Advanced reasoning and analysis',
      icon: Sparkles,
      color: 'text-orange-600',
    },
  ]

  // Filter conversations based on search
  const filteredConversations = computed(() => {
    if (!searchQuery.value) return conversations.value
    return conversations.value.filter((conv) =>
      conv.title.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
  })

  // Get current conversation
  const currentConversation = computed(() => {
    return conversations.value.find((conv) => conv.id === currentConversationId.value)
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
      messageCount: 2,
    }

    // Add another sample conversation
    const conv2: Conversation = {
      id: '2',
      title: 'Project Discussion',
      lastMessage: 'Let me help you with that feature.',
      timestamp: new Date(now.getTime() - 3600000), // 1 hour ago
      model: 'gpt-4',
      messageCount: 5,
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
          content: "Hello! I'm Iroh Chat, your AI assistant. How can I help you today?",
          timestamp: new Date(),
          conversationId: '1',
        },
      ]
    } else if (conversationId === '2') {
      messages.value = [
        {
          id: '2-1',
          role: 'user',
          content: 'Can you help me implement this new feature?',
          timestamp: new Date(Date.now() - 300000),
          conversationId: '2',
        },
        {
          id: '2-2',
          role: 'assistant',
          content:
            "I'd be happy to help you with that feature. Let me break it down into manageable steps and guide you through the implementation process.",
          timestamp: new Date(Date.now() - 180000),
          conversationId: '2',
        },
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
      messageCount: 0,
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

    const index = conversations.value.findIndex((conv) => conv.id === conversationId)
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
      conversationId: currentConversationId.value,
    })

    // Update conversation info
    const convIndex = conversations.value.findIndex(
      (conv) => conv.id === currentConversationId.value
    )
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
      await new Promise((resolve) => setTimeout(resolve, 1000))

      const botResponse = getBotResponse(userMessage)
      messages.value.push({
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: botResponse,
        timestamp: new Date(),
        conversationId: currentConversationId.value,
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
      'I understand. How can I help you with that?',
      'Thanks for sharing! What else would you like to discuss?',
      'I see your point. Let me think about that for a moment.',
      "That's a great question! Here's what I think about it.",
      'I appreciate you sharing that with me.',
      'That makes sense. What are your thoughts on this?',
      'Interesting perspective! Have you considered other angles?',
      "I'd love to help you explore that idea further.",
      "That's a fascinating topic! Let me share what I know about it.",
    ]
    return responses[Math.floor(Math.random() * responses.length)]
  }

  // Watch for message changes to trigger auto-scroll
  watch(
    messages,
    async () => {
      await scrollToBottom()
    },
    { deep: true }
  )

  // Additional methods
  function editConversationTitle(conversationId: string) {
    const conversation = conversations.value.find(c => c.id === conversationId)
    if (conversation) {
      const newTitle = prompt('Enter new title:', conversation.title)
      if (newTitle && newTitle.trim()) {
        conversation.title = newTitle.trim()
      }
    }
  }

  function shareConversation(conversationId: string) {
    // Implement sharing functionality
    alert('Share functionality coming soon!')
  }

  function handleFileUpload(event: Event) {
    const files = (event.target as HTMLInputElement).files
    if (files) {
      attachments.value = Array.from(files)
    }
  }

  function toggleRecording() {
    isRecording.value = !isRecording.value
    // Implement recording logic
  }

  function regenerateResponse() {
    // Implement response regeneration
    alert('Regenerate response functionality coming soon!')
  }

  function copyMessage(messageId: string) {
    // Implement message copying
    alert('Copy message functionality coming soon!')
  }

  function startNewChat() {
    createNewConversation()
  }

  function loadChat(chatId: string) {
    loadConversationMessages(chatId)
  }

  function toggleTheme() {
    darkMode.value = !darkMode.value
  }

  function clearChat() {
    if (currentConversationId.value) {
      messages.value = []
      const convIndex = conversations.value.findIndex(c => c.id === currentConversationId.value)
      if (convIndex > -1) {
        conversations.value[convIndex].messageCount = 0
        conversations.value[convIndex].lastMessage = ''
      }
    }
  }

  // Computed property for previous chats
  const previousChats = computed(() => {
    return conversations.value.filter(conv => conv.id !== currentConversationId.value)
  })

  const isDark = computed(() => darkMode.value)

  // Initialize on mount
  if (typeof window !== 'undefined') {
    initializeConversations()
  }
</script>

<template>
  <TooltipProvider>
    <SidebarProvider>
      <div
        class="flex h-screen font-sans bg-background"
        :class="darkMode ? 'dark' : ''"
        :style="{
          fontFamily:
            'Inter, -apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, sans-serif',
        }"
      >
        <!-- Sidebar -->
        <Sidebar collapsible="icon" variant="sidebar">
          <SidebarHeader>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton size="lg" @click="createNewConversation">
                  <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80 text-sidebar-primary-foreground">
                    <Bot class="size-4" />
                  </div>
                  <div class="flex flex-col gap-0.5 text-left">
                    <span class="font-semibold text-lg tracking-wide">Iroh Chat</span>
                    <span class="text-xs text-sidebar-foreground">AI Assistant</span>
                  </div>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>

            <SidebarSeparator class="mx-2.5" />

            <SidebarInput placeholder="Search conversations..." v-model="searchQuery" class="mt-2" />
          </SidebarHeader>

          <SidebarContent>
            <SidebarGroup>
              <SidebarGroupLabel>Conversations</SidebarGroupLabel>
              <SidebarGroupAction>
                <Tooltip>
                  <TooltipTrigger as-child>
                    <Button @click="createNewConversation" size="sm">
                      <Plus class="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">
                    <p>New Chat</p>
                  </TooltipContent>
                </Tooltip>
              </SidebarGroupAction>
              <SidebarMenu>
                <SidebarMenuItem
                  v-for="conversation in filteredConversations"
                  :key="conversation.id"
                  @click="loadConversationMessages(conversation.id)"
                >
                  <SidebarMenuButton
                    :is-active="currentConversationId === conversation.id"
                    class="w-full justify-start text-left"
                  >
                    <History class="size-4" />
                    <div class="flex-1">
                      <div class="font-medium">{{ conversation.title }}</div>
                      <div class="text-xs text-muted-foreground truncate">
                        {{ conversation.lastMessage || 'No messages yet' }}
                      </div>
                    </div>
                    <SidebarMenuBadge>
                      {{ conversation.messageCount }}
                    </SidebarMenuBadge>
                  </SidebarMenuButton>
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <SidebarMenuAction show-on-hover>
                        <MoreVertical class="h-4 w-4" />
                      </SidebarMenuAction>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent side="bottom" align="end">
                      <DropdownMenuItem @click.stop="editConversationTitle(conversation.id)">
                        <Edit class="h-4 w-4 mr-2" />
                        Rename
                      </DropdownMenuItem>
                      <DropdownMenuItem @click.stop="shareConversation(conversation.id)">
                        <Share2 class="h-4 w-4 mr-2" />
                        Share
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem
                        @click.stop="deleteConversation(conversation.id, $event)"
                        class="text-destructive"
                      >
                        <Trash2 class="h-4 w-4 mr-2" />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>

            <SidebarSeparator class="mx-2.5" />

            <SidebarGroup>
              <SidebarGroupLabel>Settings</SidebarGroupLabel>
              <SidebarMenu>
                <SidebarMenuItem>
                  <SidebarMenuButton>
                    <Brain class="size-4" />
                    <span>AI Models</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton>
                    <Settings class="size-4" />
                    <span>Preferences</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton @click="darkMode = !darkMode">
                    <Sun v-if="!darkMode" class="size-4" />
                    <Moon v-else class="size-4" />
                    <span>{{ darkMode ? 'Dark Mode' : 'Light Mode' }}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          </SidebarContent>

          <SidebarFooter>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton as-child>
                  <Select v-model="selectedModel">
                    <SelectTrigger class="w-full justify-start">
                      <component
                        :is="modelOptions.find(m => m.id === selectedModel)?.icon"
                        :class="`w-4 h-4 ${modelOptions.find(m => m.id === selectedModel)?.color}`"
                      />
                      {{ modelOptions.find(m => m.id === selectedModel)?.name }}
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem v-for="model in modelOptions" :key="model.id" :value="model.id">
                        <div class="flex items-center gap-3">
                          <component :is="model.icon" :class="`w-4 h-4 ${model.color}`" />
                          <div>
                            <div class="font-medium">{{ model.name }}</div>
                            <div class="text-xs text-muted-foreground">{{ model.description }}</div>
                          </div>
                        </div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>

            <SidebarSeparator class="mx-2.5" />

            <div class="text-xs text-muted-foreground text-center px-2.5 pb-2">
              {{ filteredConversations.length }} conversations
            </div>
          </SidebarFooter>
        </Sidebar>

        <SidebarRail />

        <!-- Mobile Menu Trigger -->
        <SidebarTrigger class="lg:hidden fixed top-4 left-4 z-50" />

        <!-- Mobile Overlay -->
        <MobileOverlay />

        <!-- Main Content -->
        <SidebarInset class="flex-1">
          <!-- Header -->
          <header class="bg-background/95 backdrop-blur-lg border-b shadow-sm">
            <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16">
              <div class="flex items-center justify-between h-16">
                <div class="flex items-center gap-3">
                  <Avatar class="h-8 w-8">
                    <AvatarFallback>
                      <Bot class="w-4 h-4" />
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <h1 class="text-lg font-semibold">
                      {{ currentConversation?.title || 'New Chat' }}
                    </h1>
                    <p class="text-sm text-muted-foreground hidden sm:block">
                      {{
                        currentConversation?.model
                          ? modelOptions.find((m) => m.id === currentConversation.model)?.name
                          : 'AI Assistant'
                      }}
                    </p>
                  </div>
                </div>

              <div class="flex items-center gap-2">
                <!-- Status Indicator -->
                <div class="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm bg-muted">
                  <span
                    class="w-2 h-2 rounded-full"
                    :class="[
                      status === 'loading' ? 'bg-yellow-500 animate-pulse' :
                      status === 'error' ? 'bg-red-500' :
                      'bg-green-500'
                    ]"
                  ></span>
                  <span class="hidden sm:inline">
                    {{
                      status === 'loading' ? 'Thinking...' :
                      status === 'error' ? 'Error' :
                      'Ready'
                    }}
                  </span>
                </div>

                <!-- Actions -->
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                      <MoreVertical class="w-4 h-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="regenerateResponse">
                      <RefreshCw class="w-4 h-4 mr-2" />
                      Regenerate
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                      <Share2 class="w-4 h-4 mr-2" />
                      Share
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem>
                      <Settings class="w-4 h-4 mr-2" />
                      Settings
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          </div>
        </header>

      <!-- Chat Area -->
        <div class="flex-1 overflow-hidden bg-muted/30">
          <div class="h-full flex flex-col">
            <!-- Messages Container -->
            <div
              ref="messagesContainer"
              class="flex-1 overflow-y-auto"
              @scroll="handleScroll"
            >
              <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16 py-6">
                <Conversation class="h-full">
                  <ConversationContent>
                    <ConversationEmptyState
                      v-if="messages.length === 0"
                      title="Start a conversation"
                      description="Type a message below to begin chatting with your AI assistant"
                    >
                      <Avatar class="w-16 h-16 mb-4">
                        <AvatarFallback>
                          <Bot class="w-8 h-8 text-muted-foreground" />
                        </AvatarFallback>
                      </Avatar>
                      <h3 class="text-lg font-semibold mb-2">Hello! I'm your AI assistant</h3>
                      <p class="text-muted-foreground text-center max-w-md">
                        How can I help you today? Feel free to ask me anything, from coding questions to creative writing.
                      </p>
                    </ConversationEmptyState>

                    <Message
                      v-for="message in messages"
                      :key="message.id"
                      :from="message.role"
                      class="mb-6"
                    >
                      <MessageAvatar class="mr-3">
                        <Avatar class="h-8 w-8">
                          <AvatarFallback :class="[
                            message.role === 'user'
                              ? 'bg-primary text-primary-foreground'
                              : 'bg-secondary text-secondary-foreground'
                          ]">
                            {{ message.role === 'user' ? 'U' : 'AI' }}
                          </AvatarFallback>
                        </Avatar>
                      </MessageAvatar>

                      <MessageContent class="flex-1">
                        <MessageResponse>
                          <div class="rounded-lg p-4" :class="[
                            message.role === 'user'
                              ? 'bg-primary text-primary-foreground ml-auto'
                              : 'bg-muted'
                          ]">
                            {{ message.content }}
                          </div>
                        </MessageResponse>

                        <MessageActions class="mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <MessageAction @click="copyMessage(message.id)">
                            <Copy class="w-4 h-4" />
                          </MessageAction>
                          <MessageAction @click="regenerateResponse" v-if="message.role === 'assistant'">
                            <RefreshCw class="w-4 h-4" />
                          </MessageAction>
                        </MessageActions>
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
        <div class="bg-background/95 backdrop-blur-lg border-t p-4">
          <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16">
            <form @submit="handleSubmit">
              <PromptInput class="relative group">
                <div class="flex items-end gap-2 bg-muted/50 rounded-xl border border-border focus-within:border-primary focus-within:ring-2 focus-within:ring-primary/20 transition-all">
                  <!-- Attachment Button -->
                  <Tooltip>
                    <TooltipTrigger as-child>
                      <Button type="button" variant="ghost" size="icon" class="ml-2 mb-2">
                        <Paperclip class="w-4 h-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Attach files</p>
                    </TooltipContent>
                  </Tooltip>

                  <!-- Text Input -->
                  <PromptInputTextarea
                    v-model="input"
                    placeholder="Type your message here..."
                    class="flex-1 resize-none border-0 bg-transparent py-3 px-2 focus:ring-0 min-h-[56px] max-h-32"
                    :disabled="status === 'loading'"
                    @keydown.enter.prevent="handleSubmit"
                  />

                  <!-- Voice Input Button -->
                  <Tooltip>
                    <TooltipTrigger as-child>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        class="mr-2 mb-2"
                        @click="toggleRecording"
                        :class="isRecording ? 'text-red-500' : ''"
                      >
                        <Mic v-if="!isRecording" class="w-4 h-4" />
                        <Square v-else class="w-4 h-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{{ isRecording ? 'Stop recording' : 'Voice input' }}</p>
                    </TooltipContent>
                  </Tooltip>

                  <!-- Submit Button -->
                  <PromptInputSubmit
                    :status="status === 'loading' ? 'streaming' : 'ready'"
                    :disabled="!input.trim() || status === 'loading'"
                    class="m-2"
                    @click="handleSubmit"
                  />
                </div>

                <!-- Attachments Preview -->
                <PromptInputAttachments v-if="attachments.length > 0" class="mt-2">
                  <div class="flex flex-wrap gap-2">
                    <div
                      v-for="(file, index) in attachments"
                      :key="index"
                      class="flex items-center gap-2 bg-muted px-2 py-1 rounded-md text-sm"
                    >
                      <Paperclip class="w-3 h-3" />
                      <span>{{ file.name }}</span>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-4 w-4 hover:bg-destructive hover:text-destructive-foreground"
                        @click="attachments.splice(index, 1)"
                      >
                        ×
                      </Button>
                    </div>
                  </div>
                </PromptInputAttachments>
              </PromptInput>
            </form>

            <!-- Error Display -->
            <div
              v-if="error"
              class="mt-3 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm"
            >
              <p class="m-0">Error: {{ error }}</p>
            </div>
          </div>
        </div>

    <!-- Footer -->
        <footer class="bg-muted/50 border-t px-4 py-2">
          <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16 text-center">
            <p class="text-xs text-muted-foreground m-0">
              Powered by AI Elements Vue • Built with Tauri + Vue 3
            </p>
          </div>
        </footer>
        </SidebarInset>
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
              <label class="text-sm font-medium mb-2 block">Select Model</label>
              <Select v-model="selectedModel">
                <SelectTrigger>
                  <SelectValue placeholder="Choose a model" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="model in modelOptions" :key="model.id" :value="model.id">
                    <div class="flex items-center gap-3">
                      <component :is="model.icon" :class="`w-5 h-5 ${model.color}`" />
                      <div>
                        <div class="font-medium">{{ model.name }}</div>
                        <div class="text-sm text-muted-foreground">{{ model.description }}</div>
                      </div>
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" @click="newChatDialogOpen = false">Cancel</Button>
            <Button @click="createNewConversation">Create Chat</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </SidebarProvider>
  </TooltipProvider>
</template>

<style scoped>
  /* 移除自定义 CSS，全部使用 Tailwind CSS */
</style>
<style>
  /* 全局样式重置 - 利用 Tailwind 的基础层 */
  html,
  body {
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
