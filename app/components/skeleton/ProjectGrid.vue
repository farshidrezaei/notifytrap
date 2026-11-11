<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number
  }>(),
  {
    count: 4,
  },
)
</script>

<template>
  <div class="grid gap-4 sm:grid-cols-2">
    <UCard v-for="index in count" :key="index">
      <div class="flex flex-col gap-3">
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0 space-y-1.5">
            <div class="flex flex-wrap items-center gap-2">
              <USkeleton class="h-6 w-36 rounded" />
              <USkeleton class="h-4 w-12 rounded-full" />
            </div>
            <USkeleton class="h-4 w-24 rounded" />
          </div>

          <div class="flex items-center gap-1">
            <USkeleton class="size-6 rounded-md" />
            <USkeleton class="size-6 rounded-md" />
          </div>
        </div>

        <USkeleton class="h-4 w-5/6 rounded" />

        <div class="flex items-center justify-between border-t border-default pt-3">
          <USkeleton class="h-3.5 w-24 rounded" />
          <USkeleton class="h-6 w-28 rounded-md" />
        </div>
      </div>
    </UCard>
  </div>
</template>
