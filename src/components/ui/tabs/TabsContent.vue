<script setup lang="ts">
  import type { HTMLAttributes } from 'vue'
  import { TabsContent } from 'reka-ui'
  import { cn } from '@/lib/utils'

  const props = defineProps<{
    class?: HTMLAttributes['class']
    value: string
  }>()
</script>

<template>
  <TabsContent
    :class="
      cn(
        'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        props.class
      )
    "
    :value="value"
  >
    <slot />
  </TabsContent>
</template>
