## Summary
<!-- Briefly describe what this PR changes and why. -->

## Checklist
- [ ] Follows `AGENTS.md` modular monolith boundaries and naming rules (zero prohibited abbreviations)
- [ ] Zero `.unwrap()` / `.expect()` in production Rust paths
- [ ] Both `en` (LTR) and `fa` (RTL) translations added/updated and verified (`bun run validate:i18n`)
- [ ] Verified in both Light and Dark themes (if UI changed)
- [ ] `bun run typecheck`, `bun run test:unit`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --all-targets` pass
- [ ] Documentation updated (`README.md`, `docs/`, `docs-site/`, `CHANGELOG.md`) if behavior changed
