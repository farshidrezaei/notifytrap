<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden p-4">
    <div class="mb-4 flex flex-wrap items-start justify-between gap-3 border-b border-default pb-3">
      <div class="min-w-0 space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <USkeleton class="h-5 w-14 rounded-full" />
          <USkeleton class="h-4 w-14 rounded-full" />
          <USkeleton class="h-4 w-20 rounded" />
        </div>
        <USkeleton class="h-6 w-64 max-w-full rounded" />
      </div>

      <div class="flex items-center gap-1.5">
        <USkeleton class="h-6 w-24 rounded-md" />
        <USkeleton class="h-6 w-20 rounded-md" />
        <USkeleton class="size-6 rounded-md" />
      </div>
    </div>

    <div class="space-y-2 rounded-lg border border-default bg-muted/30 p-3">
      <div class="flex items-center gap-2">
        <USkeleton class="h-4 w-16 rounded" />
        <USkeleton class="h-4 w-48 rounded" />
      </div>
      <div class="flex items-center gap-2">
        <USkeleton class="h-4 w-16 rounded" />
        <USkeleton class="h-4 w-56 rounded" />
      </div>
    </div>

    <div class="mt-4 flex items-center gap-2 border-b border-default pb-2">
      <USkeleton class="h-6 w-20 rounded-md" />
      <USkeleton class="h-6 w-20 rounded-md" />
      <USkeleton class="h-6 w-20 rounded-md" />
    </div>

    <div class="mt-4 h-[420px] w-full space-y-3 rounded-lg border border-default bg-muted/20 p-4">
      <USkeleton class="h-5 w-1/2 rounded" />
      <USkeleton class="h-4 w-full rounded" />
      <USkeleton class="h-4 w-5/6 rounded" />
      <USkeleton class="h-32 w-full rounded-md" />
      <USkeleton class="h-4 w-2/3 rounded" />
    </div>
  </div>
</template>
