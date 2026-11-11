<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-between gap-2">
        <USkeleton class="h-5 w-40 rounded" />
        <USkeleton class="h-6 w-24 rounded-md" />
      </div>
    </template>

    <div class="space-y-3">
      <div class="flex items-center gap-2">
        <USkeleton class="h-5 w-16 rounded-full" />
        <USkeleton class="h-4 w-32 rounded" />
      </div>
      <div class="overflow-hidden rounded-lg border border-default bg-muted/40">
        <div class="flex items-center justify-between gap-2 border-b border-default/80 bg-muted/70 px-3 py-1.5">
          <div class="flex items-center gap-2">
            <USkeleton class="size-3.5 rounded" />
            <USkeleton class="h-4 w-12 rounded-full" />
          </div>
        </div>
        <div class="space-y-2 p-3">
          <USkeleton class="h-3.5 w-4/5 rounded" />
          <USkeleton class="h-3.5 w-full rounded" />
          <USkeleton class="h-3.5 w-3/4 rounded" />
          <USkeleton class="h-3.5 w-2/3 rounded" />
        </div>
      </div>
    </div>
  </UCard>
</template>
