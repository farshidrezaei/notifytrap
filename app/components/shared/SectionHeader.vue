<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div class="min-w-0">
      <h2 class="text-base font-semibold text-highlighted">
        {{ title }}
      </h2>
      <p v-if="description" class="mt-0.5 text-xs text-muted">
        {{ description }}
      </p>
    </div>
    <slot name="actions" />
  </div>
</template>
