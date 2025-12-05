<script setup lang="ts">
  import type { HTMLAttributes } from 'vue'
  import { AccordionItem } from 'reka-ui'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    class?: HTMLAttributes['class']
    value: string
    disabled?: boolean
  }>()
</script>

<template>
  <AccordionItem :class="cn('border-b', props.class)" :value="value" :disabled="disabled">
    <slot />
  </AccordionItem>
</template>
