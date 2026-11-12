import { describe, expect, it } from 'vitest';
import {
  buildTelegramReleaseMessage,
  extractChangelogForVersion,
  formatMarkdownToTelegramHtml,
} from '../scripts/send-telegram-release.mjs';

describe('Telegram Release Notifier', () => {
  const sampleChangelog = `
# Changelog

## [1.4.0] - 2026-11-12

### Added
- **Feature A**: Description with \`<tag>\` and & symbol and [link](https://notifytrap.org).
- **Feature B**: Another item.

### Fixed
- **Bug 1**: Fixed issue in \`auth\`.

## [1.3.0] - 2026-11-11

### Added
- **Old Feature**: Something else.
`;

  it('extracts changelog specifically for target version', () => {
    const extracted = extractChangelogForVersion(sampleChangelog, '1.4.0');
    expect(extracted).toContain('### Added');
    expect(extracted).toContain('Feature A');
    expect(extracted).toContain('Bug 1');
    expect(extracted).not.toContain('Old Feature');
    expect(extracted).not.toContain('## [1.3.0]');
  });

  it('formats markdown to telegram-safe HTML with escaping', () => {
    const formatted = formatMarkdownToTelegramHtml(`### Added
- **Feature A**: Description with \`<tag>\` and & symbol and [doc link](https://notifytrap.org).
### Fixed
- **Bug 1**: Fixed issue in \`domain/mod.rs\`.`);

    expect(formatted).toContain('✨ <b>قابلیت‌های جدید (Added):</b>');
    expect(formatted).toContain('🐛 <b>رفع باگ‌ها (Fixed):</b>');
    expect(formatted).toContain('• <b>Feature A</b>: Description with <code>&lt;tag&gt;</code> and &amp; symbol and <a href="https://notifytrap.org">doc link</a>.');
    expect(formatted).toContain('• <b>Bug 1</b>: Fixed issue in <code>domain/mod.rs</code>.');
  });

  it('builds complete message within telegram character limit', () => {
    const message = buildTelegramReleaseMessage({
      version: '1.4.0',
      releaseUrl: 'https://github.com/farshidrezaei/notifytrap/releases/tag/v1.4.0',
      changelogMarkdown: `- **Feature**: Test item`,
    });

    expect(message).toContain('🚀 <b>NotifyTrap v1.4.0 منتشر شد!</b>');
    expect(message).toContain('https://github.com/farshidrezaei/notifytrap/releases/tag/v1.4.0');
    expect(message).toContain('@notifytrap');
    expect(message.length).toBeLessThan(4096);
  });

  it('truncates excessively long changelogs gracefully', () => {
    const hugeMarkdown = Array.from({ length: 200 })
      .map((_, index) => `- **Item ${index}**: Long descriptive changelog line that takes up space.`)
      .join('\n');

    const message = buildTelegramReleaseMessage({
      version: '1.4.0',
      releaseUrl: 'https://github.com/farshidrezaei/notifytrap/releases/tag/v1.4.0',
      changelogMarkdown: hugeMarkdown,
    });

    expect(message.length).toBeLessThan(4096);
    expect(message).toContain('مشاهده متن کامل در گیت‌هاب');
  });
});
