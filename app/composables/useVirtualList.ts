import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'

export interface UseVirtualListOptions {
  itemHeight: number
  overscan?: number
}

export interface VirtualItem<T> {
  data: T
  index: number
}

/**
 * Lightweight, zero-dependency virtual scrolling composable.
 * Keeps DOM footprint small and scrolling at 60fps even with thousands of items.
 */
export function useVirtualList<T>(
  items: Ref<readonly T[]>,
  options: UseVirtualListOptions,
) {
  const containerRef = ref<HTMLElement | null>(null)
  const scrollTop = ref(0)
  const containerHeight = ref(400)
  const overscan = options.overscan ?? 5
  const itemHeight = options.itemHeight

  let animationFrameId: number | null = null

  function handleScroll(): void {
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId)
    }
    animationFrameId = requestAnimationFrame(() => {
      if (containerRef.value) {
        scrollTop.value = containerRef.value.scrollTop
        containerHeight.value = containerRef.value.clientHeight || 400
      }
      animationFrameId = null
    })
  }

  const totalCount = computed(() => items.value.length)
  const totalHeight = computed(() => totalCount.value * itemHeight)

  const startIndex = computed(() => {
    return Math.max(0, Math.floor(scrollTop.value / itemHeight) - overscan)
  })

  const endIndex = computed(() => {
    return Math.min(
      totalCount.value,
      Math.ceil((scrollTop.value + containerHeight.value) / itemHeight) + overscan,
    )
  })

  const offsetY = computed(() => startIndex.value * itemHeight)

  const visibleItems = computed<VirtualItem<T>[]>(() => {
    const list: VirtualItem<T>[] = []
    const start = startIndex.value
    const end = endIndex.value
    const all = items.value

    for (let index = start; index < end; index += 1) {
      const data = all[index]
      if (data !== undefined) {
        list.push({ data, index })
      }
    }
    return list
  })

  function scrollToIndex(index: number): void {
    if (!containerRef.value) {
      return
    }
    const targetTop = index * itemHeight
    const currentScrollTop = containerRef.value.scrollTop
    const viewHeight = containerRef.value.clientHeight || 400

    if (targetTop < currentScrollTop) {
      containerRef.value.scrollTop = targetTop
    } else if (targetTop + itemHeight > currentScrollTop + viewHeight) {
      containerRef.value.scrollTop = targetTop + itemHeight - viewHeight
    }
  }

  onMounted(() => {
    if (containerRef.value) {
      containerHeight.value = containerRef.value.clientHeight || 400
      containerRef.value.addEventListener('scroll', handleScroll, { passive: true })
    }
  })

  onBeforeUnmount(() => {
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId)
    }
    if (containerRef.value) {
      containerRef.value.removeEventListener('scroll', handleScroll)
    }
  })

  // Recalculate container height when items change or window resizes
  watch(totalCount, () => {
    if (containerRef.value) {
      containerHeight.value = containerRef.value.clientHeight || 400
    }
  })

  return {
    containerRef,
    visibleItems,
    totalHeight,
    offsetY,
    startIndex,
    endIndex,
    scrollToIndex,
  }
}
