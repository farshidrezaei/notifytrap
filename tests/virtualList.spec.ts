import { ref } from 'vue'
import { describe, expect, it } from 'vitest'
import { useVirtualList } from '../app/composables/useVirtualList'

describe('useVirtualList composable', () => {
  it('correctly calculates total height and empty visible items for empty list', () => {
    const items = ref<string[]>([])
    const { totalHeight, visibleItems, startIndex, endIndex } = useVirtualList(items, {
      itemHeight: 80,
      overscan: 2,
    })

    expect(totalHeight.value).toBe(0)
    expect(visibleItems.value).toHaveLength(0)
    expect(startIndex.value).toBe(0)
    expect(endIndex.value).toBe(0)
  })

  it('virtualizes large item list and restricts visible items to bounded slice', () => {
    const sampleItems = Array.from({ length: 1000 }, (_, index) => `Item #${index + 1}`)
    const items = ref(sampleItems)

    const { totalHeight, visibleItems, startIndex, endIndex } = useVirtualList(items, {
      itemHeight: 50,
      overscan: 5,
    })

    // 1000 items * 50px = 50,000px total virtual height
    expect(totalHeight.value).toBe(50_000)

    // With default initial height 400px and scrollTop 0:
    // startIndex = 0, endIndex = min(1000, ceil(400/50) + 5) = 8 + 5 = 13 items
    expect(startIndex.value).toBe(0)
    expect(endIndex.value).toBe(13)
    expect(visibleItems.value).toHaveLength(13)
    expect(visibleItems.value[0]?.data).toBe('Item #1')
    expect(visibleItems.value[12]?.data).toBe('Item #13')
  })

  it('handles item updates reactively', () => {
    const items = ref(['Alpha', 'Beta', 'Gamma'])
    const { totalHeight, visibleItems } = useVirtualList(items, {
      itemHeight: 60,
    })

    expect(totalHeight.value).toBe(180)
    expect(visibleItems.value).toHaveLength(3)

    items.value = ['Alpha', 'Beta', 'Gamma', 'Delta', 'Epsilon']
    expect(totalHeight.value).toBe(300)
    expect(visibleItems.value).toHaveLength(5)
  })

  it('handles scrollToIndex safely when container is not yet attached', () => {
    const items = ref(['A', 'B', 'C'])
    const { scrollToIndex } = useVirtualList(items, { itemHeight: 50 })
    expect(() => scrollToIndex(2)).not.toThrow()
  })

  it('adjusts scrollTop correctly when scrolling forward and backward to index', () => {
    const items = ref(Array.from({ length: 50 }, (_, i) => `Entry ${i}`))
    const { containerRef, scrollToIndex } = useVirtualList(items, { itemHeight: 40 })

    const mockContainer = document.createElement('div')
    Object.defineProperty(mockContainer, 'clientHeight', { value: 200, configurable: true })
    mockContainer.scrollTop = 500
    containerRef.value = mockContainer

    // Scroll up to index 2 (targetTop: 80 < current 500)
    scrollToIndex(2)
    expect(mockContainer.scrollTop).toBe(80)

    // Scroll down to index 20 (targetTop: 800 + 40 = 840 > current 80 + 200 = 280)
    // targetTop + itemHeight - viewHeight = 800 + 40 - 200 = 640
    scrollToIndex(20)
    expect(mockContainer.scrollTop).toBe(640)
  })
})
