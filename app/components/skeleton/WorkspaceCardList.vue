<script setup lang="ts">
withDefaults(
  defineProps<{
    count?: number
  }>(),
  {
    count: 2,
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
              <USkeleton class="h-6 w-36 rounded" />
              <USkeleton class="h-5 w-20 rounded" />
              <USkeleton class="h-4 w-12 rounded-full" />
            </div>
            <USkeleton class="h-4 w-56 rounded" />
            <USkeleton class="h-3.5 w-32 rounded" />
          </div>

          <div class="flex items-center gap-1.5">
            <USkeleton class="h-6 w-24 rounded-md" />
            <USkeleton class="size-6 rounded-md" />
            <USkeleton class="size-6 rounded-md" />
          </div>
        </div>

        <div class="border-t border-default pt-3">
          <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            <div
              v-for="projectIndex in 3"
              :key="projectIndex"
              class="flex items-center justify-between gap-2 rounded-lg border border-default px-3 py-2"
            >
              <div class="space-y-1.5">
                <USkeleton class="h-4 w-24 rounded" />
                <USkeleton class="h-3.5 w-20 rounded" />
              </div>
              <USkeleton class="size-3.5 shrink-0 rounded" />
            </div>
          </div>
        </div>
      </div>
    </UCard>
  </div>
</template>
