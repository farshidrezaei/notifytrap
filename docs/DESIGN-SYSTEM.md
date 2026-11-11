# NotifyTrap — Design System Specification (`Midnight Indigo v1.0`)

## 1. Visual Identity & Principles

<p>
  <img src="../notifytrap-logo.png" alt="NotifyTrap Logo" width="72" />
</p>

NotifyTrap uses the **Midnight Indigo** developer-tool visual identity:
- **Midnight navy foundations** (Dark: `Midnight Navy`, Light: `Clean Cool White`).
- **Restrained indigo accents** (`#6366F1` brand primary) paired with **selective cyan highlights** (`#22D3EE` brand accent).
- **Clear surface hierarchy** with subtle borders, restrained shadows, minimal decorative effects, and high WCAG readability.
- **Dual-theme parity**: Light and Dark palettes are independently calibrated. Dark mode is **never** inverted light mode.
- **Bilingual parity**: English (`en`, LTR) and Persian (`fa`, RTL) receive equal typographic care using `IBM Plex Sans`, `Vazirmatn`, and `JetBrains Mono`.
- **Nuxt UI First**: Official Nuxt UI 4 components (`UApp`, `USidebar`, `UButton`, `UBadge`, `UInput`, `USelect`, `UTable`, `UModal`, `UTabs`, etc.) mapped via Tailwind CSS v4 `@theme` and `app.config.ts`.

---

## 2. Semantic Color Tokens

### Brand Roles (`app.config.ts` & `app/assets/css/main.css`)
- **Brand Primary / Indigo**: `#6366F1` (Hover `#4F46E5`, Soft `#EEF2FF`), mapped in `@theme` / `:root` & `.dark` (`--ui-primary`)
  - Light theme: `primary` `#4F46E5`, `primary-hover` `#4338CA`, `primary-active` `#3730A3`, `primary-soft` `#EEF2FF`, `secondary` `#6366F1`, `accent` `#0891B2`, `accent-soft` `#ECFEFF`
  - Dark theme: `primary` `#818CF8`, `primary-hover` `#A5B4FC`, `primary-active` `#6366F1`, `primary-soft` `#24234A`, `secondary` `#A5B4FC`, `accent` `#22D3EE`, `accent-soft` `#123744`
- **Neutral**: `slate` scale overridden with Midnight Navy (dark) and Clean Cool White (light) surfaces and borders.

### Surfaces & Borders
| Role | Light (`Clean Cool White`) | Dark (`Midnight Navy`) | Token |
| --- | --- | --- | --- |
| Application Background | `#F7F8FC` | `#090D18` | `--nt-background` |
| Sidebar Background | `#F0F2F8` | `#0D1423` | `--nt-sidebar` |
| Primary Surface | `#FFFFFF` | `#111827` | `--nt-surface` |
| Elevated Surface | `#FFFFFF` | `#182235` | `--nt-surface-elevated` |
| Hover Surface | `#EEF2FF` | `#202D42` | `--nt-surface-hover` |
| Active Surface | `#E0E7FF` | `#263449` | `--nt-surface-active` |
| Modal Overlay | `#172033` | `#050914` | `--nt-overlay` |
| Border / Subtle / Strong | `#E2E8F0` / `#EDF0F5` / `#CBD5E1` | `#263449` / `#1E293B` / `#3B4A61` | `--nt-border`, `--nt-border-subtle`, `--nt-border-strong` |
| Focus Indicator | `#4F46E5` | `#818CF8` | `--nt-border-focus` |

### Semantic Status Colors
| Status | Light Theme | Dark Theme | Token |
| --- | --- | --- | --- |
| Success | `#15803D` | `#4ADE80` | `--ui-success` |
| Warning | `#B45309` | `#FBBF24` | `--ui-warning` |
| Error | `#DC2626` | `#F87171` | `--ui-error` |
| Information | `#0369A1` | `#38BDF8` | `--ui-info` |
| Pending | `#64748B` | `#94A3B8` | `--nt-status-pending` |
| Live | `#059669` | `#34D399` | `--nt-status-live` |

