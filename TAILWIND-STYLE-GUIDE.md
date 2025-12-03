# Tailwind CSS 样式指南

本文档展示了如何使用 Tailwind CSS v4 来构建现代化的聊天机器人界面。

## ✅ 已完成的 Tailwind CSS 重构

### 🎨 主要样式特性

#### 1. 现代化渐变背景
```vue
<div class="bg-gradient-to-br from-indigo-500 to-purple-600">
```
- 使用 `bg-gradient-to-br` 创建从左上到右下的渐变
- `from-indigo-500` 起始颜色为靛蓝色
- `to-purple-600` 结束颜色为紫色

#### 2. 玻璃态效果 (Glassmorphism)
```vue
<header class="bg-white/95 backdrop-blur-lg border-b border-white/20">
```
- `bg-white/95` - 95% 不透明白色背景
- `backdrop-blur-lg` - 强烈的背景模糊效果
- `border-b border-white/20` - 底部半透明白色边框

#### 3. 响应式布局
```vue
<div class="flex flex-col h-screen">                    <!-- 全屏垂直布局 -->
<div class="flex-1 overflow-hidden">                   <!-- 自适应高度聊天区域 -->
<div class="max-w-screen-2xl mx-auto">                 <!-- 最大宽度限制 -->
<div class="max-w-[90%]">                              <!-- 百分比宽度 -->
```

#### 4. 现代化阴影效果
```vue
<div class="shadow-2xl">                    <!-- 最强阴影 -->
<div class="shadow-lg">                     <!-- 大阴影 -->
<div class="focus-within:shadow-lg">         <!-- 聚焦时阴影 -->
<div class="focus-within:shadow-indigo-500/10"> <!-- 带颜色的阴影 -->
```

#### 5. 交互状态样式
```vue
<button class="transition-all focus-within:border-indigo-500 focus-within:shadow-lg">
<span class="animate-pulse">                                   <!-- 脉冲动画 -->
<div class="hover:bg-green-100 transition-colors">            <!-- 悬停效果 -->
```

#### 6. 现代化间距系统
```vue
<div class="px-8 py-4">      <!-- 水平 32px，垂直 16px -->
<div class="m-4">            <!-- 外边距 16px -->
<div class="gap-4">          <!-- 间隙 16px -->
<div class="max-w-screen-2xl mx-auto">  <!-- 响应式容器 -->
```

#### 7. 现代化圆角设计
```vue
<div class="rounded-2xl">     <!-- 超大圆角 -->
<div class="rounded-full">    <!-- 完全圆角 -->
```

### 🎯 组件样式示例

#### 头部组件
```vue
<header class="bg-white/95 backdrop-blur-lg border-b border-white/20 shadow-lg px-8 py-4">
  <div class="max-w-screen-2xl mx-auto">
    <div class="flex items-center gap-4">
      <Bot class="w-10 h-10 text-indigo-500" />
      <h1 class="text-3xl font-bold text-gray-900 m-0">Iroh Chat</h1>
      <!-- 状态指示器 -->
      <div class="ml-auto flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium">
        <span class="w-2 h-2 rounded-full animate-pulse"></span>
        <span>Online</span>
      </div>
    </div>
  </div>
</header>
```

#### 聊天容器
```vue
<div class="flex-1 overflow-hidden m-4 bg-white/95 backdrop-blur-lg rounded-2xl shadow-2xl">
  <Conversation class="h-full">
    <!-- AI Elements 组件 -->
  </Conversation>
</div>
```

#### 输入区域
```vue
<div class="bg-white/95 backdrop-blur-lg border-t border-white/20 px-8 py-4">
  <form class="max-w-screen-2xl mx-auto">
    <PromptInput class="relative bg-white rounded-2xl border-2 border-gray-200 transition-all focus-within:border-indigo-500">
      <PromptInputTextarea
        class="px-6 pr-16 py-4 border-0 resize-none text-base leading-6 max-h-32 focus:ring-0"
        placeholder="Type your message here..."
      />
      <PromptInputSubmit class="absolute bottom-3 right-3" />
    </PromptInput>
  </form>
</div>
```

#### 底部组件
```vue
<footer class="bg-gray-900/95 backdrop-blur-lg border-t border-white/10 px-8 py-4">
  <div class="max-w-screen-2xl mx-auto text-center">
    <p class="text-gray-400 text-sm m-0">
      Powered by AI Elements Vue • Built with Tauri + Vue 3
    </p>
  </div>
</footer>
```

### 🛠️ 配置文件

#### Tailwind 配置 (`tailwind.config.js`)
```javascript
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'sans-serif'],
      },
      colors: {
        // 与 shadcn-vue 兼容的颜色系统
        border: "hsl(var(--border))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        // ... 其他颜色
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },
    },
  },
  darkMode: 'class',
}
```

### 📱 响应式设计

虽然主界面使用固定布局以提供最佳桌面体验，但 Tailwind CSS 提供了完整的响应式支持：

```vue
<!-- 响应式间距 -->
<div class="px-4 py-2 md:px-8 md:py-4 lg:px-12 lg:py-6">

<!-- 响应式字体 -->
<h1 class="text-2xl md:text-3xl lg:text-4xl">

<!-- 响应式显示 -->
<div class="hidden md:block lg:hidden">
```

### 🎨 颜色系统

#### 主要颜色
- **主色调**: `indigo-500` / `indigo-600`
- **成功色**: `green-500` / `green-100`
- **错误色**: `red-500` / `red-50`
- **文本色**: `gray-900` / `gray-500` / `gray-400`

#### 透明度变体
- `white/95` - 95% 不透明白色
- `gray-900/95` - 95% 不透明深灰
- `indigo-500/10` - 10% 不透明靛蓝

### ✨ 动画效果

#### 内置动画
- `animate-pulse` - 脉冲效果
- `transition-all` - 所有属性过渡
- `transition-colors` - 颜色过渡
- `duration-200` - 动画持续时间

### 🚀 性能优化

1. **按需加载**: Tailwind v4 自动移除未使用的样式
2. **CSS 变量**: 与 shadcn-vue 的设计系统集成
3. **优化构建**: 生产构建时自动压缩 CSS

### 📁 文件结构

```
src/
├── App.vue              # 主组件 - 展示 Tailwind CSS 应用
├── style.css            # Tailwind CSS 入口
├── lib/
│   └── utils.ts         # 样式工具函数
├── components/
│   ├── ai-elements/     # AI Elements 组件
│   └── ui/              # shadcn-vue 组件
└── tailwind.config.js   # Tailwind 配置
```

### 🎯 最佳实践

1. **使用原子类**: 避免创建自定义 CSS 类
2. **保持一致性**: 使用 Tailwind 的间距和颜色系统
3. **响应式优先**: 移动端优先的设计方法
4. **性能考虑**: 避免过度使用复杂的工具类组合
5. **可读性**: 合理使用换行和缩进保持模板清晰

这个样式系统展示了如何利用 Tailwind CSS v4 的强大功能来创建现代化、美观且高性能的用户界面。