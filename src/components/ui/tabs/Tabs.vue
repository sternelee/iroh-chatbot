<script setup lang="ts">
  import type { HTMLAttributes } from 'vue'
  import { TabsRoot } from 'reka-ui'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    class?: HTMLAttributes['class']
    defaultValue?: string
    modelValue?: string
    orientation?: 'horizontal' | 'vertical'
  }>()

  const emit = defineEmits<{
    (e: 'update:modelValue', value: string): void
  }>()
</script>

<template>
  <TabsRoot
    :class="cn(props.class)"
    :default-value="defaultValue"
    :model-value="modelValue"
    :orientation="orientation || 'horizontal'"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <slot />
  </TabsRoot>
</template>
