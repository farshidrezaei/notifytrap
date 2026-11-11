<template>
  <div class="flex min-h-0 flex-1 flex-col gap-4">
    <!-- Filter bar skeleton matching InboxFilterBar dimensions -->
    <div class="flex flex-col gap-3 rounded-lg border border-default bg-muted/30 p-3">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <div class="flex flex-wrap items-center gap-1 pt-0.5">
          <USkeleton v-for="index in 5" :key="index" class="h-6 w-14 rounded-md" />
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <USkeleton class="h-6 w-36 rounded-md" />
          <USkeleton class="h-6 w-56 rounded-md" />
          <USkeleton class="size-6 rounded-md" />
          <USkeleton class="size-6 rounded-md" />
        </div>
      </div>
    </div>

    <!-- Split list + inspector grid matching pages/inbox.vue -->
    <div class="grid min-h-0 flex-1 gap-4 lg:grid-cols-12">
      <div class="min-h-[320px] lg:col-span-5 lg:min-h-0">
        <div class="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border border-default bg-default/50">
          <SkeletonInboxMessageList />
        </div>
      </div>
      <div class="min-h-[380px] lg:col-span-7 lg:min-h-0">
        <div class="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border border-default bg-default/50">
          <SkeletonInboxInspector />
        </div>
      </div>
    </div>
  </div>
</template>
