export type ChannelKind = 'email' | 'sms' | 'push' | 'webhook'

export interface ColorTokenDefinition {
  id: string
  token: string
  surfaceStyle: Record<string, string>
}

export interface TypographyScaleDefinition {
  id: string
  className: string
  fontSize: string
  lineHeight: string
  fontWeight: string
  letterSpacing: string
}

export interface SpacingTokenDefinition {
  step: number
  token: string
  pixelValue: number
}

export interface RadiusTokenDefinition {
  id: string
  token: string
  value: string
}

export interface ShadowTokenDefinition {
  id: string
  token: string
}

export interface SizingTokenDefinition {
  id: string
  token: string
  value: string
}

export interface MotionTokenDefinition {
  id: string
  token: string
  value: string
}

export const BRAND_AND_STATUS_COLOR_TOKENS: ColorTokenDefinition[] = [
  { id: 'primary', token: '--ui-primary', surfaceStyle: { backgroundColor: 'var(--ui-primary)' } },
  { id: 'secondary', token: '--ui-secondary', surfaceStyle: { backgroundColor: 'var(--ui-secondary)' } },
  { id: 'accent', token: '--nt-accent', surfaceStyle: { backgroundColor: 'var(--nt-accent)' } },
  { id: 'success', token: '--ui-success', surfaceStyle: { backgroundColor: 'var(--ui-success)' } },
  { id: 'warning', token: '--ui-warning', surfaceStyle: { backgroundColor: 'var(--ui-warning)' } },
  { id: 'error', token: '--ui-error', surfaceStyle: { backgroundColor: 'var(--ui-error)' } },
  { id: 'information', token: '--ui-info', surfaceStyle: { backgroundColor: 'var(--ui-info)' } },
  { id: 'pending', token: '--nt-status-pending', surfaceStyle: { backgroundColor: 'var(--nt-status-pending)' } },
  { id: 'live', token: '--nt-status-live', surfaceStyle: { backgroundColor: 'var(--nt-status-live)' } },
]

export const SURFACE_AND_CONTROL_COLOR_TOKENS: ColorTokenDefinition[] = [
  { id: 'background', token: '--nt-background', surfaceStyle: { backgroundColor: 'var(--nt-background)' } },
  { id: 'sidebar', token: '--nt-sidebar', surfaceStyle: { backgroundColor: 'var(--nt-sidebar)' } },
  { id: 'foreground', token: '--ui-text-highlighted', surfaceStyle: { backgroundColor: 'var(--ui-text-highlighted)' } },
  { id: 'surface', token: '--nt-surface', surfaceStyle: { backgroundColor: 'var(--nt-surface)' } },
  { id: 'elevatedSurface', token: '--nt-surface-elevated', surfaceStyle: { backgroundColor: 'var(--nt-surface-elevated)' } },
  { id: 'hoverSurface', token: '--nt-surface-hover', surfaceStyle: { backgroundColor: 'var(--nt-surface-hover)' } },
  { id: 'activeSurface', token: '--nt-surface-active', surfaceStyle: { backgroundColor: 'var(--nt-surface-active)' } },
  { id: 'border', token: '--ui-border', surfaceStyle: { backgroundColor: 'var(--ui-border)' } },
  { id: 'input', token: '--nt-input-bg', surfaceStyle: { backgroundColor: 'var(--nt-input-bg)', border: '1px solid var(--nt-input-border)' } },
  { id: 'focus', token: '--nt-border-focus', surfaceStyle: { backgroundColor: 'var(--nt-surface)', boxShadow: 'var(--nt-shadow-focus)' } },
  { id: 'disabled', token: '--nt-disabled-bg', surfaceStyle: { backgroundColor: 'var(--nt-disabled-bg)', color: 'var(--nt-disabled-text)' } },
  { id: 'overlay', token: '--nt-overlay', surfaceStyle: { backgroundColor: 'var(--nt-overlay)' } },
]

export const CHANNEL_TYPES: ChannelKind[] = ['email', 'sms', 'push', 'webhook']

