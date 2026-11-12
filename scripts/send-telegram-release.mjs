#!/usr/bin/env node
/**
 * scripts/send-telegram-release.mjs
 *
 * Broadcasts newly published NotifyTrap release changelog to the official Telegram channel.
 *
 * Rules:
 * - Only broadcasts published (non-draft) releases.
 * - Idempotent: checks for `<!-- telegram:announced -->` marker in release body.
 * - Telegram HTML compatible formatting with character limit guards.
 * - Sensitive tokens read strictly from environment variables.
 * - Emits clear GitHub Actions error annotations on failure.
 */

import fs from 'node:fs';
import path from 'node:path';

function escapeHtml(text) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

export function extractChangelogForVersion(changelogContent, version) {
  const versionHeader = `## [${version}]`;
  const lines = changelogContent.split('\n');
  let capturing = false;
  const result = [];

  for (const line of lines) {
    if (line.startsWith(versionHeader)) {
      capturing = true;
      continue;
    }
    if (capturing && line.startsWith('## [')) {
      break;
    }
    if (capturing) {
      result.push(line);
    }
  }

  return result.join('\n').trim();
}

export function formatMarkdownToTelegramHtml(markdown) {
  const lines = markdown.split('\n');
  const formattedLines = [];

  for (let rawLine of lines) {
    const line = rawLine.trimEnd();
    if (!line) {
      formattedLines.push('');
      continue;
    }

    if (line.startsWith('### Added')) {
      formattedLines.push('✨ <b>قابلیت‌های جدید (Added):</b>');
      continue;
    }
    if (line.startsWith('### Fixed')) {
      formattedLines.push('🐛 <b>رفع باگ‌ها (Fixed):</b>');
      continue;
    }
    if (line.startsWith('### Changed')) {
      formattedLines.push('🔄 <b>تغییرات (Changed):</b>');
      continue;
    }
    if (line.startsWith('### Security')) {
      formattedLines.push('🛡️ <b>بهبودهای امنیتی (Security):</b>');
      continue;
    }
    if (line.startsWith('### Removed') || line.startsWith('### Deprecated')) {
      formattedLines.push('⚠️ <b>تغییرات ساختاری (Deprecated/Removed):</b>');
      continue;
    }

    let isBullet = false;
    let content = line;
    if (content.startsWith('- ') || content.startsWith('* ')) {
      isBullet = true;
      content = content.substring(2);
    }

    // Escape HTML special characters
    let escaped = escapeHtml(content);

    // Convert bold **text** to <b>text</b>
    escaped = escaped.replace(/\*\*(.*?)\*\*/g, '<b>$1</b>');

    // Convert inline code `text` to <code>text</code>
    escaped = escaped.replace(/`([^`]+)`/g, '<code>$1</code>');

    // Convert markdown link [text](url) to <a href="url">text</a>
    escaped = escaped.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>');

    if (isBullet) {
      formattedLines.push(`• ${escaped}`);
    } else {
      formattedLines.push(escaped);
    }
  }

  return formattedLines.join('\n').replace(/\n{3,}/g, '\n\n').trim();
}

export function buildTelegramReleaseMessage({ version, releaseUrl, changelogMarkdown }) {
  const formattedChangelog = formatMarkdownToTelegramHtml(changelogMarkdown);

  const header =
    `🚀 <b>NotifyTrap v${version} منتشر شد!</b>\n` +
    `────────────────────\n` +
    `<i>Local-first, offline-capable notification inspector</i>\n\n`;

  const platforms =
    `\n\n📦 <b>نصاب‌های رسمی:</b>\n` +
    `🐧 <b>Linux:</b> <code>.AppImage</code> | <code>.deb</code> | <code>.rpm</code>\n` +
    `🪟 <b>Windows:</b> <code>.exe</code> | <code>.msi</code>\n` +
    `🍏 <b>macOS:</b> <code>.dmg</code> (Universal Intel &amp; Apple Silicon)\n\n`;

  const footer =
    `🔗 <a href="${releaseUrl}">دانلود نصاب‌ها و مشاهده نسخه کامل</a>\n` +
    `────────────────────\n` +
    `📢 کانال رسمی: @notifytrap | 🌐 <a href="https://t.me/notifytrap">عضویت در کانال</a>`;

  // Telegram strict limit is 4096 characters; allow up to 2600 chars for changelog
  const maxChangelogLength = 2600;
  let finalChangelog = formattedChangelog;
  if (finalChangelog.length > maxChangelogLength) {
    finalChangelog = finalChangelog.slice(0, maxChangelogLength);
    const lastNewline = finalChangelog.lastIndexOf('\n');
    if (lastNewline > 0) {
      finalChangelog = finalChangelog.slice(0, lastNewline);
    }
    finalChangelog += `\n\n<i>... و تغییرات بیشتر (مشاهده متن کامل در گیت‌هاب)</i>`;
  }

  return `${header}${finalChangelog}${platforms}${footer}`;
}

async function main() {
  const botToken = process.env.TELEGRAM_BOT_TOKEN;
  const chatId = process.env.TELEGRAM_CHAT_ID;
  const ghToken = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
  const repository = process.env.GITHUB_REPOSITORY || 'farshidrezaei/notifytrap';
  const releaseId = process.env.RELEASE_ID;
  const isDryRun = process.env.DRY_RUN === 'true';

  if (!botToken || !chatId) {
    console.log('Notice: TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID not provided. Skipping Telegram announcement.');
    process.exit(0);
  }

  if (!releaseId && !isDryRun) {
    console.error('::error::RELEASE_ID environment variable is missing.');
    process.exit(1);
  }

  // 1. Fetch release details from GitHub API
  let release;
  if (isDryRun && !releaseId) {
    release = {
      tag_name: 'v1.4.0',
      html_url: `https://github.com/${repository}/releases/tag/v1.4.0`,
      draft: false,
      body: 'Mock release body',
    };
  } else {
    const ghResponse = await fetch(`https://api.github.com/repos/${repository}/releases/${releaseId}`, {
      headers: {
        Accept: 'application/vnd.github+json',
        Authorization: `Bearer ${ghToken}`,
        'X-GitHub-Api-Version': '2022-11-28',
      },
    });

    if (!ghResponse.ok) {
      const errorText = await ghResponse.text();
      console.error(`::error::Failed to fetch release ${releaseId} from GitHub API: ${ghResponse.status} ${errorText}`);
      process.exit(1);
    }
    release = await ghResponse.json();
  }

  // 2. Enforce only published releases
  if (release.draft) {
    console.log(`Release ${release.tag_name} is still a draft. Telegram notification will only run after publication.`);
    process.exit(0);
  }

  // 3. Idempotency Guard: prevent duplicate telegram broadcast
  const ANNOUNCED_MARKER = '<!-- telegram:announced -->';
  if (release.body && release.body.includes(ANNOUNCED_MARKER)) {
    console.log(`Idempotency check: Release ${release.tag_name} was already announced to Telegram. Skipping.`);
    process.exit(0);
  }

  // 4. Resolve version and changelog
  const version = release.tag_name.replace(/^v/, '');
  let changelogSection = '';

  const changelogPath = path.resolve(process.cwd(), 'CHANGELOG.md');
  if (fs.existsSync(changelogPath)) {
    const changelogFileContent = fs.readFileSync(changelogPath, 'utf8');
    changelogSection = extractChangelogForVersion(changelogFileContent, version);
  }

  if (!changelogSection && release.body) {
    changelogSection = release.body
      .replace(/Cross-platform desktop installers.*/, '')
      .replace(ANNOUNCED_MARKER, '')
      .trim();
  }

  // 5. Build template message
  const messageText = buildTelegramReleaseMessage({
    version,
    releaseUrl: release.html_url,
    changelogMarkdown: changelogSection || 'تغییرات و نصاب‌ها در صفحه رسمی ریلیز در دسترس هستند.',
  });

  if (isDryRun) {
    console.log('=== DRY RUN TELEGRAM RELEASE MESSAGE ===');
    console.log(messageText);
    console.log(`Length: ${messageText.length} characters`);
    process.exit(0);
  }

  // 6. Send message to Telegram Channel
  console.log(`Sending release announcement for ${release.tag_name} to Telegram channel ${chatId}...`);
  const telegramUrl = `https://api.telegram.org/bot${botToken}/sendMessage`;
  const telegramResponse = await fetch(telegramUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      chat_id: chatId,
      text: messageText,
      parse_mode: 'HTML',
      disable_web_page_preview: false,
    }),
  });

  const responseJson = await telegramResponse.json().catch(() => null);

  if (!telegramResponse.ok || !responseJson?.ok) {
    console.error(`::error::Telegram API sendMessage failed: ${telegramResponse.status}`);
    console.error('Response details:', JSON.stringify(responseJson, null, 2));
    process.exit(1);
  }

  console.log(`Successfully sent release announcement to Telegram! Message ID: ${responseJson.result.message_id}`);

  // 7. Update GitHub Release body with idempotency marker
  if (ghToken && releaseId) {
    const updatedBody = release.body ? `${release.body}\n\n${ANNOUNCED_MARKER}` : ANNOUNCED_MARKER;
    const patchResponse = await fetch(`https://api.github.com/repos/${repository}/releases/${releaseId}`, {
      method: 'PATCH',
      headers: {
        Accept: 'application/vnd.github+json',
        Authorization: `Bearer ${ghToken}`,
        'Content-Type': 'application/json',
        'X-GitHub-Api-Version': '2022-11-28',
      },
      body: JSON.stringify({
        body: updatedBody,
      }),
    });

    if (!patchResponse.ok) {
      console.warn('Warning: Failed to update release body with idempotency marker:', await patchResponse.text());
    } else {
      console.log('Successfully recorded idempotency marker in release body.');
    }
  }
}

if (process.argv[1] && import.meta.url.endsWith(process.argv[1].replace(/^.*\//, ''))) {
  main().catch((error) => {
    console.error('::error::Unhandled exception in send-telegram-release.mjs:', error);
    process.exit(1);
  });
}
