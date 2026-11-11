<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number
  }>(),
  {
    count: 3,
  },
)
</script>

<template>
  <div class="space-y-4">
    <UCard v-for="index in count" :key="index">
      <div class="flex flex-col gap-4">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0 space-y-1.5">
            <div class="flex flex-wrap items-center gap-2">
              <USkeleton class="h-5 w-16 rounded-full" />
              <USkeleton class="h-5 w-36 rounded" />
              <USkeleton class="h-4 w-14 rounded-full" />
            </div>
            <USkeleton class="h-4 w-28 rounded" />
          </div>

          <div class="flex items-center gap-2">
            <USkeleton class="h-5 w-9 rounded-full" />
            <USkeleton class="size-6 rounded-md" />
            <USkeleton class="size-6 rounded-md" />
          </div>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <div
            v-for="fieldIndex in 2"
            :key="fieldIndex"
            class="rounded-md border border-default bg-muted/40 p-2.5"
          >
            <div class="mb-1.5 flex items-center justify-between gap-2">
              <USkeleton class="h-4 w-28 rounded" />
              <USkeleton class="size-6 rounded-md" />
            </div>
            <USkeleton class="h-4 w-44 rounded" />
          </div>
        </div>

        <div class="flex items-center justify-between border-t border-default/70 pt-3">
          <USkeleton class="h-6 w-32 rounded-md" />
        </div>
      </div>
    </UCard>
  </div>
</template>