### Notification Channel Colors (Consistent Across Themes)
| Channel | Color | HEX | Icon | Token |
| --- | --- | --- | --- | --- |
| Email | Blue | `#3B82F6` | `i-lucide-mail` | `--nt-channel-email` |
| SMS | Emerald | `#10B981` | `i-lucide-message-square` | `--nt-channel-sms` |
| Push | Violet | `#8B5CF6` | `i-lucide-bell` | `--nt-channel-push` |
| Webhook | Amber | `#D97706` | `i-lucide-webhook` | `--nt-channel-webhook` |

---

## 3. Typography System

- **UI Sans (LTR)**: `'IBM Plex Sans', 'Vazirmatn', system-ui, sans-serif`
- **UI Sans (RTL / Persian)**: `'Vazirmatn', 'IBM Plex Sans', system-ui, sans-serif`
- **Monospace (Payloads, Headers, Keys, Ports)**: `'JetBrains Mono', ui-monospace, monospace`
- **Scale**: Display (`1.5rem` / 24px), Heading (`1.25rem` / 20px), Section (`1rem` / 16px), Body (`0.875rem` / 14px), Label (`0.8125rem` / 13px), Caption/Meta (`0.75rem` / 12px).

---

## 4. Spacing, Radius, Shadow, Sizing & Motion Tokens

- **Spacing Scale (4px base)**: Steps `0, 1, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24` (`0px` through `96px`).
- **Border Radius Scale**:
  - `none`: `0px` (`--nt-radius-none`)
  - `sm`: `6px` (`--nt-radius-sm`)
  - `md`: `10px` (`--nt-radius-md`)
  - `lg`: `14px` (`--nt-radius-lg`)
  - `xl`: `16px` (`--nt-radius-xl`)
  - `2xl`: `20px` (`--nt-radius-2xl`)
  - `full`: `9999px` (`--nt-radius-full`)
- **Shadow Scale**: Independent Light & Dark calibrations (`--nt-shadow-none`, `--nt-shadow-sm`, `--nt-shadow-md`, `--nt-shadow-lg`, `--nt-shadow-xl`, `--nt-shadow-focus`).
- **Component Dimensions**:
  - Control heights: `sm` (`30px`), `md` (`36px`), `lg` (`42px`)
  - Input height: `38px` (`--nt-input-height`)
  - Sidebar width: `248px` (`--nt-sidebar-width`), collapsed: `64px` (`--nt-sidebar-collapsed-width`)
- **Motion Tokens**:
  - `--nt-duration-fast: 120ms`, `--nt-duration-standard: 180ms`, `--nt-duration-slow: 280ms`
  - Mandatory `@media (prefers-reduced-motion: reduce)` override.

---

## 5. Living Showcase (`/design-system`)

The `/design-system` route serves as the executable visual source of truth displaying:
1. Color tokens (semantic + surfaces + channel badges)
2. Typography scale (`en` + `fa` + monospace)
3. Spacing, Radius, Shadow, Sizing, and Motion demos
4. Core Nuxt UI components across states (default, hover, focus, active, disabled, loading, error, success)
5. Live interactive **Light / Dark** and **English (LTR) / Persian (RTL)** verification switchers.

---

## 6. Form Validation (`Zod`) & Input Masking (`Maska`)

- **Schema Validation**: All forms must use `<UForm :schema="schema" :state="state" :validate-on="['blur', 'change', 'input']">` backed by localized **Zod v4** schema factories under `app/schemas/` (`commonValidators.ts`, `serverSettingsForm.ts`, `workspaceForms.ts`, `designSystemShowcaseForm.ts`).
- **Localized Errors**: Validation factories accept `translate` (`t`) from `useI18n()` inside a `computed()` schema so switching between `en` and `fa` updates error messages immediately without hardcoded strings.
- **Input Masks**: Structured technical inputs (IPv4 addresses, TCP ports, slugs, E.164 phone numbers, verification codes, integer bounds) use `v-maska="INPUT_MASKS.<preset>"` registered globally via `app/plugins/maska.ts` and `app/utils/inputMasks.ts`, paired with `dir="ltr"` and `font-mono`.
