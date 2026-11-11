<template>
  <div class="mt-5 flex flex-col gap-4 border-t border-default pt-5">
    <div
      v-for="rowIndex in 2"
      :key="rowIndex"
      class="grid grid-cols-1 gap-4 sm:grid-cols-3"
    >
      <div v-for="columnIndex in 3" :key="columnIndex" class="space-y-1.5">
        <div class="flex items-center gap-1.5">
          <USkeleton class="h-5 w-28 rounded" />
          <USkeleton v-if="rowIndex === 1 && columnIndex === 1" class="size-4 rounded-full" />
        </div>
        <USkeleton class="h-8 w-full rounded-md" />
      </div>
    </div>

    <div class="flex flex-wrap items-center justify-end gap-2 pt-2">
      <USkeleton class="h-8 w-16 rounded-md" />
      <USkeleton class="h-8 w-36 rounded-md" />
    </div>
  </div>
</template>
