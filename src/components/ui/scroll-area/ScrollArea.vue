<script setup lang="ts">
  import type { HTMLAttributes } from 'vue'
  import {
    ScrollAreaRoot,
    ScrollAreaScrollbar,
    ScrollAreaThumb,
    ScrollAreaViewport,
    ScrollAreaCorner,
  } from 'reka-ui'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    class?: HTMLAttributes['class']
    orientation?: 'vertical' | 'horizontal'
  }>()
</script>

<template>
  <ScrollAreaRoot :class="cn('relative overflow-hidden', props.class)">
    <ScrollAreaViewport class="h-full w-full rounded-[inherit]">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar
      :orientation="orientation || 'vertical'"
      class="flex touch-none select-none transition-colors"
      :class="{
        'h-full w-2.5 border-l border-l-transparent p-[1px]': orientation !== 'horizontal',
        'h-2.5 flex-col border-t border-t-transparent p-[1px]': orientation === 'horizontal',
      }"
    >
      <ScrollAreaThumb class="relative flex-1 rounded-full bg-border" />
    </ScrollAreaScrollbar>
    <ScrollAreaCorner />
  </ScrollAreaRoot>
</template>
