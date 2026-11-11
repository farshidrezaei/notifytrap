<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number
  }>(),
  {
    count: 6,
  },
)
</script>

<template>
  <div class="flex-1 divide-y divide-default overflow-hidden">
    <div
      v-for="index in count"
      :key="index"
      class="flex flex-col gap-1.5 p-3"
    >
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <USkeleton class="size-2 rounded-full" />
          <USkeleton class="h-5 w-14 rounded-full" />
        </div>
        <USkeleton class="h-3.5 w-14 rounded" />
      </div>
      <USkeleton class="h-4 w-3/4 rounded" />
      <USkeleton class="h-4 w-1/2 rounded" />
      <USkeleton class="h-3.5 w-11/12 rounded" />
    </div>
  </div>
</template>
