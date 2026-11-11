<template>
  <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
    <UCard v-for="index in 4" :key="index">
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2.5">
          <USkeleton class="size-9 shrink-0 rounded-lg" />
          <div class="space-y-1.5">
            <USkeleton class="h-3.5 w-14 rounded" />
            <USkeleton class="h-6 w-8 rounded" />
          </div>
        </div>
        <USkeleton class="size-6 shrink-0 rounded-md" />
      </div>
    </UCard>
  </div>
</template>
