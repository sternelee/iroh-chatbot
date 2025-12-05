<script setup lang="ts">
  import type { HTMLAttributes } from 'vue'
  import { AccordionRoot } from 'reka-ui'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    class?: HTMLAttributes['class']
    type?: 'single' | 'multiple'
    collapsible?: boolean
    defaultValue?: string | string[]
    modelValue?: string | string[]
    disabled?: boolean
  }>()

  const emit = defineEmits<{
    (e: 'update:modelValue', value: string | string[]): void
  }>()
</script>

<template>
  <AccordionRoot
    :class="cn(props.class)"
    :type="type || 'single'"
    :collapsible="collapsible"
    :default-value="defaultValue"
    :model-value="modelValue"
    :disabled="disabled"
    @update:model-value="
      (v) => v !== undefined && v !== null && emit('update:modelValue', v)
    "
  >
    <slot />
  </AccordionRoot>
</template>
