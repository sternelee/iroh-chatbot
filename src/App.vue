<script setup lang="ts">
  import { ref, computed, watch, nextTick, defineComponent } from 'vue'
  import {
    Conversation,
    ConversationContent,
    ConversationScrollButton,
  } from '@/components/ai-elements/conversation'
  import {
    Message,
    MessageAction,
    MessageActions,
    MessageAttachment,
    MessageAttachments,
    MessageBranch,
    MessageBranchContent,
    MessageBranchNext,
    MessageBranchPage,
    MessageBranchPrevious,
    MessageBranchSelector,
    MessageContent,
    MessageResponse,
    MessageToolbar,
  } from '@/components/ai-elements/message'
  import {
    PromptInput,
    PromptInputAttachment,
    PromptInputAttachments,
    PromptInputBody,
    PromptInputButton,
    PromptInputCommand,
    PromptInputCommandEmpty,
    PromptInputCommandGroup,
    PromptInputCommandInput,
    PromptInputCommandItem,
    PromptInputCommandList,
    PromptInputCommandSeparator,
    PromptInputFooter,
    PromptInputHeader,
    PromptInputHoverCard,
    PromptInputHoverCardContent,
    PromptInputHoverCardTrigger,
    PromptInputProvider,
    PromptInputSubmit,
    PromptInputTab,
    PromptInputTabBody,
    PromptInputTabItem,
    PromptInputTabLabel,
    PromptInputTextarea,
    PromptInputTools,
  } from '@/components/ai-elements/prompt-input'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
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
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
  import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupAction,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarInput,
    SidebarInset,
    SidebarMenu,
    SidebarMenuAction,
    SidebarMenuBadge,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarProvider,
    SidebarRail,
    SidebarSeparator,
    SidebarTrigger,
    useSidebar,
  } from '@/components/ui/sidebar'
  // New shadcn-vue components
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
  import { Badge } from '@/components/ui/badge'
  import { ScrollArea } from '@/components/ui/scroll-area'
  import { Switch } from '@/components/ui/switch'
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
  } from '@/components/ui/accordion'
  import {
    Bot,
    Plus,
    Trash2,
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
    ChevronRight,
    History,
    LogOut,
    AtSign,
    Ruler,
    Files,
    Globe,
    Image,
    Check,
    ThumbsUp,
    ThumbsDown,
    Code,
    Lightbulb,
    Zap,
    BookOpen,
    PenTool,
    PanelLeftClose,
    PanelLeft,
  } from 'lucide-vue-next'
  import type { PromptInputMessage } from '@/components/ai-elements/prompt-input'
  import { useCompletion } from '@ai-sdk/vue'

  interface Attachment {
    type: 'file'
    url: string
    mediaType?: string
    filename?: string
  }

  interface Version {
    id: string
    content: string
  }

  interface EnhancedMessage {
    key: string
    from: 'user' | 'assistant'
    versions?: Version[]
    content?: string
    attachments?: Attachment[]
  }

  // AI SDK compatible message structure
  interface ChatMessage {
    id: string
    role: 'user' | 'assistant' | 'system'
    content: string
    createdAt?: Date
    attachments?: Attachment[]
    metadata?: Record<string, unknown>
  }

  // AI SDK compatible conversation structure
  interface ConversationData {
    id: string
    title: string
    messages: ChatMessage[]
    createdAt: Date
    updatedAt: Date
    model: string
    metadata?: Record<string, unknown>
  }

  // Custom fetch function to handle Tauri byte array responses
  const customFetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    // Only use custom fetch for API calls in Tauri environment
    if (typeof window !== 'undefined' && window.__TAURI__ && typeof input === 'string' && input.startsWith('/api/')) {
      try {
        console.log('🚀 Custom fetch called for:', input)
        const { invoke } = await import('@tauri-apps/api/core')

        // Parse the URL and request body
        const url = new URL(input, window.location.origin)
        const body = init?.body ? (typeof init.body === 'string' ? init.body : JSON.stringify(init.body)) : null

        console.log('📤 Request to Tauri:', {
          uri: url.pathname + url.search,
          method: init?.method || 'GET',
          headers: init?.headers || {},
          body: body
        })

        // Call Tauri's local_app_request
        const response = await invoke('local_app_request', {
          localRequest: {
            uri: url.pathname + url.search,
            method: init?.method || 'GET',
            headers: init?.headers || {},
            body: body
          }
        })

        console.log('📥 Raw Tauri response:', response)

        // Handle the response from Tauri
        const tauriResponse = response as { status_code: number; body: number[] | string; headers: Record<string, string> }

        // Convert body to string if it's a byte array
        let bodyText: string
        if (Array.isArray(tauriResponse.body)) {
          console.log('🔄 Converting byte array to string:', tauriResponse.body.slice(0, 20), '...')
          bodyText = new TextDecoder().decode(new Uint8Array(tauriResponse.body))
          console.log('✅ Converted body:', bodyText)
        } else {
          bodyText = tauriResponse.body
          console.log('✅ Body already string:', bodyText)
        }

        // Return a proper Response object
        const finalResponse = new Response(bodyText, {
          status: tauriResponse.status_code,
          headers: tauriResponse.headers
        })

        console.log('🎯 Final response object:', {
          status: finalResponse.status,
          headers: Object.fromEntries(finalResponse.headers.entries()),
          bodyUsed: finalResponse.bodyUsed
        })

        return finalResponse
      } catch (error) {
        console.error('❌ Tauri fetch error:', error)
        // Fallback to regular fetch if Tauri call fails
        console.log('🔄 Falling back to regular fetch')
        return fetch(input, init)
      }
    }

    // Use regular fetch for non-API calls or non-Tauri environments
    return fetch(input, init)
  }

  // Initialize ai-sdk/vue useCompletion - for single prompt/response completion
  const { completion, isLoading, error, complete } = useCompletion({
    api: '/api/chat',
    fetch: customFetch,
  })

  const input = ref('')
  const enhancedMessages = ref<EnhancedMessage[]>([])
  const conversations = ref<ConversationData[]>([])
  const currentConversationId = ref<string>('')
  const searchQuery = ref('')
  const status = computed(() => (isLoading.value ? 'loading' : error.value ? 'error' : 'idle'))
  const selectedModel = ref('gpt-3.5-turbo')
  const newChatDialogOpen = ref(false)
  const messagesContainer = ref<HTMLElement>()
  const autoScrollEnabled = ref(true)
  const darkMode = ref(false)
  const settingsDialogOpen = ref(false)

  // Settings state
  const settingsTab = ref('general')
  const enableNotifications = ref(true)
  const enableSoundEffects = ref(false)
  const autoSaveConversations = ref(true)

  // Enhanced message features
  const liked = ref<Record<string, boolean>>({})
  const disliked = ref<Record<string, boolean>>({})

  // Cursor-style state
  // eslint-disable-next-line no-undef
  const textareaRef = ref<HTMLTextAreaElement | null>(null)

  const user = ref({
    email: 'user@example.com',
    name: 'User Name',
    avatar: 'https://avatars.githubusercontent.com/u/1000000?v=4', // Placeholder avatar
  })

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
    `,
  })

  // Constants for Cursor-style behavior
  const SUBMITTING_TIMEOUT = 200
  const STREAMING_TIMEOUT = 2000

  // Sample data for demonstration
  const sampleFiles = {
    activeTabs: [{ path: 'prompt-input.tsx', location: 'packages/elements/src' }],
    recents: [
      { path: 'queue.tsx', location: 'apps/test/app/examples' },
      { path: 'queue.tsx', location: 'packages/elements/src' },
    ],
    added: [
      { path: 'prompt-input.tsx', location: 'packages/elements/src' },
      { path: 'queue.tsx', location: 'apps/test/app/examples' },
      { path: 'queue.tsx', location: 'packages/elements/src' },
    ],
    filesAndFolders: [
      { path: 'prompt-input.tsx', location: 'packages/elements/src' },
      { path: 'queue.tsx', location: 'apps/test/app/examples' },
    ],
    code: [{ path: 'prompt-input.tsx', location: 'packages/elements/src' }],
    docs: [{ path: 'README.md', location: 'packages/elements' }],
  }

  const sampleTabs = {
    active: [{ path: 'packages/elements/src/task-queue-panel.tsx' }],
    recents: [
      { path: 'apps/test/app/examples/task-queue-panel.tsx' },
      { path: 'apps/test/app/page.tsx' },
      { path: 'packages/elements/src/task.tsx' },
      { path: 'apps/test/app/examples/prompt-input.tsx' },
      { path: 'packages/elements/src/queue.tsx' },
      { path: 'apps/test/app/examples/queue.tsx' },
    ],
  }

  // Model options (legacy, kept for compatibility)
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

  // Suggestion cards for empty state
  const suggestionCards = [
    {
      id: 'code',
      title: 'Write Code',
      description: 'Help me write a function or debug code',
      icon: Code,
      prompt: 'Help me write a function that...',
      color: 'text-blue-500',
    },
    {
      id: 'explain',
      title: 'Explain Concepts',
      description: 'Learn about programming concepts',
      icon: BookOpen,
      prompt: 'Explain the concept of...',
      color: 'text-green-500',
    },
    {
      id: 'brainstorm',
      title: 'Brainstorm Ideas',
      description: 'Generate creative ideas and solutions',
      icon: Lightbulb,
      prompt: 'Help me brainstorm ideas for...',
      color: 'text-yellow-500',
    },
    {
      id: 'improve',
      title: 'Improve Writing',
      description: 'Polish and enhance your text',
      icon: PenTool,
      prompt: 'Help me improve this text:',
      color: 'text-purple-500',
    },
  ]

  // Format timestamp for display
  function formatTimestamp(date: Date): string {
    const now = new Date()
    const diff = now.getTime() - date.getTime()

    if (diff < 60000) return 'Just now'
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
    return date.toLocaleDateString()
  }

  // Get model badge color
  function getModelBadgeVariant(
    modelId: string
  ): 'default' | 'secondary' | 'destructive' | 'outline' {
    if (modelId.includes('gpt-4')) return 'default'
    if (modelId.includes('claude')) return 'secondary'
    return 'outline'
  }

  // Get last message content from conversation
  function getLastMessage(conv: ConversationData): string {
    if (!conv.messages || conv.messages.length === 0) return 'No messages yet'
    const lastMsg = conv.messages[conv.messages.length - 1]
    return lastMsg.content.substring(0, 50) + (lastMsg.content.length > 50 ? '...' : '')
  }

  // Get message count from conversation
  function getMessageCount(conv: ConversationData): number {
    return conv.messages?.length || 0
  }

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

    // Create first conversation with AI SDK compatible structure
    const conv1: ConversationData = {
      id: '1',
      title: 'Vue 3 Composition API Guide',
      messages: [
        {
          id: 'msg-1',
          role: 'user',
          content: 'How do Vue composition APIs work and when should I use them?',
          createdAt: now,
        },
        {
          id: 'msg-2',
          role: 'assistant',
          content: 'The Vue Composition API is a powerful feature...',
          createdAt: now,
        },
      ],
      createdAt: now,
      updatedAt: now,
      model: 'gpt-3.5-turbo',
    }

    // Add another sample conversation
    const conv2: ConversationData = {
      id: '2',
      title: 'Frontend Development Discussion',
      messages: [
        {
          id: 'msg-3',
          role: 'user',
          content: 'Can you help me implement this new feature?',
          createdAt: new Date(now.getTime() - 3600000),
        },
        {
          id: 'msg-4',
          role: 'assistant',
          content: "I'd be happy to help with that feature...",
          createdAt: new Date(now.getTime() - 3600000),
        },
      ],
      createdAt: new Date(now.getTime() - 3600000),
      updatedAt: new Date(now.getTime() - 3600000),
      model: 'gpt-4',
    }

    conversations.value = [conv1, conv2]
    currentConversationId.value = conv1.id

    // Initialize enhanced messages for demo
    enhancedMessages.value = [
      {
        key: 'msg-1',
        from: 'user',
        content: 'How do Vue composition APIs work and when should I use them?',
        attachments: [
          {
            type: 'file',
            url: 'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=400&fit=crop',
            mediaType: 'image/jpeg',
            filename: 'palace-of-fine-arts.jpg',
          },
          {
            type: 'file',
            url: '',
            mediaType: 'application/pdf',
            filename: 'vue-guide.pdf',
          },
        ],
      },
      {
        key: 'msg-2',
        from: 'assistant',
        versions: [
          {
            id: 'version-1',
            content: `# Vue Composition API Guide

