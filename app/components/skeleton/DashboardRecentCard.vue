<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between gap-2">
        <USkeleton class="h-5 w-40 rounded" />
        <USkeleton class="h-6 w-16 rounded-md" />
      </div>
    </template>

    <div class="divide-y divide-default">
      <div
        v-for="index in 5"
        :key="index"
        class="flex items-center justify-between gap-3 py-2.5"
      >
        <div class="min-w-0 flex-1 space-y-1.5">
          <div class="flex items-center gap-2">
            <USkeleton class="h-5 w-14 shrink-0 rounded-full" />
            <USkeleton class="h-4 w-44 rounded" />
          </div>
          <USkeleton class="h-3.5 w-56 rounded" />
        </div>
        <USkeleton class="h-3.5 w-14 shrink-0 rounded" />
      </div>
    </div>
  </UCard>
</template>
