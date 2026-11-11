import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  BRAND_AND_STATUS_COLOR_TOKENS,
  CHANNEL_TYPES,
  MOTION_TOKENS,
  RADIUS_TOKENS,
  SHADOW_TOKENS,
  SIZING_TOKENS,
  SPACING_TOKENS,
  SURFACE_AND_CONTROL_COLOR_TOKENS,
} from '../app/utils/designTokens'

const mainCssContent = fs.readFileSync(
  path.resolve(process.cwd(), 'app/assets/css/main.css'),
  'utf8',
)

describe('design token system contract', () => {
  it('defines all required Midnight Indigo spacing steps from 0 to 24 in CSS', () => {
    const requiredSteps = [0, 1, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24]
    expect(SPACING_TOKENS.map((item) => item.step)).toEqual(requiredSteps)

    for (const token of SPACING_TOKENS) {
      expect(mainCssContent).toContain(`${token.token}:`)
    }
  })

  it.each([
    ...RADIUS_TOKENS.map((item) => ({ group: 'radius', token: item.token, value: item.value })),
    ...SHADOW_TOKENS.map((item) => ({ group: 'shadow', token: item.token, value: undefined })),
    ...SIZING_TOKENS.map((item) => ({ group: 'sizing', token: item.token, value: item.value })),
    ...MOTION_TOKENS.map((item) => ({ group: 'motion', token: item.token, value: item.value })),
  ])('declares $group token $token in main.css', ({ token, value }) => {
    expect(mainCssContent).toContain(`${token}:`)
    if (value) {
      expect(mainCssContent).toContain(`${token}: ${value}`)
    }
  })

  it('declares light (Clean Cool White) and dark (Midnight Navy) blocks with official HEX palette and channel tokens', () => {
    const normalizedCss = mainCssContent.toLowerCase()
    const [lightSection, darkSection] = normalizedCss.split('.dark {')
    expect(lightSection).toBeDefined()
    expect(darkSection).toBeDefined()

    expect(lightSection).toContain('--nt-background: #f7f8fc')
    expect(lightSection).toContain('--nt-sidebar: #f0f2f8')
    expect(lightSection).toContain('--ui-primary: #4f46e5')
    expect(lightSection).toContain('--nt-accent: #0891b2')

    expect(darkSection).toContain('--nt-background: #090d18')
    expect(darkSection).toContain('--nt-sidebar: #0d1423')
    expect(darkSection).toContain('--ui-primary: #818cf8')
    expect(darkSection).toContain('--nt-accent: #22d3ee')

    const expectedChannels: Record<string, string> = {
      email: '#3b82f6',
      sms: '#10b981',
      push: '#8b5cf6',
      webhook: '#d97706',
    }

    for (const channel of CHANNEL_TYPES) {
      expect(lightSection).toContain(`--nt-channel-${channel}: ${expectedChannels[channel]}`)
      expect(lightSection).toContain(`--nt-channel-${channel}-bg:`)
      expect(darkSection).toContain(`--nt-channel-${channel}-bg:`)
    }
  })

  it('covers all semantic brand, status, and surface color definitions', () => {
    expect(BRAND_AND_STATUS_COLOR_TOKENS.length).toBeGreaterThanOrEqual(9)
    expect(SURFACE_AND_CONTROL_COLOR_TOKENS.length).toBeGreaterThanOrEqual(12)
  })
})