The Vue Composition API is a set of functions that let you use Vue's reactivity and lifecycle features inside the \`setup()\` function of your components. Here's what you need to know:

## Core Composables

### ref()
Creates reactive primitive values:

\`\`\`vue
<script setup>
import { ref } from 'vue'

const count = ref(0)
<\/script>

<template>
  <button @click="count++">Count: {{ count }}</button>
</template>
\`\`\`

### watch()
Runs side effects when reactive values change:

\`\`\`vue
<script setup>
import { ref, watch } from 'vue'

const count = ref(0)

watch(count, (newVal, oldVal) => {
  console.log(\`Count changed from \${oldVal} to \${newVal}\`)
})
<\/script>
\`\`\`

## When to Use the Composition API

- ✅ **For complex logic** — Easier to organize and reuse reactive state
- ✅ **For reusable code** — Create your own composables (like custom hooks)
- ✅ **For TypeScript support** — More type-friendly than Options API
- ❌ **For simple components** — The Options API might be enough

Would you like to explore more advanced composables like \`computed\` or \`onMounted\`?`,
          },
          {
            id: 'version-2',
            content: `The Vue Composition API is a modern way to write components in Vue 3. It replaces the Options API's data, methods, and computed properties with a single \`setup()\` function.

Here are the most common composables:

- **ref()** — creates reactive primitive values
- **reactive()** — makes entire objects reactive
- **computed()** — creates derived reactive values
- **watch()** — runs side effects on data changes
- **onMounted()** — lifecycle hook for when a component is mounted

Here's a simple example:

\`\`\`vue
<script setup>
import { ref, onMounted } from 'vue'

const count = ref(0)

onMounted(() => {
  console.log('Component mounted!')
})
<\/script>

<template>
  <button @click="count++">Clicked {{ count }} times</button>
</template>
\`\`\`

Which specific composable would you like to learn more about?`,
          },
          {
            id: 'version-3',
            content: `Absolutely! The Vue Composition API brings a new, more flexible way to manage logic and reactivity in Vue components.

## Key Benefits

1. **Cleaner code organization** — Group related logic by feature
2. **Reusable logic** — Build and share your own composables
3. **Better TypeScript support** — Stronger typing than the Options API

## Most Popular Composables

| Composable | Purpose |
|-------------|----------|
| ref | Reactive primitive values |
| reactive | Reactive objects |
| computed | Derived reactive values |
| watch | React to data changes |
| onMounted | Run code when component mounts |
| onUnmounted | Cleanup logic when destroyed |

The beauty of the Composition API is that it lets you reuse stateful logic without changing your component structure. Want to dive into a specific composable?`,
          },
        ],
      },
    ]

    // Load messages for current conversation
    loadConversationMessages(conv1.id)
  }

  // Load messages for a specific conversation
  function loadConversationMessages(conversationId: string) {
    currentConversationId.value = conversationId

    if (conversationId === '1') {
      // Keep demo enhanced messages for conversation 1
    } else if (conversationId === '2') {
      // Clear enhanced messages for conversation 2
      enhancedMessages.value = []
    } else {
      // Clear enhanced messages for new conversations
      enhancedMessages.value = []
    }

    input.value = ''
    error.value = null
    status.value = 'idle'
  }

  // Create a new conversation
  function createNewConversation() {
    const newId = Date.now().toString()
    const now = new Date()
    const newConversation: ConversationData = {
      id: newId,
      title: 'New Conversation',
      messages: [],
      createdAt: now,
      updatedAt: now,
      model: selectedModel.value,
    }

    conversations.value.unshift(newConversation)
    currentConversationId.value = newId
    enhancedMessages.value = [] // Clear enhanced messages for new conversations
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

  // Cursor-style submit handler using ai-sdk/vue useCompletion
  async function handleCursorSubmit(message: PromptInputMessage) {
    const hasText = !!message.text
    const hasAttachments = !!message.files?.length

    if (!hasText && !hasAttachments) {
      return
    }

    const prompt = message.text || 'File uploaded'

    // Add user message to enhanced messages for UI display
    const newUserMessage: EnhancedMessage = {
      key: Date.now().toString(),
      from: 'user',
      content: prompt,
    }
    enhancedMessages.value.push(newUserMessage)

    try {
      // Use ai-sdk/vue complete function for completion
      const response = await complete(prompt)

      // Add assistant response to enhanced messages
      if (response) {
        const assistantMessage: EnhancedMessage = {
          key: (Date.now() + 1).toString(),
          from: 'assistant',
          content: response,
        }
        enhancedMessages.value.push(assistantMessage)
      }
    } catch (err) {
      console.error('Completion error:', err)
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

  // Watch for enhanced message changes to trigger auto-scroll
  watch(
    enhancedMessages,
    async () => {
      await scrollToBottom()
    },
    { deep: true }
  )

  // Additional methods
  function editConversationTitle(_conversationId: string) {
    const conversation = conversations.value.find((c) => c.id === _conversationId)
    if (conversation) {
      const newTitle = prompt('Enter new title:', conversation.title)
      if (newTitle && newTitle.trim()) {
        conversation.title = newTitle.trim()
      }
    }
  }

  function shareConversation(_conversationId: string) {
    // Implement sharing functionality
    alert('Share functionality coming soon!')
  }

  function regenerateResponse() {
    // Implement response regeneration
    alert('Regenerate response functionality coming soon!')
  }

  // Enhanced message functions
  function handleCopy(content: string) {
    navigator.clipboard.writeText(content)
  }

  function handleRetry() {
    // Implement retry logic
    regenerateResponse()
  }

  function toggleLike(key: string) {
    liked.value = {
      ...liked.value,
      [key]: !liked.value[key],
    }
    // Reset dislike when liked
    if (liked.value[key]) {
      disliked.value = {
        ...disliked.value,
        [key]: false,
      }
    }
  }

  function toggleDislike(key: string) {
    disliked.value = {
      ...disliked.value,
      [key]: !disliked.value[key],
    }
    // Reset like when disliked
    if (disliked.value[key]) {
      liked.value = {
        ...liked.value,
        [key]: false,
      }
    }
  }

  function hasMultipleVersions(message: EnhancedMessage) {
    return message.versions && message.versions.length > 1
  }

  function handleBranchChange(_index: number) {
    // Handle branch selection change
  }

  function toggleTheme() {
    darkMode.value = !darkMode.value
  }

  // Handle suggestion card click
  function handleSuggestionClick(prompt: string) {
    // Create a PromptInputMessage-like object and submit
    const message: PromptInputMessage = {
      text: prompt,
      files: [],
    }
    handleCursorSubmit(message)
  }

  // Initialize on mount
  if (typeof window !== 'undefined') {
    initializeConversations()
  }
</script>

<template>
  <TooltipProvider>
    <SidebarProvider>
      <div
        class="flex h-screen font-sans bg-background w-full"
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
                  <div
                    class="flex aspect-square size-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary/80 text-sidebar-primary-foreground"
                  >
                    <Bot class="size-4" />
                  </div>
                  <div class="flex flex-col gap-0.5 text-left">
                    <span class="font-semibold text-lg tracking-wide">Iroh Chat</span>
                    <span class="text-xs text-sidebar-foreground">AI Assistant</span>
                  </div>
                </SidebarMenuButton>
                <!-- Sidebar collapse button -->
                <SidebarTrigger class="ml-auto group-data-[collapsible=icon]:hidden">
                  <Tooltip>
                    <TooltipTrigger as-child>
                      <Button variant="ghost" size="icon" class="size-8">
                        <PanelLeftClose class="size-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="right">
                      <p>Collapse sidebar</p>
                    </TooltipContent>
                  </Tooltip>
                </SidebarTrigger>
              </SidebarMenuItem>
            </SidebarMenu>

            <SidebarSeparator class="mx-2.5" />

            <SidebarInput
              placeholder="Search conversations..."
              v-model="searchQuery"
              class="mt-2 group-data-[collapsible=icon]:hidden"
            />
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
                    class="w-full justify-start text-left h-auto py-2"
                  >
                    <History class="size-4 shrink-0" />
                    <div class="flex-1 min-w-0 overflow-hidden">
                      <div class="flex items-center gap-1.5 mb-0.5">
                        <span class="font-medium text-sm truncate flex-1">
                          {{ conversation.title }}
                        </span>
                        <Badge
                          :variant="getModelBadgeVariant(conversation.model)"
                          class="text-[10px] px-1 py-0 shrink-0"
                        >
                          {{ conversation.model.split('-')[0] }}
                        </Badge>
                      </div>
                      <div class="text-xs text-muted-foreground truncate">
                        {{ getLastMessage(conversation) }}
                      </div>
                      <div class="text-[10px] text-muted-foreground/70 mt-0.5">
                        {{ formatTimestamp(conversation.updatedAt) }}
                      </div>
                    </div>
                    <SidebarMenuBadge class="shrink-0">
                      {{ getMessageCount(conversation) }}
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
                  <SidebarMenuButton @click="settingsDialogOpen = true">
                    <Brain class="size-4" />
                    <span>AI Models</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton @click="settingsDialogOpen = true">
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
            <SidebarGroup>
              <SidebarMenuItem>
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <SidebarMenuButton class="h-auto justify-start py-2 group">
                      <Avatar class="h-7 w-7">
                        <AvatarImage :src="user.avatar" />
                        <AvatarFallback>{{ user.name.charAt(0) }}</AvatarFallback>
                      </Avatar>
                      <div class="flex flex-col items-start gap-px overflow-hidden">
                        <p class="w-full truncate text-sm font-medium">
                          {{ user.name }}
                        </p>
                        <p class="w-full truncate text-xs text-muted-foreground">
                          {{ user.email }}
                        </p>
                      </div>
                      <ChevronRight
                        class="ml-auto h-4 w-4 shrink-0 opacity-0 transition-all group-hover:opacity-100 peer-data-[state=open]:opacity-100"
                      />
                    </SidebarMenuButton>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent side="right" align="end" class="w-56">
                    <DropdownMenuItem>
                      <Settings class="w-4 h-4 mr-2" />
                      Settings
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem @click="toggleTheme">
                      <Sun v-if="!darkMode" class="size-4 mr-2" />
                      <Moon v-else class="size-4 mr-2" />
                      <span>{{ darkMode ? 'Dark Mode' : 'Light Mode' }}</span>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem class="text-destructive">
                      <LogOut class="w-4 h-4 mr-2" />
                      Log Out
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </SidebarMenuItem>
            </SidebarGroup>
          </SidebarFooter>
        </Sidebar>

        <SidebarRail />

        <!-- Mobile Overlay -->
        <MobileOverlay />

        <!-- Main Content -->
        <SidebarInset class="flex-1">
          <!-- Header -->
          <header class="bg-background/95 backdrop-blur-lg border-b shadow-sm">
            <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16">
              <div class="flex items-center justify-between h-16">
                <div class="flex items-center gap-3">
                  <!-- Sidebar expand button (visible when sidebar is collapsed) -->
                  <SidebarTrigger class="hidden lg:flex">
                    <Tooltip>
                      <TooltipTrigger as-child>
                        <Button variant="ghost" size="icon" class="size-8">
                          <PanelLeft class="size-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent side="right">
                        <p>Toggle sidebar</p>
                      </TooltipContent>
                    </Tooltip>
                  </SidebarTrigger>
                  <!-- Mobile sidebar trigger -->
                  <SidebarTrigger class="lg:hidden">
                    <Button variant="ghost" size="icon" class="size-8">
                      <PanelLeft class="size-4" />
                    </Button>
                  </SidebarTrigger>
                  <Avatar class="h-8 w-8">
                    <AvatarFallback>
                      <Bot class="w-4 h-4" />
                    </AvatarFallback>
                  </Avatar>
                  <div>
                    <h1 class="text-lg font-semibold">
                      {{ currentConversation?.title || 'New Chat' }}
                    </h1>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <!-- Status Indicator -->
                  <div class="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm bg-muted">
                    <span
                      class="w-2 h-2 rounded-full"
                      :class="[
                        status === 'loading'
                          ? 'bg-yellow-500 animate-pulse'
                          : status === 'error'
                            ? 'bg-red-500'
                            : 'bg-green-500',
                      ]"
                    ></span>
                    <span class="hidden sm:inline">
                      {{
                        status === 'loading'
                          ? 'Thinking...'
                          : status === 'error'
                            ? 'Error'
                            : 'Ready'
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
              <ScrollArea class="flex-1" @scroll="handleScroll">
                <div ref="messagesContainer" class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16 py-6">
                  <Conversation class="h-full">
                    <ConversationContent>
                      <!-- Enhanced Empty State with Suggestion Cards -->
                      <div
                        v-if="enhancedMessages.length === 0"
                        class="flex flex-col items-center justify-center py-12"
                      >
                        <Avatar class="w-20 h-20 mb-6">
                          <AvatarFallback class="bg-gradient-to-br from-primary to-primary/60">
                            <Bot class="w-10 h-10 text-primary-foreground" />
                          </AvatarFallback>
                        </Avatar>
                        <h2 class="text-2xl font-bold mb-2">Welcome to Iroh Chat</h2>
                        <p class="text-muted-foreground text-center max-w-md mb-8">
                          I'm your AI assistant powered by advanced language models. How can I help
                          you today?
                        </p>

                        <!-- Suggestion Cards Grid -->
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full max-w-2xl">
                          <Card
                            v-for="suggestion in suggestionCards"
                            :key="suggestion.id"
                            class="cursor-pointer transition-all hover:shadow-md hover:border-primary/50 group"
                            @click="handleSuggestionClick(suggestion.prompt)"
                          >
                            <CardHeader class="pb-2">
                              <div class="flex items-center gap-3">
                                <div
                                  class="p-2 rounded-lg bg-muted group-hover:bg-primary/10 transition-colors"
                                >
                                  <component
                                    :is="suggestion.icon"
                                    :class="['w-5 h-5', suggestion.color]"
                                  />
                                </div>
                                <CardTitle class="text-base">{{ suggestion.title }}</CardTitle>
                              </div>
                            </CardHeader>
                            <CardContent class="pt-0">
                              <CardDescription>{{ suggestion.description }}</CardDescription>
                            </CardContent>
                          </Card>
                        </div>

                        <!-- Quick Tips Accordion -->
                        <div class="w-full max-w-2xl mt-8">
                          <Accordion type="single" collapsible class="w-full">
                            <AccordionItem value="tips">
                              <AccordionTrigger>
                                <div class="flex items-center gap-2">
                                  <Zap class="w-4 h-4 text-yellow-500" />
                                  <span>Pro Tips for Better Results</span>
                                </div>
                              </AccordionTrigger>
                              <AccordionContent>
                                <div class="space-y-3 text-sm text-muted-foreground">
                                  <p>
                                    •
                                    <strong>Be specific:</strong>
                                    The more context you provide, the better response you'll get.
                                  </p>
                                  <p>
                                    •
                                    <strong>Use examples:</strong>
                                    Show what you're looking for with sample inputs or outputs.
                                  </p>
                                  <p>
                                    •
                                    <strong>Ask follow-ups:</strong>
                                    Don't hesitate to ask for clarification or alternatives.
                                  </p>
                                  <p>
                                    •
                                    <strong>Try different models:</strong>
                                    Each model has strengths - experiment to find what works best.
                                  </p>
                                </div>
                              </AccordionContent>
                            </AccordionItem>
                          </Accordion>
                        </div>
                      </div>

                      <Message
                        v-for="message in enhancedMessages"
                        :key="message.key"
                        :from="message.from"
                      >
                        <!-- Multiple versions with branch selector -->
                        <MessageBranch
                          v-if="hasMultipleVersions(message)"
                          :default-branch="0"
                          @branch-change="handleBranchChange"
                        >
                          <MessageBranchContent>
                            <MessageContent v-for="version in message.versions" :key="version.id">
                              <MessageResponse :content="version.content" />
                            </MessageContent>
                          </MessageBranchContent>

                          <MessageToolbar v-if="message.from === 'assistant'">
                            <MessageBranchSelector :from="message.from">
                              <MessageBranchPrevious />
                              <MessageBranchPage />
                              <MessageBranchNext />
                            </MessageBranchSelector>

                            <MessageActions>
                              <MessageAction
                                label="Retry"
                                tooltip="Regenerate response"
                                @click="handleRetry"
                              >
                                <RefreshCw class="size-4" />
                              </MessageAction>

                              <MessageAction
                                label="Like"
                                tooltip="Like this response"
                                @click="toggleLike(message.key)"
                              >
                                <ThumbsUp
                                  class="size-4"
                                  :fill="liked[message.key] ? 'currentColor' : 'none'"
                                />
                              </MessageAction>

                              <MessageAction
                                label="Dislike"
                                tooltip="Dislike this response"
                                @click="toggleDislike(message.key)"
                              >
                                <ThumbsDown
                                  class="size-4"
                                  :fill="disliked[message.key] ? 'currentColor' : 'none'"
                                />
                              </MessageAction>

                              <MessageAction
                                label="Copy"
                                tooltip="Copy to clipboard"
                                @click="
                                  handleCopy(
                                    message.versions?.find((v) => v.id === message.key)?.content ||
                                      message.content ||
                                      ''
                                  )
                                "
                              >
                                <Copy class="size-4" />
                              </MessageAction>
                            </MessageActions>
                          </MessageToolbar>
                        </MessageBranch>

                        <!-- Single version without branch selector -->
                        <div v-else>
                          <MessageAttachments
                            v-if="message.attachments && message.attachments.length > 0"
                            class="mb-2"
                          >
                            <MessageAttachment
                              v-for="attachment in message.attachments"
                              :key="attachment.url"
                              :data="{
                                ...attachment,
                                mediaType: attachment.mediaType ?? 'application/octet-stream',
                              }"
                            />
                          </MessageAttachments>

                          <MessageContent>
                            <MessageResponse
                              v-if="message.from === 'assistant'"
                              :content="message.content"
                            />
                            <template v-else>
                              {{ message.content }}
                            </template>
                          </MessageContent>

                          <MessageActions v-if="message.from === 'assistant'">
                            <MessageAction
                              label="Retry"
                              tooltip="Regenerate response"
                              @click="handleRetry"
                            >
                              <RefreshCw class="size-4" />
                            </MessageAction>

                            <MessageAction
                              label="Like"
                              tooltip="Like this response"
                              @click="toggleLike(message.key)"
                            >
                              <ThumbsUp
                                class="size-4"
                                :fill="liked[message.key] ? 'currentColor' : 'none'"
                              />
                            </MessageAction>

                            <MessageAction
                              label="Dislike"
                              tooltip="Dislike this response"
                              @click="toggleDislike(message.key)"
                            >
                              <ThumbsDown
                                class="size-4"
                                :fill="disliked[message.key] ? 'currentColor' : 'none'"
                              />
                            </MessageAction>

                            <MessageAction
                              label="Copy"
                              tooltip="Copy to clipboard"
                              @click="handleCopy(message.content || '')"
                            >
                              <Copy class="size-4" />
                            </MessageAction>
                          </MessageActions>
                        </div>
                      </Message>
                    </ConversationContent>

                    <ConversationScrollButton />
                  </Conversation>
                </div>
              </ScrollArea>
            </div>
          </div>

          <!-- Input Area - Cursor Style -->
          <div class="bg-background/95 backdrop-blur-lg border-t">
            <div class="px-4 sm:px-6 lg:px-8 xl:px-12 2xl:px-16 py-4">
              <div class="flex size-full flex-col justify-end">
                <PromptInputProvider @submit="handleCursorSubmit">
                  <PromptInput global-drop multiple>
                    <PromptInputHeader>
                      <PromptInputHoverCard>
                        <PromptInputHoverCardTrigger>
                          <PromptInputButton class="!h-8" size="icon-sm" variant="outline">
                            <AtSign class="text-muted-foreground" :size="12" />
                          </PromptInputButton>
                        </PromptInputHoverCardTrigger>
                        <PromptInputHoverCardContent class="w-[400px] p-0">
                          <PromptInputCommand>
                            <PromptInputCommandInput
                              class="border-none focus-visible:ring-0"
                              placeholder="Add files, folders, docs..."
                            />
                            <PromptInputCommandList>
                              <PromptInputCommandEmpty class="p-3 text-muted-foreground text-sm">
                                No results found.
                              </PromptInputCommandEmpty>
                              <PromptInputCommandGroup heading="Added">
                                <PromptInputCommandItem value="active-tabs">
                                  <Globe />
                                  <span>Active Tabs</span>
                                  <span class="ml-auto text-muted-foreground">✓</span>
                                </PromptInputCommandItem>
                              </PromptInputCommandGroup>
                              <PromptInputCommandSeparator />
                              <PromptInputCommandGroup heading="Other Files">
                                <PromptInputCommandItem
                                  v-for="(file, index) in sampleFiles.added"
                                  :key="`${file.path}-${index}`"
                                  :value="file.path"
                                >
                                  <Globe class="text-primary" />
                                  <div class="flex flex-col">
                                    <span class="font-medium text-sm">
                                      {{ file.path }}
                                    </span>
                                    <span class="text-muted-foreground text-xs">
                                      {{ file.location }}
                                    </span>
                                  </div>
                                </PromptInputCommandItem>
                              </PromptInputCommandGroup>
                            </PromptInputCommandList>
                          </PromptInputCommand>
                        </PromptInputHoverCardContent>
                      </PromptInputHoverCard>
                      <PromptInputHoverCard>
                        <PromptInputHoverCardTrigger>
                          <PromptInputButton size="sm" variant="outline">
                            <Ruler class="text-muted-foreground" :size="12" />
                            <span>1</span>
                          </PromptInputButton>
                        </PromptInputHoverCardTrigger>
                        <PromptInputHoverCardContent class="divide-y overflow-hidden p-0">
                          <div class="space-y-2 p-3">
                            <p class="font-medium text-muted-foreground text-sm">
                              Attached Project Rules
                            </p>
                            <p class="ml-4 text-muted-foreground text-sm">Always Apply:</p>
                            <p class="ml-8 text-sm">ultracite.mdc</p>
                          </div>
                          <p class="bg-sidebar px-4 py-3 text-muted-foreground text-sm">
                            Click to manage
                          </p>
                        </PromptInputHoverCardContent>
                      </PromptInputHoverCard>
                      <PromptInputHoverCard>
                        <PromptInputHoverCardTrigger>
                          <PromptInputButton size="sm" variant="outline">
                            <Files class="text-muted-foreground" :size="12" />
                            <span>1 Tab</span>
                          </PromptInputButton>
                        </PromptInputHoverCardTrigger>
                        <PromptInputHoverCardContent class="w-[300px] space-y-4 px-0 py-3">
                          <PromptInputTab>
                            <PromptInputTabLabel>Active Tabs</PromptInputTabLabel>
                            <PromptInputTabBody>
                              <PromptInputTabItem v-for="tab in sampleTabs.active" :key="tab.path">
                                <Globe class="text-primary" :size="16" />
                                <span class="truncate" dir="rtl">
                                  {{ tab.path }}
                                </span>
                              </PromptInputTabItem>
                            </PromptInputTabBody>
                          </PromptInputTab>
                          <PromptInputTab>
                            <PromptInputTabLabel>Recents</PromptInputTabLabel>
                            <PromptInputTabBody>
                              <PromptInputTabItem v-for="tab in sampleTabs.recents" :key="tab.path">
                                <Globe class="text-primary" :size="16" />
                                <span class="truncate" dir="rtl">
                                  {{ tab.path }}
                                </span>
                              </PromptInputTabItem>
                            </PromptInputTabBody>
                          </PromptInputTab>
                          <div class="border-t px-3 pt-2 text-muted-foreground text-xs">
                            Only file paths are included
                          </div>
                        </PromptInputHoverCardContent>
                      </PromptInputHoverCard>
                      <PromptInputAttachments>
                        <template #default="{ file }">
                          <PromptInputAttachment :file="file" />
                        </template>
                      </PromptInputAttachments>
                    </PromptInputHeader>
                    <PromptInputBody>
                      <PromptInputTextarea
                        ref="textareaRef"
                        placeholder="Plan, search, build anything"
                      />
                    </PromptInputBody>
                    <PromptInputFooter>
                      <PromptInputTools>
                        <!-- Model Selector using Select component -->
                        <Select v-model="selectedModel">
                          <SelectTrigger as-child>
                            <PromptInputButton>
                              <component
                                :is="modelOptions.find((m) => m.id === selectedModel)?.icon"
                                :class="`w-4 h-4 mr-2 ${modelOptions.find((m) => m.id === selectedModel)?.color}`"
                              />
                              <span>
                                {{ modelOptions.find((m) => m.id === selectedModel)?.name }}
                              </span>
                            </PromptInputButton>
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem
                              v-for="model in modelOptions"
                              :key="model.id"
                              :value="model.id"
                              @select="
                                () => {
                                  selectedModel = model.id
                                }
                              "
                            >
                              <div class="flex items-center gap-3">
                                <component :is="model.icon" :class="`w-4 h-4 ${model.color}`" />
                                <div>
                                  <div class="font-medium">{{ model.name }}</div>
                                  <div class="text-xs text-muted-foreground">
                                    {{ model.description }}
                                  </div>
                                </div>
                                <Check v-if="selectedModel === model.id" class="ml-auto size-4" />
                              </div>
                            </SelectItem>
                          </SelectContent>
                        </Select>
                      </PromptInputTools>
                      <div class="flex items-center gap-2">
                        <Button size="icon-sm" variant="ghost">
                          <Image class="text-muted-foreground" :size="16" />
                        </Button>
                        <PromptInputSubmit
                          class="!h-8"
                          :status="status === 'loading' ? 'streaming' : 'ready'"
                        />
                      </div>
                    </PromptInputFooter>
                  </PromptInput>
                </PromptInputProvider>
              </div>

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

      <!-- Settings Dialog -->
      <Dialog v-model:open="settingsDialogOpen">
        <DialogContent class="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>Application Settings</DialogTitle>
            <DialogDescription>Manage your application preferences and settings.</DialogDescription>
          </DialogHeader>

          <Tabs v-model="settingsTab" class="w-full">
            <TabsList class="grid w-full grid-cols-3">
              <TabsTrigger value="general">
                <Settings class="w-4 h-4 mr-2" />
                General
              </TabsTrigger>
              <TabsTrigger value="models">
                <Brain class="w-4 h-4 mr-2" />
                AI Models
              </TabsTrigger>
              <TabsTrigger value="appearance">
                <Sun class="w-4 h-4 mr-2" />
                Appearance
              </TabsTrigger>
            </TabsList>

            <TabsContent value="general" class="space-y-4 mt-4">
              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">User Profile</CardTitle>
                  <CardDescription>Manage your profile information</CardDescription>
                </CardHeader>
                <CardContent class="space-y-4">
                  <div class="flex items-center gap-4">
                    <Avatar class="h-16 w-16">
                      <AvatarImage :src="user.avatar" />
                      <AvatarFallback>{{ user.name.charAt(0) }}</AvatarFallback>
                    </Avatar>
                    <div class="flex-1 space-y-1">
                      <label class="text-sm font-medium">Display Name</label>
                      <Input v-model="user.name" placeholder="Your name" />
                    </div>
                  </div>
                  <div class="space-y-1">
                    <label class="text-sm font-medium">Email</label>
                    <Input v-model="user.email" placeholder="Your email" type="email" />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">Preferences</CardTitle>
                  <CardDescription>Customize your experience</CardDescription>
                </CardHeader>
                <CardContent class="space-y-4">
                  <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                      <label class="text-sm font-medium">Enable Notifications</label>
                      <p class="text-xs text-muted-foreground">
                        Receive notifications for new messages
                      </p>
                    </div>
                    <Switch v-model:checked="enableNotifications" />
                  </div>
                  <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                      <label class="text-sm font-medium">Sound Effects</label>
                      <p class="text-xs text-muted-foreground">Play sounds for message events</p>
                    </div>
                    <Switch v-model:checked="enableSoundEffects" />
                  </div>
                  <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                      <label class="text-sm font-medium">Auto-save Conversations</label>
                      <p class="text-xs text-muted-foreground">
                        Automatically save conversation history
                      </p>
                    </div>
                    <Switch v-model:checked="autoSaveConversations" />
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="models" class="space-y-4 mt-4">
              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">Default Model</CardTitle>
                  <CardDescription>
                    Select your preferred AI model for new conversations
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Select v-model="selectedModel">
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="Choose a default model" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem v-for="model in modelOptions" :key="model.id" :value="model.id">
                        <div class="flex items-center gap-3">
                          <component :is="model.icon" :class="`w-5 h-5 ${model.color}`" />
                          <div>
                            <div class="font-medium">{{ model.name }}</div>
                            <div class="text-xs text-muted-foreground">{{ model.description }}</div>
                          </div>
                        </div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </CardContent>
              </Card>

              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">Available Models</CardTitle>
                  <CardDescription>Learn about each AI model's capabilities</CardDescription>
                </CardHeader>
                <CardContent>
                  <Accordion type="single" collapsible class="w-full">
                    <AccordionItem v-for="model in modelOptions" :key="model.id" :value="model.id">
                      <AccordionTrigger>
                        <div class="flex items-center gap-2">
                          <component :is="model.icon" :class="`w-4 h-4 ${model.color}`" />
                          <span>{{ model.name }}</span>
                          <Badge v-if="selectedModel === model.id" variant="secondary" class="ml-2">
                            Default
                          </Badge>
                        </div>
                      </AccordionTrigger>
                      <AccordionContent>
                        <p class="text-sm text-muted-foreground">{{ model.description }}</p>
                      </AccordionContent>
                    </AccordionItem>
                  </Accordion>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="appearance" class="space-y-4 mt-4">
              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">Theme</CardTitle>
                  <CardDescription>Customize the appearance of the application</CardDescription>
                </CardHeader>
                <CardContent class="space-y-4">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <div class="p-2 rounded-lg bg-muted">
                        <Moon v-if="darkMode" class="w-5 h-5" />
                        <Sun v-else class="w-5 h-5" />
                      </div>
                      <div class="space-y-0.5">
                        <label class="text-sm font-medium">Dark Mode</label>
                        <p class="text-xs text-muted-foreground">
                          {{
                            darkMode ? 'Currently using dark theme' : 'Currently using light theme'
                          }}
                        </p>
                      </div>
                    </div>
                    <Switch v-model:checked="darkMode" />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader class="pb-3">
                  <CardTitle class="text-base">Preview</CardTitle>
                  <CardDescription>See how your theme looks</CardDescription>
                </CardHeader>
                <CardContent>
                  <div
                    class="p-4 rounded-lg border"
                    :class="darkMode ? 'bg-slate-900 text-white' : 'bg-white text-slate-900'"
                  >
                    <div class="flex items-center gap-2 mb-2">
                      <Avatar class="h-8 w-8">
                        <AvatarFallback>
                          <Bot class="w-4 h-4" />
                        </AvatarFallback>
                      </Avatar>
                      <span class="font-medium">Sample Message</span>
                    </div>
                    <p class="text-sm opacity-80">
                      This is how your messages will look with the selected theme.
                    </p>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

          <DialogFooter>
            <Button variant="outline" @click="settingsDialogOpen = false">Close</Button>
            <Button @click="settingsDialogOpen = false">Save changes</Button>
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