export const TYPOGRAPHY_SCALE: TypographyScaleDefinition[] = [
  {
    id: 'display',
    className: 'text-2xl font-semibold tracking-tight text-highlighted',
    fontSize: '1.5rem (24px)',
    lineHeight: '2rem',
    fontWeight: '600',
    letterSpacing: '-0.02em',
  },
  {
    id: 'heading',
    className: 'text-xl font-semibold tracking-tight text-highlighted',
    fontSize: '1.25rem (20px)',
    lineHeight: '1.75rem',
    fontWeight: '600',
    letterSpacing: '-0.015em',
  },
  {
    id: 'section',
    className: 'text-base font-medium text-highlighted',
    fontSize: '1rem (16px)',
    lineHeight: '1.5rem',
    fontWeight: '500',
    letterSpacing: '0',
  },
  {
    id: 'body',
    className: 'text-sm font-normal text-default',
    fontSize: '0.875rem (14px)',
    lineHeight: '1.375rem',
    fontWeight: '400',
    letterSpacing: '0',
  },
  {
    id: 'label',
    className: 'text-[0.8125rem] font-medium text-toned',
    fontSize: '0.8125rem (13px)',
    lineHeight: '1.125rem',
    fontWeight: '500',
    letterSpacing: '0.005em',
  },
  {
    id: 'caption',
    className: 'text-xs font-normal text-muted',
    fontSize: '0.75rem (12px)',
    lineHeight: '1rem',
    fontWeight: '400',
    letterSpacing: '0.01em',
  },
]

export const SPACING_TOKENS: SpacingTokenDefinition[] = [0, 1, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24].map(
  (step) => ({
    step,
    token: `--nt-space-${step}`,
    pixelValue: step * 4,
  }),
)

export const RADIUS_TOKENS: RadiusTokenDefinition[] = [
  { id: 'none', token: '--nt-radius-none', value: '0px' },
  { id: 'small', token: '--nt-radius-sm', value: '6px' },
  { id: 'medium', token: '--nt-radius-md', value: '10px' },
  { id: 'large', token: '--nt-radius-lg', value: '14px' },
  { id: 'extraLarge', token: '--nt-radius-xl', value: '16px' },
  { id: 'twoExtraLarge', token: '--nt-radius-2xl', value: '20px' },
  { id: 'full', token: '--nt-radius-full', value: '9999px' },
]

export const SHADOW_TOKENS: ShadowTokenDefinition[] = [
  { id: 'none', token: '--nt-shadow-none' },
  { id: 'sm', token: '--nt-shadow-sm' },
  { id: 'md', token: '--nt-shadow-md' },
  { id: 'lg', token: '--nt-shadow-lg' },
  { id: 'xl', token: '--nt-shadow-xl' },
  { id: 'focus', token: '--nt-shadow-focus' },
]

export const SIZING_TOKENS: SizingTokenDefinition[] = [
  { id: 'controlSm', token: '--nt-control-height-sm', value: '30px' },
  { id: 'controlMd', token: '--nt-control-height-md', value: '36px' },
  { id: 'controlLg', token: '--nt-control-height-lg', value: '42px' },
  { id: 'inputHeight', token: '--nt-input-height', value: '38px' },
  { id: 'sidebarWidth', token: '--nt-sidebar-width', value: '248px' },
  { id: 'sidebarCollapsedWidth', token: '--nt-sidebar-collapsed-width', value: '64px' },
  { id: 'iconSm', token: '--nt-icon-size-sm', value: '14px' },
  { id: 'iconMd', token: '--nt-icon-size-md', value: '16px' },
  { id: 'iconLg', token: '--nt-icon-size-lg', value: '20px' },
]

export const MOTION_TOKENS: MotionTokenDefinition[] = [
  { id: 'fast', token: '--nt-duration-fast', value: '120ms' },
  { id: 'standard', token: '--nt-duration-standard', value: '180ms' },
  { id: 'slow', token: '--nt-duration-slow', value: '280ms' },
  { id: 'easing', token: '--nt-ease-standard', value: 'cubic-bezier(0.16, 1, 0.3, 1)' },
]
