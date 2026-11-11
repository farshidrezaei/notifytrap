<template>
  <div class="flex flex-col gap-6">
    <!-- Section header skeleton matching SharedSectionHeader + 2 sm buttons -->
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div class="space-y-1.5">
        <USkeleton class="h-6 w-48 rounded" />
        <USkeleton class="h-4 w-72 rounded" />
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <USkeleton class="h-7 w-28 rounded-md" />
        <USkeleton class="h-7 w-28 rounded-md" />
      </div>
    </div>

    <!-- Capture Servers & Database Status (3 cards) -->
    <div class="grid gap-4 md:grid-cols-3">
      <UCard v-for="index in 3" :key="index">
        <div class="flex items-start justify-between gap-2">
          <div class="space-y-2">
            <USkeleton class="h-4 w-28 rounded" />
            <USkeleton class="h-5 w-32 rounded" />
          </div>
          <div class="flex items-center gap-1">
            <USkeleton class="h-4 w-14 rounded-full" />
            <USkeleton v-if="index < 3" class="size-6 rounded-md" />
          </div>
        </div>
      </UCard>
    </div>

    <!-- Scoped Channel Matrix (4 cards) -->
    <SkeletonDashboardChannelCards />

    <!-- Quick Snippet & Recent Activity (2 cards) -->
    <div class="grid gap-6 lg:grid-cols-2">
      <SkeletonDashboardSnippetCard />
      <SkeletonDashboardRecentCard />
    </div>
  </div>
</template>
