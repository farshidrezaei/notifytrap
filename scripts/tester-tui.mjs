#!/usr/bin/env node
/**
 * NotifyTrap Interactive Step-by-Step TUI Tester
 * Pure Node.js (ESM) — Zero external dependencies.
 * Uses native fetch for HTTP and native node:net for local SMTP.
 */

import net from 'node:net'
import readline from 'node:readline'

const HTTP_BASE_URL = process.env.NOTIFYTRAP_HTTP_URL || 'http://127.0.0.1:4080'
const SMTP_HOST = process.env.NOTIFYTRAP_SMTP_HOST || '127.0.0.1'
const SMTP_PORT = Number.parseInt(process.env.NOTIFYTRAP_SMTP_PORT || '1025', 10)

const CREDENTIALS = {
  email: 'mail-982585d408012163.ntsec_ky-ODMwkhVI5q0wcWwWfg_ZoXh8Qlzc_',
  sms: 'sms-011b1c032dccbf3d.ntsec_lz54WN8Z_fceB4MQKIu2JaJ-vhgoRSNW',
  push: 'push-c0905f95054b866a.ntsec_RuyIkBtGbCrEDgs-IGQ4A_bePqKZSaY3',
  webhook: 'hook-6d32b459ceefcea3.ntsec_c4rnsRyjzEJyQHUcIe9MYTkbGD6HITJW',
}

// === ANSI Colors & Styles ===
const RESET = '\x1b[0m'
const BOLD = '\x1b[1m'
const DIM = '\x1b[2m'
const CYAN = '\x1b[38;2;6;182;212m'
const BLUE = '\x1b[38;2;59;130;246m'
const INDIGO = '\x1b[38;2;99;102;241m'
const EMERALD = '\x1b[38;2;16;185;129m'
const AMBER = '\x1b[38;2;245;158;11m'
const ROSE = '\x1b[38;2;244;63;94m'
const GRAY = '\x1b[38;2;100;116;139m'
const WHITE = '\x1b[38;2;248;250;252m'
const BG_SELECT = '\x1b[48;2;30;41;59m'

function clearScreen() {
  process.stdout.write('\x1b[2J\x1b[H')
}

// === Minimal Native SMTP Client ===
async function sendRawSmtpEmail(channelKey, subject, textContent, htmlContent) {
  if (!channelKey.includes('.')) {
    return { ok: false, output: 'Invalid email channel key format (expected identifier.secret)' }
  }
  const [username, password] = channelKey.split('.', 2)

  return new Promise((resolve) => {
    const socket = net.createConnection({ host: SMTP_HOST, port: SMTP_PORT }, () => {
      let step = 0
      let buffer = ''

      socket.on('data', (chunk) => {
        buffer += chunk.toString()
        const lines = buffer.split('\r\n')
        buffer = lines.pop() ?? ''

        for (const line of lines) {
          const code = line.slice(0, 3)
          if (step === 0 && code === '220') {
            socket.write('EHLO localhost\r\n')
            step = 1
          } else if (step === 1 && code === '250') {
            socket.write('AUTH LOGIN\r\n')
            step = 2
          } else if (step === 2 && code === '334') {
            socket.write(`${Buffer.from(username).toString('base64')}\r\n`)
            step = 3
          } else if (step === 3 && code === '334') {
            socket.write(`${Buffer.from(password).toString('base64')}\r\n`)
            step = 4
          } else if (step === 4 && code === '235') {
            socket.write('MAIL FROM:<system@local.notifytrap>\r\n')
            step = 5
          } else if (step === 5 && code === '250') {
            socket.write('RCPT TO:<developer@example.com>\r\n')
            step = 6
          } else if (step === 6 && code === '250') {
            socket.write('DATA\r\n')
            step = 7
          } else if (step === 7 && code === '354') {
            const boundary = '----=_Part_NotifyTrap_482'
            const rawMessage = [
              'From: system@local.notifytrap',
              'To: developer@example.com',
              `Subject: ${subject}`,
              'MIME-Version: 1.0',
              'X-NotifyTrap-Run: node-tui-test',
              `Content-Type: multipart/alternative; boundary="${boundary}"`,
              '',
              `--${boundary}`,
              'Content-Type: text/plain; charset=utf-8',
              'Content-Transfer-Encoding: 8bit',
              '',
              textContent,
              '',
              `--${boundary}`,
              'Content-Type: text/html; charset=utf-8',
              'Content-Transfer-Encoding: 8bit',
              '',
              htmlContent || `<p>${textContent}</p>`,
              '',
              `--${boundary}--`,
              '',
              '.\r\n',
            ].join('\r\n')

            socket.write(rawMessage)
            step = 8
          } else if (step === 8 && code === '250') {
            socket.write('QUIT\r\n')
            socket.end()
            resolve({ ok: true, output: `250 2.0.0 OK: Queued message '${subject}' via local SMTP` })
          }
        }
      })
    })

    socket.setTimeout(8000, () => {
      socket.destroy()
      resolve({ ok: false, output: 'SMTP connection timed out' })
    })

    socket.on('error', (err) => {
      resolve({ ok: false, output: `SMTP Connection error: ${err.message}` })
    })
  })
}

