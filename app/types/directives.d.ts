import type { MaskInputOptions } from 'maska'
import type { Directive } from 'vue'

declare module 'vue' {
  export interface ComponentCustomProperties {
    vMaska: Directive<HTMLElement, MaskInputOptions | string | undefined>
  }

  export interface GlobalDirectives {
    vMaska: Directive<HTMLElement, MaskInputOptions | string | undefined>
  }
}

export {}
