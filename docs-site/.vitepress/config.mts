import { defineConfig } from 'vitepress'

const SITE_BASE = '/notifytrap/'
const SITE_ORIGIN = 'https://farshidrezaei.github.io/notifytrap'

export default defineConfig({
  base: SITE_BASE,
  title: 'NotifyTrap',
  description:
    'Local-first, offline-capable desktop utility for capturing, inspecting, and debugging Email, SMS, Push, and Webhook notifications.',
  cleanUrls: true,
  lastUpdated: true,

  head: [
    ['link', { rel: 'icon', type: 'image/png', href: `${SITE_BASE}notifytrap-logo.png` }],
    [
      'link',
      {
        rel: 'stylesheet',
        href: 'https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&family=Vazirmatn:wght@400;500;600;700&display=swap',
      },
    ],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'NotifyTrap — Local Notification Capture Desktop Utility' }],
    [
      'meta',
      {
        property: 'og:description',
        content:
          'Capture, inspect, and debug Email (SMTP), SMS, Push, and Webhook notifications locally with zero real-world delivery.',
      },
    ],
    ['meta', { property: 'og:image', content: `${SITE_ORIGIN}/notifytrap-banner.png` }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:image', content: `${SITE_ORIGIN}/notifytrap-banner.png` }],
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      dir: 'ltr',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Channels & API', link: '/guide/channels-and-api' },
          { text: 'Inspection Tools', link: '/guide/advanced-features' },
          { text: 'Ingress & Forwarding', link: '/guide/ingress-and-forwarding' },
          { text: 'CLI & Headless', link: '/guide/cli-and-headless' },
          { text: 'Architecture', link: '/guide/security-and-architecture' },
          { text: 'Development', link: '/guide/development' },
        ],
        sidebar: [
          {
            text: 'Getting Started',
            items: [
              { text: 'Installation & Quickstart', link: '/guide/getting-started' },
              { text: 'Channels, SMTP & HTTP API', link: '/guide/channels-and-api' },
            ],
          },
          {
            text: 'Features & Diagnostics',
            items: [
              { text: 'Advanced Inspection Tools', link: '/guide/advanced-features' },
              { text: 'Ingress Tunnels & Forwarding', link: '/guide/ingress-and-forwarding' },
              { text: 'Headless Server & CLI Companion', link: '/guide/cli-and-headless' },
            ],
          },
          {
            text: 'System & Engineering',
            items: [
              { text: 'Security & Architecture', link: '/guide/security-and-architecture' },
              { text: 'Build from Source & Contributing', link: '/guide/development' },
            ],
          },
        ],
      },
    },
    fa: {
      label: 'فارسی',
      lang: 'fa-IR',
      dir: 'rtl',
      description:
        'ابزار دسکتاپ آفلاین و محلی برای دریافت، بررسی و دیباگ ایمیل، پیامک، پوش‌نوتیفیکیشن و وب‌هوک بدون ارسال واقعی.',
      themeConfig: {
        nav: [
          { text: 'خانه', link: '/fa/' },
          { text: 'شروع سریع', link: '/fa/guide/getting-started' },
          { text: 'کانال‌ها و API', link: '/fa/guide/channels-and-api' },
          { text: 'ابزارهای بازرسی', link: '/fa/guide/advanced-features' },
          { text: 'تانل و فوروارد', link: '/fa/guide/ingress-and-forwarding' },
          { text: 'سرور هدلس و CLI', link: '/fa/guide/cli-and-headless' },
          { text: 'معماری و امنیت', link: '/fa/guide/security-and-architecture' },
          { text: 'توسعه', link: '/fa/guide/development' },
        ],
        sidebar: [
          {
            text: 'شروع کار',
            items: [
              { text: 'نصب و راه‌اندازی سریع', link: '/fa/guide/getting-started' },
              { text: 'کانال‌ها، SMTP و HTTP API', link: '/fa/guide/channels-and-api' },
            ],
          },
          {
            text: 'قابلیت‌ها و ابزارها',
            items: [
              { text: 'ابزارهای پیشرفته بازرسی', link: '/fa/guide/advanced-features' },
              { text: 'تانل اینگرس و فوروارد وب‌هوک', link: '/fa/guide/ingress-and-forwarding' },
              { text: 'سرور هدلس و ابزار CLI', link: '/fa/guide/cli-and-headless' },
            ],
          },
          {
            text: 'مهندسی و معماری',
            items: [
              { text: 'امنیت و معماری سیستم', link: '/fa/guide/security-and-architecture' },
              { text: 'ساخت از سورس و مشارکت', link: '/fa/guide/development' },
            ],
          },
        ],
        outline: { label: 'در این صفحه' },
        docFooter: { prev: 'صفحه قبلی', next: 'صفحه بعدی' },
        lastUpdated: { text: 'آخرین بروزرسانی' },
        darkModeSwitchLabel: 'حالت نمایش',
        sidebarMenuLabel: 'فهرست',
        returnToTopLabel: 'بازگشت به بالا',
      },
    },
  },

  themeConfig: {
    logo: '/notifytrap-logo.png',
    siteTitle: 'NotifyTrap',
    search: {
      provider: 'local',
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/farshidrezaei/notifytrap' },
    ],
    footer: {
      message: 'Released under the MIT License — Local-First Development Utility.',
      copyright: 'NotifyTrap Contributors',
    },
  },
})