// === Native HTTP Client ===
async function sendRawHttp(path, channelKey, payload) {
  try {
    const response = await fetch(`${HTTP_BASE_URL}${path}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json; charset=utf-8',
        Authorization: `Bearer ${channelKey}`,
        'X-NotifyTrap-Run': 'node-tui-test',
      },
      body: JSON.stringify(payload),
    })
    const body = await response.text()
    return {
      ok: response.ok,
      output: `HTTP ${response.status} ${response.statusText}\n${body}`,
    }
  } catch (error) {
    return { ok: false, output: `Fetch error: ${error.message}` }
  }
}

// === Scenarios Catalog ===
const SCENARIOS = {
  email: [
    {
      id: 'email_otp',
      title: 'Email: Standard OTP Verification Code',
      desc: 'Tests regular OTP extraction, magic link parsing, and delivery status.',
      hint: 'Check Inbox > Verification Badge & copy OTP button in UI.',
      data: {
        subject: 'کد ورود به حساب کاربری: 849-201',
        text: 'کد یکبار مصرف شما: 849-201\nاین کد تا ۲ دقیقه معتبر است.\nلینک مستقیم: https://local.notifytrap/auth/verify?code=849201',
        html: `<div style="font-family:sans-serif;padding:24px;background:#f8fafc;">
          <div style="max-width:480px;margin:auto;background:#fff;padding:24px;border-radius:12px;border:1px solid #e2e8f0;">
            <h2 style="color:#0f172a;margin-top:0;">کد تایید دو مرحله‌ای</h2>
            <p style="color:#475569;">سلام فرشید، کد یکبار مصرف شما جهت ورود:</p>
            <div style="background:#f1f5f9;padding:16px;text-align:center;font-size:28px;letter-spacing:6px;font-weight:bold;color:#1e293b;border-radius:8px;">849-201</div>
            <p style="font-size:12px;color:#94a3b8;margin-top:20px;">اگر شما این درخواست را نداده‌اید، این پیام را نادیده بگیرید.</p>
          </div>
        </div>`,
      },
    },
    {
      id: 'email_linter',
      title: 'Email: Unrendered Placeholders & Leaked Sentinels',
      desc: 'Contains {{ user.firstName }}, ${token}, and null/undefined values to test Phase 1 Linter.',
      hint: 'Check Email Inspector > Audit Tab! Look for the orange Variables metric and warning findings.',
      data: {
        subject: 'Welcome {{ user.firstName }}! Your account status is undefined',
        text: 'Hello {{ user.name }},\nToken: ${activation_token}\nAssigned team: null\nRegards.',
        html: `<div style="font-family:sans-serif;padding:20px;">
          <h2>Welcome {{ user.firstName }}!</h2>
          <p>Your activation link: <a href="https://example.com/activate?token=\${activation_token}">Activate</a></p>
          <p>Assigned plan: <strong>None</strong> | Expiry: <strong>null</strong></p>
          <p>Metadata: [object Object]</p>
        </div>`,
      },
    },
    {
      id: 'email_mjml',
      title: 'Email: Advanced MJML Newsletter Template',
      desc: 'Sends valid MJML markup to test the Phase 1 MJML Inspector Tab & semantic compiler.',
      hint: 'Check Email Inspector > MJML Tab! View parsed tags, Copy HTML button, and side-by-side preview.',
      data: {
        subject: 'جشنواره پاییزه NotifyTrap (MJML Template)',
        text: 'قالب MJML حاوی دکمه فعالسازی و استایل ریسپانسیو.',
        html: `<mjml>
  <mj-head>
    <mj-title>NotifyTrap MJML Test</mj-title>
    <mj-preview>پیش‌نمایش قالب ریسپانسیو MJML</mj-preview>
  </mj-head>
  <mj-body background-color="#f4f5f7">
    <mj-section background-color="#1e1b4b" padding="24px 0">
      <mj-column>
        <mj-text color="#ffffff" font-size="20px" font-weight="bold" align="center">
          ⚡ رویداد مهندسی NotifyTrap Pro
        </mj-text>
      </mj-column>
    </mj-section>
    <mj-section background-color="#ffffff" padding="20px">
      <mj-column width="100%">
        <mj-text font-size="15px" color="#334155" line-height="1.8">
          سلام کاربر گرامی، شما به نسخه آزمایشی NotifyTrap دعوت شده‌اید.
        </mj-text>
        <mj-divider border-color="#e2e8f0" border-width="1px" />
        <mj-button href="https://notifytrap.dev/claim" background-color="#4f46e5" color="#ffffff" border-radius="8px" font-size="14px">
          تایید و فعال‌سازی اشتراک
        </mj-button>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>`,
      },
    },
    {
      id: 'email_responsive',
      title: 'Email: Multi-Column Responsive Newsletter',
      desc: 'Designed to test the iPhone 15, Pixel 8, and iPad Mini viewport switcher and Dark Client mode.',
      hint: 'Click iPhone 15 / Pixel 8 / Dark Client buttons in the Email Inspector header.',
      data: {
        subject: 'NotifyTrap Monthly Engineering Dispatch',
        text: 'Read our monthly dispatch online at https://notifytrap.dev',
        html: `<div style="max-width:680px;margin:auto;font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;color:#1e293b;background:#ffffff;border:1px solid #e2e8f0;border-radius:12px;overflow:hidden;">
          <div style="background:linear-gradient(135deg,#4f46e5,#7c3aed);padding:32px 24px;color:#ffffff;text-align:center;">
            <h1 style="margin:0;font-size:24px;">NotifyTrap Engineering Update</h1>
            <p style="margin:8px 0 0;opacity:0.9;">Local capture, zero cloud leaks, instant verification.</p>
          </div>
          <div style="padding:24px;">
            <h3 style="color:#0f172a;">Phase 1 Features Active:</h3>
            <ul style="color:#475569;line-height:1.7;">
              <li>Device frame simulation (iPhone 15, Pixel 8, iPad Mini)</li>
              <li>Dark client simulation inside sandboxed iframe</li>
              <li>Template placeholder detection in Audit checklist</li>
              <li>Native MJML parser and compiler</li>
            </ul>
          </div>
        </div>`,
      },
    },
    {
      id: 'email_asset_audit',
      title: 'Email: Broken Links & Missing CDN Image Test',
      desc: 'Contains active links, a 404 broken URL, and a broken image to test Phase 2 Asset Auditor.',
      hint: 'Check Email Inspector > Audit Tab > Click "Audit Links & Assets" to view live HTTP status codes!',
      data: {
        subject: 'Phase 2 Test: Broken Links and Image Assets',
        text: 'Checking links: https://notifytrap.dev and http://127.0.0.1:4080/non-existent-page-404',
        html: `<div style="font-family:sans-serif;padding:20px;">
          <h2>NotifyTrap Asset Health Audit Test</h2>
          <p>Healthy link: <a href="https://github.com/farshidrezaei/notifytrap">NotifyTrap GitHub</a></p>
          <p>Broken link: <a href="http://127.0.0.1:4080/api/v1/non-existent-page-404">Dead 404 Endpoint</a></p>
          <p>Remote image: <img src="https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/72x72/1f680.png" alt="Rocket" width="36" /></p>
          <p>Broken image: <img src="http://127.0.0.1:4080/images/missing-logo-404.png" alt="Missing Logo" width="36" /></p>
        </div>`,
      },
    },
  ],
  push: [
    {
      id: 'push_ios',
      title: 'Push: Apple iOS (APNs Rich Order Success)',
      desc: 'APNs payload with badge 3, sound chime, deep-link data, and action handlers.',
      hint: 'Check Push Inspector > Switch between Lock Screen & Banner views!',
      data: {
        path: '/api/v1/push',
        payload: {
          device_token: 'apns-token-iphone15-pro-max-998811',
          platform: 'ios',
          title: '🎉 سفارش جدید #48291 ثبت شد',
          body: 'پرداخت با موفقیت انجام شد. مبلغ: ۲,۴۵۰,۰۰۰ تومان. کد پیگیری: ۹۸۲۱',
          badge: 3,
          sound: 'payment_success.caf',
          data: {
            orderId: 48291,
            deepLink: 'myapp://orders/48291',
            priority: 'high',
            customer: { id: 'usr_9981', name: 'فرشید رضایی', tier: 'VIP' },
          },
        },
      },
    },
    {
      id: 'push_android',
      title: 'Push: Android (FCM Critical Server Alert)',
      desc: 'FCM payload simulating high CPU latency and infrastructure alert channel.',
      hint: 'Check Push Inspector > Notification Shade view!',
      data: {
        path: '/api/v1/push',
        payload: {
          device_token: 'fcm_eKx9_sample_token_android_pixel8',
          platform: 'android',
          title: '⚡ هشدار مانیتورینگ سرور ابری',
          body: 'مصرف پردازنده سرور cluster-prod-02 به ۸۹٪ رسید. بررسی فوری لازم است.',
          badge: 1,
          sound: 'critical_alarm.mp3',
          data: {
            cluster: 'cluster-prod-02',
            metric: 'cpu_utilization',
            value: 89.4,
            urgency: 'critical',
          },
        },
      },
    },
    {
      id: 'push_web',
      title: 'Push: Web Push (Browser Promotional Campaign)',
      desc: 'Web push payload with URL redirection and marketing campaign tags.',
      hint: 'Check Push Inspector > Web device view.',
      data: {
        path: '/api/v1/push',
        payload: {
          device_token: 'web-push-chrome-macos-token-8877',
          platform: 'web',
          title: 'تخفیف ویژه دوره‌های آموزش Rust و Tauri',
          body: 'کد تخفیف اختصاصی ۵۰ درصدی شما برای دوره تائوری فعال شد: TAURI50',
          badge: 1,
          sound: 'default',
          data: { campaign: 'tauri_adv_2026', url: '/courses/tauri' },
        },
      },
    },
  ],
  sms: [
    {
      id: 'sms_otp',
      title: 'SMS: English Short OTP (GSM-7 Single Segment)',
      desc: 'Single-segment SMS with auto-extracted verification code.',
      hint: 'Check SMS Inspector > Conversation View with Delivered receipt.',
      data: {
        path: '/api/v1/sms',
        payload: {
          to: '+989121112233',
          from: 'NOTIFYTRAP',
          message: 'Your verification code is 591024. Valid for 3 minutes.',
          metadata: { template: 'otp_en', attempt: 1 },
        },
      },
    },
    {
      id: 'sms_long',
      title: 'SMS: Persian Multi-Segment Delivery Update (3 Segments)',
      desc: 'Long Persian Unicode message triggering 3 SMS segments calculation.',
      hint: 'Check SMS Inspector > PDU segments count & Lock Screen incoming alert card.',
      data: {
        path: '/api/v1/sms',
        payload: {
          to: '+989354445566',
          from: 'بانک سامان',
          message: 'فرشید گرامی، سفارش شماره #9821 با موفقیت در انبار مرکزی پردازش گردید و تحویل راننده اسنپ‌باکس شد. کد تحویل مرسوله: 4920 است. جهت رهگیری آنلاین روی لینک کلیک کنید: https://track.notifytrap.dev/order/9821',
          metadata: { carrier: 'MCI', provider: 'kavenegar' },
        },
      },
    },
  ],
  webhook: [
    {
      id: 'webhook_stripe',
      title: 'Webhook: Stripe Payment Succeeded (invoice.paid)',
      desc: 'Rich Stripe event payload with amounts, customer metadata, and transaction ID.',
      hint: 'Check Webhook Inspector > Headers, Subpath, and Replay / Chaos simulation.',
      data: {
        subpath: 'stripe/events',
        payload: {
          id: 'evt_3MvwE2LkdIwHu7ix0H645bS8',
          type: 'invoice.paid',
          created: Math.floor(Date.now() / 1000),
          data: {
            object: {
              id: 'in_1MvwE2LkdIwHu7ixwGZ2nK',
              amount_paid: 4900,
              currency: 'usd',
              customer_email: 'farshid@example.com',
              status: 'paid',
            },
          },
        },
      },
    },
    {
      id: 'webhook_github',
      title: 'Webhook: GitHub Push Commit Event',
      desc: 'Simulates GitHub repository push event with commit messages and author info.',
      hint: 'Inspect JSON payload formatting and try Replay button in NotifyTrap.',
      data: {
        subpath: 'github/events',
        payload: {
          event: 'push',
          ref: 'refs/heads/main',
          repository: { full_name: 'farshidrezaei/notifytrap' },
          sender: { login: 'farshidrezaei' },
          commits: [
            {
              id: 'acf10e0',
              message: 'feat(phase-1): device frames, mjml preview and template linter',
              timestamp: '2026-11-12T20:15:00Z',
            },
          ],
        },
      },
    },
  ],
}

class StepTuiNode {
  constructor() {
    this.activeChannels = Object.entries(CREDENTIALS).filter(([_, key]) => Boolean(key && key.trim()))
    this.selectedChannelIdx = 0
    this.selectedScenarioIdx = 0
    this.currentStep = 1
    this.lastResult = null
  }

  renderHeader(stepTitle) {
    clearScreen()
    const w = 70
    console.log(`${INDIGO}╭${'─'.repeat(w)}╮${RESET}`)
    console.log(`${INDIGO}│${RESET} ${BOLD}${WHITE}🚀 NotifyTrap Pro Test Dispatcher (Node.js ESM)${RESET}${' '.repeat(w - 47)}${INDIGO}│${RESET}`)
    console.log(`${INDIGO}│${RESET} ${DIM}${GRAY}HTTP: ${HTTP_BASE_URL}  |  SMTP: ${SMTP_HOST}:${SMTP_PORT}${RESET}${' '.repeat(w - 51)}${INDIGO}│${RESET}`)
    console.log(`${INDIGO}├${'─'.repeat(w)}┤${RESET}`)

    const steps = [
      ['1', 'Channel'],
      ['2', 'Scenario'],
      ['3', 'Preview'],
      ['4', 'Dispatch'],
    ]
    let crumb = '  '
    for (const [num, label] of steps) {
      const sNum = Number.parseInt(num, 10)
      if (sNum === this.currentStep) {
        crumb += `${EMERALD}${BOLD}[${num}] ${label}${RESET} → `
      } else if (sNum < this.currentStep) {
        crumb += `${CYAN}✓ ${label}${RESET} → `
      } else {
        crumb += `${GRAY}${num}. ${label}${RESET} → `
      }
    }
    crumb = crumb.replace(/ → $/, '')
    console.log(`${INDIGO}│${RESET} ${crumb}${' '.repeat(Math.max(0, w - crumb.length + 12))}${INDIGO}│${RESET}`)
    console.log(`${INDIGO}├${'─'.repeat(w)}┤${RESET}`)
    console.log(`${INDIGO}│${RESET} ${BOLD}${CYAN}▶ ${stepTitle}${RESET}${' '.repeat(Math.max(0, w - stepTitle.length - 4))}${INDIGO}│${RESET}`)
    console.log(`${INDIGO}╰${'─'.repeat(w)}╯${RESET}\n`)
  }

  async readKey() {
    readline.emitKeypressEvents(process.stdin)
    if (process.stdin.isTTY) {
      process.stdin.setRawMode(true)
    }

    return new Promise((resolve) => {
      const onKeypress = (str, key) => {
        process.stdin.removeListener('keypress', onKeypress)
        if (key.ctrl && key.name === 'c') {
          clearScreen()
          process.exit(0)
        }
        if (key.name === 'up') resolve('UP')
        else if (key.name === 'down') resolve('DOWN')
        else if (key.name === 'return') resolve('ENTER')
        else if (key.name === 'escape') resolve('ESC')
        else if (key.name === 'backspace') resolve('BACKSPACE')
        else resolve(str || key.name)
      }
      process.stdin.on('keypress', onKeypress)
    })
  }

  async runStep1Channel() {
    const items = [
      ...this.activeChannels.map(([kind, key]) => [kind, key.split('.')[0]]),
      ['batch', 'Fire all matrix scenarios sequentially'],
    ]

    while (true) {
      this.renderHeader('Step 1/4: Select Target Channel')
      console.log(` ${GRAY}Use ${BOLD}[↑/↓]${RESET}${GRAY} to navigate, ${BOLD}[ENTER]${RESET}${GRAY} to select, ${BOLD}[Q]${RESET}${GRAY} to quit:${RESET}\n`)

      for (let idx = 0; idx < items.length; idx++) {
        const [kind, ident] = items[idx]
        const isSelected = idx === this.selectedChannelIdx
        const prefix = isSelected ? `${EMERALD}● ${RESET}` : '  '
        const icon = { email: '✉', push: '🔔', sms: '💬', webhook: '⚡', batch: '🚀' }[kind] || '•'
        const line = `${icon}  ${kind.toUpperCase().padEnd(8)} (${ident})`

        if (isSelected) {
          console.log(` ${prefix}${BG_SELECT}${BOLD}${WHITE} ${line.padEnd(58)} ${RESET}`)
        } else {
          console.log(` ${prefix}${GRAY} ${line.padEnd(58)} ${RESET}`)
        }
      }

      const key = await this.readKey()
      if (key === 'UP' || key === 'k') {
        this.selectedChannelIdx = (this.selectedChannelIdx - 1 + items.length) % items.length
      } else if (key === 'DOWN' || key === 'j') {
        this.selectedChannelIdx = (this.selectedChannelIdx + 1) % items.length
      } else if (/^[1-9]$/.test(key) && Number(key) <= items.length) {
        this.selectedChannelIdx = Number(key) - 1
        if (this.selectedChannelIdx === items.length - 1) {
          await this.executeBatchDispatch()
          this.currentStep = 4
        } else {
          this.currentStep = 2
          this.selectedScenarioIdx = 0
        }
        break
      } else if (key === 'ENTER') {
        if (this.selectedChannelIdx === items.length - 1) {
          await this.executeBatchDispatch()
          this.currentStep = 4
        } else {
          this.currentStep = 2
          this.selectedScenarioIdx = 0
        }
        break
      } else if (key === 'q' || key === 'Q' || key === 'ESC') {
        clearScreen()
        process.exit(0)
      }
    }
  }

  async runStep2Scenario() {
    const [channelKind] = this.activeChannels[this.selectedChannelIdx]
    const scenarios = SCENARIOS[channelKind] || []

    while (true) {
      this.renderHeader(`Step 2/4: Select ${channelKind.toUpperCase()} Scenario`)
      console.log(` ${GRAY}Use ${BOLD}[↑/↓]${RESET}${GRAY} to navigate, ${BOLD}[ENTER]${RESET}${GRAY} to select, ${BOLD}[B/ESC]${RESET}${GRAY} for back:${RESET}\n`)

      for (let idx = 0; idx < scenarios.length; idx++) {
        const item = scenarios[idx]
        const isSelected = idx === this.selectedScenarioIdx
        const prefix = isSelected ? `${EMERALD}● ${RESET}` : '  '

        if (isSelected) {
          console.log(` ${prefix}${BG_SELECT}${BOLD}${WHITE} ${idx + 1}. ${item.title.padEnd(58)} ${RESET}`)
          console.log(`    ${CYAN}↳ ${item.desc}${RESET}`)
          console.log(`    ${AMBER}💡 ${item.hint}${RESET}\n`)
        } else {
          console.log(` ${prefix}${GRAY} ${idx + 1}. ${item.title.padEnd(58)} ${RESET}`)
        }
      }

      const key = await this.readKey()
      if (key === 'UP' || key === 'k') {
        this.selectedScenarioIdx = (this.selectedScenarioIdx - 1 + scenarios.length) % scenarios.length
      } else if (key === 'DOWN' || key === 'j') {
        this.selectedScenarioIdx = (this.selectedScenarioIdx + 1) % scenarios.length
      } else if (/^[1-9]$/.test(key) && Number(key) <= scenarios.length) {
        this.selectedScenarioIdx = Number(key) - 1
        this.currentStep = 3
        break
      } else if (key === 'ENTER') {
        this.currentStep = 3
        break
      } else if (key === 'b' || key === 'B' || key === 'ESC' || key === 'BACKSPACE') {
        this.currentStep = 1
        break
      } else if (key === 'q' || key === 'Q') {
        clearScreen()
        process.exit(0)
      }
    }
  }

  async runStep3Preview() {
    const [channelKind, channelKey] = this.activeChannels[this.selectedChannelIdx]
    const scenario = SCENARIOS[channelKind][this.selectedScenarioIdx]

    while (true) {
      this.renderHeader('Step 3/4: Inspect Payload & Confirm Dispatch')
      console.log(` ${BOLD}Target Channel:${RESET}  ${CYAN}${channelKind.toUpperCase()}${RESET} (${channelKey.split('.')[0]})`)
      console.log(` ${BOLD}Scenario Title:${RESET}  ${WHITE}${scenario.title}${RESET}`)
      console.log(` ${BOLD}Inspector Hint:${RESET}  ${AMBER}${scenario.hint}${RESET}`)
      console.log(`\n${GRAY}── Payload Preview ──────────────────────────────────────${RESET}`)

      if (channelKind === 'email') {
        console.log(` ${BOLD}Subject:${RESET} ${scenario.data.subject}`)
        console.log(` ${BOLD}Text:${RESET}    ${scenario.data.text.slice(0, 120)}...`)
        console.log(` ${BOLD}HTML:${RESET}    <${scenario.data.html.length} bytes of content>`)
      } else {
        const pretty = JSON.stringify(scenario.data.payload, null, 2)
        const lines = pretty.split('\n')
        for (const line of lines.slice(0, 14)) {
          console.log(`  ${line}`)
        }
        if (lines.length > 14) {
          console.log(`  ${GRAY}... (${lines.length - 14} lines truncated)${RESET}`)
        }
      }
      console.log(`${GRAY}─────────────────────────────────────────────────────────${RESET}\n`)
      console.log(` ${BOLD}${EMERALD}[ENTER]${RESET} Send Now     ${BOLD}[B]${RESET} Back     ${BOLD}[Q]${RESET} Cancel\n`)

      const key = await this.readKey()
      if (key === 'ENTER') {
        const res = await this.dispatchSingle(channelKind, channelKey, scenario)
        this.lastResult = { ...res, hint: scenario.hint }
        this.currentStep = 4
        break
      } else if (key === 'b' || key === 'B' || key === 'ESC' || key === 'BACKSPACE') {
        this.currentStep = 2
        break
      } else if (key === 'q' || key === 'Q') {
        clearScreen()
        process.exit(0)
      }
    }
  }

  async dispatchSingle(channelKind, channelKey, scenario) {
    if (channelKind === 'email') {
      return sendRawSmtpEmail(
        channelKey,
        scenario.data.subject,
        scenario.data.text,
        scenario.data.html,
      )
    }
    if (channelKind === 'push') {
      return sendRawHttp(scenario.data.path, channelKey, scenario.data.payload)
    }
    if (channelKind === 'sms') {
      return sendRawHttp(scenario.data.path, channelKey, scenario.data.payload)
    }
    if (channelKind === 'webhook') {
      const ident = channelKey.split('.')[0]
      return sendRawHttp(`/api/v1/webhooks/${ident}/${scenario.data.subpath}`, channelKey, scenario.data.payload)
    }
    return { ok: false, output: 'Unknown channel kind' }
  }

  async executeBatchDispatch() {
    this.renderHeader('Batch Matrix Dispatch: Firing All Scenarios')
    console.log(` ${CYAN}Sending full notification suite to NotifyTrap...${RESET}\n`)

    const results = []
    for (const [kind, key] of this.activeChannels) {
      const scenarios = SCENARIOS[kind] || []
      for (const scenario of scenarios) {
        process.stdout.write(`  → Sending [${kind.toUpperCase()}] ${scenario.title}... `)
        const res = await this.dispatchSingle(kind, key, scenario)
        if (res.ok) {
          console.log(`${EMERALD}✔ OK${RESET}`)
        } else {
          console.log(`${ROSE}✖ FAIL (${res.output})${RESET}`)
        }
        results.push(res.ok)
        await new Promise((r) => setTimeout(r, 150))
      }
    }

    const totalOk = results.filter(Boolean).length
    this.lastResult = {
      ok: totalOk === results.length,
      output: `Batch Complete: ${totalOk}/${results.length} notifications delivered successfully.`,
      hint: 'Check NotifyTrap Inbox! All notifications and inspector features are populated.',
    }
  }

  async runStep4Result() {
    if (!this.lastResult) {
      this.currentStep = 1
      return
    }

    while (true) {
      this.renderHeader('Step 4/4: Dispatch Result & Verification Guide')

      if (this.lastResult.ok) {
        console.log(` ${EMERALD}${BOLD}✔ DISPATCH SUCCESSFUL${RESET}\n`)
      } else {
        console.log(` ${ROSE}${BOLD}✖ DISPATCH FAILED${RESET}\n`)
      }

      console.log(`${GRAY}── Server Response ──────────────────────────────────────${RESET}`)
      for (const line of this.lastResult.output.split('\n')) {
        console.log(` ${line}`)
      }
      console.log(`${GRAY}─────────────────────────────────────────────────────────${RESET}\n`)

      console.log(` ${BOLD}🔍 What to inspect in NotifyTrap Desktop:${RESET}`)
      console.log(` ${AMBER}👉 ${this.lastResult.hint}${RESET}\n`)

      console.log(` ${BOLD}${EMERALD}[ENTER]${RESET} Send Another     ${BOLD}[Q]${RESET} Exit\n`)

      const key = await this.readKey()
      if (key === 'ENTER' || key === 'b' || key === 'B' || key === 'ESC') {
        this.currentStep = 1
        break
      } else if (key === 'q' || key === 'Q') {
        clearScreen()
        process.exit(0)
      }
    }
  }

  async start() {
    clearScreen()
    while (true) {
      if (this.currentStep === 1) await this.runStep1Channel()
      else if (this.currentStep === 2) await this.runStep2Scenario()
      else if (this.currentStep === 3) await this.runStep3Preview()
      else if (this.currentStep === 4) await this.runStep4Result()
    }
  }
}

const app = new StepTuiNode()
app.start().catch((err) => {
  console.error(err)
  process.exit(1)
})
