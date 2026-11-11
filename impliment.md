# NotifyTrap — Product, Architecture, Engineering & Design Constitution

You are acting as a Senior Rust Engineer, Senior Tauri Engineer, Senior Nuxt/Vue Engineer, Software Architect, Product Designer, and Quality Engineer.

Your task is to design and implement **NotifyTrap**, a lightweight, local-first, open-source desktop application for capturing, inspecting, and debugging application notifications during development.

This is not a SaaS platform.
This is not a distributed infrastructure project.
This is not a microservices architecture.

NotifyTrap should feel like Postman, Mailpit, or a similar developer desktop utility: install it, open it, create a project, configure notification channels, connect your application, and inspect incoming notifications through a beautiful interface.

You are expected to implement a maintainable, production-quality application rather than a disposable prototype.

---

# PART 1 — NON-NEGOTIABLE ENGINEERING CONSTITUTION

The following rules are permanent engineering contracts.

They apply to every file, feature, refactoring operation, generated component, test, and future contribution.

Do not treat them as suggestions.

## 1.1. Naming and Readability

- Never use abbreviations in variable, function, method, class, component, module, or type names unless the abbreviation is an established technical term or protocol name.
- Prefer descriptive and explicit names.
- Avoid names such as `cfg`, `ctx`, `mgr`, `svc`, `repo`, `req`, `res`, `msg`, `usr`, or `val`.
- Prefer `configuration`, `context`, `notificationManager`, `notificationService`, `request`, `response`, and `notificationMessage`.
- Standard protocol names such as HTTP, SMTP, API, URL, UUID, JSON, HTML, CSS, SQL, and TCP are acceptable.
- Avoid ambiguous single-letter variables except for conventional mathematical or iterator contexts where appropriate.
- Functions must communicate their intention through their names.
- Boolean variables should use meaningful prefixes such as `is`, `has`, `can`, or `should`.

Code must be readable without requiring comments to explain unnecessarily cryptic naming.

## 1.2. No Hardcoded Values

Never scatter hardcoded values throughout the application.

Every value must belong to one of these categories:

1. An enum when it represents a finite, explicitly defined set of states or variants.
2. A configuration value when it should be configurable by the application or user.
3. A named constant when it represents an invariant.
4. A design token when it represents a visual design decision.
5. A translation key when it represents user-facing text.
6. A domain value when it belongs to a persisted business entity.

Examples:

- Notification channel types must be enums.
- Notification statuses must be enums.
- Default listener ports must be named configuration values.
- Timeouts must be configurable or named constants.
- Pagination defaults must be centralized.
- UI colors must come from design tokens.
- Spacing, radii, shadows, typography, and component sizes must come from the design system.
- User-facing messages must never be scattered as literal strings in application components.

Do not introduce unnecessary configuration complexity for values that are genuine invariants.

Avoid both arbitrary magic numbers and overengineering.

## 1.3. No God Files or God Components

Absolutely prohibit:

- God files.
- God classes.
- God components.
- Giant composables.
- Giant Rust modules.
- Excessively large functions.
- Business logic inside UI components.
- Database logic inside API handlers.
- Duplicated validation logic.

Every module must have a clear responsibility.

Split code based on domain boundaries and responsibilities, not arbitrary line counts.

A file should be understandable in isolation and should not require understanding the entire application.

## 1.4. Maintainability and Reusability

- Apply SOLID principles where they genuinely improve the design.
- Prefer composition over inheritance.
- Follow separation of concerns.
- Keep domain logic independent from infrastructure wherever practical.
- Use dependency inversion where it creates meaningful flexibility.
- Avoid premature abstractions.
- Do not create unnecessary interfaces for code that has only one implementation and no meaningful substitution requirement.
- Eliminate duplicated business logic.
- Build reusable UI components for recurring patterns.
- Keep reusable components configurable through explicit props and typed interfaces.
- Avoid unnecessary global state.
- Keep components focused and predictable.

The simplest maintainable solution is preferred over a sophisticated but unnecessarily complicated one.

## 1.5. Mandatory Documentation

Maintain the following project documents:

- `AGENTS.md` — Permanent engineering constitution and agent instructions.
- `README.md` — Product overview, installation, development, and usage.
- `docs/ARCHITECTURE.md` — System architecture and module responsibilities.
- `docs/DECISIONS.md` — Important architectural decisions and their rationale.
- `docs/DESIGN-SYSTEM.md` — Complete design system specification.
- `docs/TESTING.md` — Testing strategy and quality requirements.
- `docs/I18N.md` — Internationalization and translation rules.
- `docs/DEVELOPMENT.md` — Development environment and contribution workflow.

Update relevant documentation whenever architecture, behavior, or conventions change.

Do not allow documentation to become inconsistent with the implementation.

---

# PART 2 — MANDATORY SKILLS AND TECHNICAL RESEARCH

Before implementing the application, investigate and install relevant agent skills.

Do not assume existing skills are sufficient.

## 2.1. Skill Discovery

Inspect the currently available skills and identify relevant ones.

Prioritize skills covering:

### Rust
- Advanced Rust development.
- Rust ownership and borrowing.
- Async Rust and Tokio.
- Error handling and error propagation.
- Rust testing strategies.
- Rust API design and modular architecture.

### Tauri
- Tauri 2 architecture.
- Tauri commands and events.
- Tauri capabilities and permissions.
- Tauri plugin development.
- Native desktop application lifecycle.
- Secure IPC.
- Application packaging and distribution.

### Frontend
- Nuxt 4.
- Vue 3 Composition API.
- TypeScript strict mode.
- Nuxt UI.
- Tailwind CSS.
- Accessible component design.
- Frontend architecture and composables.

### UI/UX and Design
- Modern desktop application design.
- Design systems.
- Design tokens.
- Dark and light themes.
- RTL and LTR interface design.
- Accessibility.
- Responsive desktop layouts.
- Information architecture.
- Data-heavy interfaces and developer tools.

### Testing
- Rust unit and integration testing.
- Table-driven testing.
- Matrix-based testing.
- Property-based testing.
- Frontend component testing.
- End-to-end desktop testing.
- Accessibility testing.

## 2.2. Skill Installation Rules

- Inspect the available skill registry and documentation.
- Install only relevant, reputable, and compatible skills.
- Prefer official or well-maintained sources.
- Never install unknown scripts blindly.
- Never execute untrusted installation commands without inspecting them.
- Verify installed skills and their instructions.
- Use the installed skills throughout implementation.
- Do not merely install skills and ignore them.

If an appropriate skill cannot be found, use official documentation and document the missing capability.

## 2.3. Official Documentation First

Before making technical decisions, inspect current official documentation.

Prioritize:

- Rust: https://doc.rust-lang.org/
- Tokio: https://tokio.rs/
- Tauri 2: https://v2.tauri.app/
- Nuxt: https://nuxt.com/docs
- Vue: https://vuejs.org/
- Nuxt UI: https://ui.nuxt.com/
- Tailwind CSS: https://tailwindcss.com/docs
- SQLite: https://sqlite.org/docs.html

Check compatibility between framework versions and their dependencies.

Do not use outdated Tauri 1 examples or APIs.

Do not invent APIs, configuration properties, plugin capabilities, or package names.

---

# PART 3 — PRODUCT DEFINITION

## 3.1. Product Identity

Product name: NotifyTrap

Product category: Local-first Notification Testing and Debugging Desktop Application.

Product description:

"Capture, inspect, and debug application notifications in one local desktop workspace."

NotifyTrap allows developers to create isolated projects and configure notification channels.

Applications connect to these channels using familiar protocols or simple HTTP endpoints.

NotifyTrap captures incoming notifications and displays them in a beautiful, real-time interface.

The application must not send actual messages to real recipients.

All incoming notifications are captured locally.

## 3.2. Core Product Principles

1. Local-first.
2. Offline-capable.
3. Simple to install and use.
4. No mandatory cloud account.
5. No external infrastructure dependencies.
6. No real-world notification delivery.
7. Beautiful developer-focused interface.
8. Strong project and channel isolation.
9. Easy future extensibility.
10. Small and maintainable codebase.

## 3.3. Target User Journey

The primary workflow must be extremely simple:

1. Install NotifyTrap.
2. Double-click the application.
3. Create a project or tenant.
4. Create notification channels.
5. Obtain connection credentials and configuration.
6. Configure the user's development application.
7. Send notifications from that application.
8. See incoming notifications immediately inside NotifyTrap.
9. Inspect, search, filter, and debug them.

Example:

Workspace / Tenant: Arsam

Channels:

- Email
- SMS
- Push
- Webhook

A developer should not need to understand the internal architecture to use the application.

---

# PART 4 — MVP SCOPE

Keep the first version intentionally small.

Do not implement features outside the defined MVP without a documented reason and explicit prioritization.

## 4.1. Workspace / Tenant Management

Users must be able to:

- Create a tenant.
- Edit tenant details.
- Delete a tenant.
- View all tenants.
- Select an active tenant.

A tenant is a local logical namespace.

There is no cloud tenancy, user registration, team management, or billing system in the MVP.

## 4.2. Project Management

A tenant may contain multiple projects.

Users must be able to:

- Create projects.
- Edit projects.
- Delete projects.
- View project details.
- Select an active project.

Example:

Tenant: Arsam

Project: Arsam Backend

Project: Arsam Dashboard

Project: Arsam Mobile

Use explicit ownership relationships between tenants and projects.

## 4.3. Channel Management

Each project may contain multiple channels.

Supported MVP channels:

- Email
- SMS
- Push
- Webhook

Users must be able to:

- Create a channel.
- Select its type.
- Configure its name.
- Enable or disable it.
- View its connection settings.
- Regenerate its credentials.
- Delete it.
- View its operational status.

Each channel must have an isolated credential.

Do not confuse channel credentials with real SMS provider credentials, SMTP provider credentials, FCM credentials, or APNs credentials.

NotifyTrap credentials are local capture credentials.

## 4.4. Environment Configuration

The initial version should remain simple.

Support project environments such as:

- Local
- Development
- Testing
- Staging

Environment identifiers and defaults must be centralized.

Avoid building a complex environment-management platform.

A project may have separate channels or credentials per environment.

Ensure notification isolation between environments.

## 4.5. Email Capture

Implement an actual local SMTP server.

The SMTP server must:

- Listen on a configurable local address and port.
- Accept SMTP connections.
- Support standard email submission.
- Parse MIME messages.
- Extract headers.
- Extract sender and recipient information.
- Extract subject.
- Extract text/plain content.
- Extract text/html content.
- Handle multipart messages.
- Handle attachments.
- Preserve raw email source when practical.
- Associate messages with the correct project and channel.
- Persist received messages locally.

No external SMTP provider is required.

The application should support configuration similar to:

MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=arsam_email
MAIL_PASSWORD=generated_local_secret

Do not assume every incoming email will contain HTML.

Handle malformed and partially valid messages gracefully.

## 4.6. SMS Capture

Implement a local HTTP API.

Example:

POST /api/v1/sms

Headers:

X-Channel-Key: generated_channel_key

Content-Type: application/json

Payload:

{
  "to": "+989120000000",
  "message": "Your verification code is 123456"
}

The API must:

- Authenticate the channel.
- Validate the request.
- Associate the message with its tenant, project, environment, and channel.
- Persist the notification.
- Publish a local real-time update.
- Return a meaningful HTTP response.

SMS messages are captured only.

Never deliver them to real telephone numbers.

## 4.7. Push Notification Capture

Implement a local HTTP API.

Example:

POST /api/v1/push

Payload:

{
  "device_token": "sample-device-token",
  "platform": "android",
  "title": "Welcome",
  "body": "Your account is ready",
  "data": {
    "screen": "dashboard"
  }
}

Capture and inspect:

- Platform.
- Device token or device identifier.
- Title.
- Body.
- Custom data.
- Raw payload.
- Received timestamp.

Do not claim FCM or APNs protocol compatibility unless actual compatibility is implemented and tested.

Do not send push notifications to real devices.

## 4.8. Webhook Capture

Implement a local HTTP endpoint for capturing arbitrary incoming webhook requests.

Capture:

- HTTP method.
- Request path.
- Headers.
- Query parameters.
- Request body.
- Content type.
- Received timestamp.
- Request metadata.

Preserve the original payload where possible.

Support JSON and plain text initially.

Avoid unnecessarily restricting valid webhook payloads.

## 4.9. Unified Inbox

The Unified Inbox is the central feature of NotifyTrap.

It must show all incoming notifications belonging to the selected project and environment.

Features:

- Real-time incoming notifications.
- Channel filters.
- Environment filters.
- Search.
- Date filters.
- Notification details.
- Empty states.
- Loading states.
- Error states.
- Pagination or efficient incremental loading.

Each notification should display:

- Channel type.
- Recipient.
- Subject or message preview.
- Source.
- Received timestamp.
- Relevant status.
- Project and environment context.

Use channel-specific presentation components.

Do not force all notification types into an identical visual representation.

## 4.10. Notification Inspector

When selecting a notification, provide a detailed inspector.

Email:

- HTML preview.
- Plain text preview.
- Headers.
- Sender.
- Recipients.
- Attachments.
- Raw MIME source.

SMS:

- Recipient.
- Message content.
- Sender metadata when available.
- Payload.

Push:

- Device information.
- Platform.
- Title.
- Body.
- Data payload.

Webhook:

- HTTP method.
- URL path.
- Headers.
- Query parameters.
- Request body.

Provide convenient copy actions for relevant technical values.

---

# PART 5 — TECHNICAL ARCHITECTURE

## 5.1. Technology Stack

Use the following technologies:

### Desktop
- Tauri 2.
- Rust.

### Frontend
- Nuxt 4.
- Vue 3.
- TypeScript.
- Nuxt UI.
- Tailwind CSS.

### Local Persistence
- SQLite.

### Local Networking
- Rust-based SMTP server.
- Axum for HTTP APIs.
- Tokio for asynchronous execution.

### Testing
- Rust unit and integration tests.
- Table-driven tests.
- Matrix-based tests.
- Frontend component tests.
- End-to-end tests.

Verify the current compatible versions before installation.

Use stable releases unless there is a compelling reason otherwise.

## 5.2. Architecture Principles

Use a modular monolith.

Do not introduce microservices.

Do not introduce PostgreSQL, Redis, RabbitMQ, Kafka, Kubernetes, or external infrastructure for the MVP.

The application must run as one desktop application with local background services.

The Rust backend is responsible for:

- Application lifecycle.
- Local SMTP server.
- Local HTTP server.
- Credential validation.
- Notification processing.
- Persistence.
- Database migrations.
- Background tasks.
- Realtime event publishing.
- Secure communication with the frontend.

The frontend is responsible for:

- User interface.
- Navigation.
- User interactions.
- Form validation feedback.
- Notification visualization.
- Design system.
- Localization.
- Theme management.

Keep business logic out of Vue components.

Keep database operations out of Tauri commands.

Tauri commands should be thin application boundaries.

## 5.3. Suggested Rust Structure

Use domain-oriented modular architecture.

Suggested structure:

src-tauri/src/

- main.rs
- lib.rs

- application/
  - application_state.rs
  - application_lifecycle.rs

- domain/
  - tenant/
  - project/
  - environment/
  - channel/
  - notification/

- infrastructure/
  - database/
  - smtp/
  - http/
  - security/
  - migrations/

- services/
  - tenant_service.rs
  - project_service.rs
  - channel_service.rs
  - notification_service.rs

- commands/
  - tenant_commands.rs
  - project_commands.rs
  - channel_commands.rs
  - notification_commands.rs

- events/
  - notification_events.rs
  - application_events.rs

- configuration/
  - application_configuration.rs
  - server_configuration.rs

This structure is illustrative.

Refine it based on actual domain responsibilities.

Avoid unnecessary file proliferation.

Every module must have a clearly defined purpose.

## 5.4. Suggested Frontend Structure

Use Nuxt 4 conventions and its recommended directory structure.

Keep a clear separation between:

- Pages.
- Layouts.
- Components.
- Composables.
- Domain models.
- API or IPC clients.
- Stores.
- Utilities.
- Localization.
- Design system.

Organize components by feature when appropriate.

Example conceptual organization:

app/

- components/
  - layout/
  - shared/
  - tenant/
  - project/
  - channel/
  - notification/
  - design-system/

- composables/
- layouts/
- pages/
- stores/
- types/
- utils/

Do not put every component inside one shared directory.

Avoid creating huge page components.

Extract reusable behavior into composables.

Keep frontend domain types consistent with backend contracts.

## 5.5. Communication Between Rust and Vue

Use Tauri commands for request-response interactions.

Use Tauri events for real-time updates.

Do not expose unnecessary backend functionality to the frontend.

Validate all incoming command arguments.

Use typed request and response structures.

Avoid unstructured stringly typed communication.

Do not expose arbitrary filesystem access or unrestricted shell execution.

## 5.6. Background Service Lifecycle

SMTP and HTTP servers must run as managed background services.

They must:

- Start when appropriate.
- Stop cleanly when the application exits.
- Support graceful shutdown.
- Handle port conflicts.
- Report their status.
- Report meaningful startup errors.
- Avoid blocking the UI thread.
- Avoid uncontrolled background task creation.

Bind local capture servers to loopback addresses by default.

Do not expose the servers to the public network unless a user explicitly configures a different binding address.

Do not assume localhost inside another Docker container refers to the desktop host.

Document Docker-to-host connectivity as a future integration concern.

## 5.7. Persistence

Use SQLite with proper migrations.

Requirements:

- Explicit database schema.
- Foreign key enforcement.
- Appropriate indexes.
- Transactional operations where necessary.
- Safe database initialization.
- Migration version tracking.
- Proper error handling.
- Referential integrity.
- Tenant and project isolation.

Do not store generated credentials as plaintext unless there is a documented, unavoidable reason.

Prefer operating-system credential storage when available.

Otherwise, use a documented secure local secret-storage strategy.

Sensitive notification payloads must be treated as potentially confidential.

Provide safe deletion and retention foundations.

---

# PART 6 — INTERNATIONALIZATION CONTRACT

Internationalization is a first-class architectural requirement.

Supported initial languages:

1. English: en
2. Persian: fa

The architecture must make adding a third language straightforward.

## 6.1. Mandatory Rules

- Never hardcode user-facing strings inside components.
- Never leave untranslated buttons, labels, tooltips, notifications, dialogs, validation messages, or empty states.
- Every feature must be implemented in both languages.
- Every new translation key must be added to every supported locale.
- Translation keys must be organized by feature and meaning.
- Avoid using one generic translation key for unrelated meanings.
- Avoid building UI components around English text lengths.
- Support dynamic text interpolation.
- Support pluralization where appropriate.
- Support locale-aware dates and numbers.

## 6.2. RTL and LTR

Persian must use RTL direction.

English must use LTR direction.

Direction must be controlled centrally by the localization system.

Do not manually reverse individual components to achieve RTL.

Use logical layout properties wherever possible.

Avoid unnecessary left/right-specific CSS.

Examples of logical properties:

- margin-inline-start.
- margin-inline-end.
- padding-inline-start.
- padding-inline-end.
- text-align: start.
- text-align: end.

Verify all Nuxt UI components in RTL mode.

Verify:

- Sidebar placement.
- Navigation.
- Dropdown menus.
- Dialogs.
- Tables.
- Icons.
- Breadcrumbs.
- Forms.
- Notification previews.
- Keyboard navigation.

## 6.3. Translation Completeness Enforcement

Implement automated translation validation.

The validation process must detect:

- Missing keys.
- Extra keys.
- Empty translations.
- Invalid interpolation variables.
- Incorrect locale structures.

CI must fail when a required translation is missing.

A feature is not complete until both English and Persian translations are implemented and verified.

---

# PART 7 — DESIGN SYSTEM CONSTITUTION

UI/UX quality is one of the highest priorities of NotifyTrap.

Do not create a generic dashboard.

Design a premium, modern, polished desktop developer tool.

The interface should feel deliberate, consistent, fast, and professional.

## 7.1. Design Direction

Visual direction:

- Modern developer utility.
- Premium desktop application.
- Minimal but expressive.
- Information-dense without visual clutter.
- Excellent visual hierarchy.
- Carefully balanced whitespace.
- Subtle borders.
- Restrained shadows.
- Clear interactive feedback.
- High-quality typography.
- Consistent component behavior.

Avoid:

- Excessive gradients.
- Unnecessary glassmorphism.
- Overuse of colored backgrounds.
- Excessive rounded cards.
- Excessive animation.
- Generic AI-generated dashboard aesthetics.
- Inconsistent spacing.
- Arbitrary decorative elements.

The application must look like a carefully designed professional desktop product.

## 7.2. Mandatory Design System Page

Create a dedicated Design System page inside the application.

Suggested route:

`/design-system`

This page must demonstrate the complete visual language and all reusable UI primitives.

It is not merely a documentation page.

It is the visual source of truth for the implemented design system.

Include the following sections.

### A. Color System

Define complete semantic color tokens.

Include:

- Primary.
- Secondary.
- Accent.
- Background.
- Foreground.
- Surface.
- Elevated surface.
- Muted surface.
- Border.
- Input.
- Focus.
- Ring.
- Success.
- Warning.
- Error.
- Information.
- Disabled.
- Overlay.

Include all necessary shades and semantic variations.

Use consistent naming.

Avoid arbitrary hexadecimal values in components.

Ensure both light and dark themes have complete and carefully balanced palettes.

All text/background combinations must meet appropriate accessibility contrast requirements.

### B. Typography System

Define:

- Font families.
- Font weights.
- Font sizes.
- Line heights.
- Letter spacing.
- Heading scale.
- Body scale.
- Label scale.
- Caption scale.
- Monospace typography.

Provide a suitable Persian font with an appropriate fallback strategy.

Ensure Persian and English typography both look intentional.

Use a consistent typography scale.

### C. Spacing System

Define a centralized spacing scale.

Include:

- 0.
- 1.
- 2.
- 3.
- 4.
- 6.
- 8.
- 10.
- 12.
- 16.
- 20.
- 24.
- 32.
- 40.
- 48.

Use the project's established token conventions and document actual rendered values.

Avoid arbitrary spacing.

### D. Radius System

Define semantic radius tokens:

- None.
- Small.
- Medium.
- Large.
- Extra large.
- Full.

Demonstrate them visually.

### E. Shadow System

Define:

- None.
- Subtle.
- Surface elevation.
- Dropdown elevation.
- Dialog elevation.
- Focus elevation where appropriate.

Keep shadows restrained and consistent.

### F. Sizing System

Define:

- Control heights.
- Button sizes.
- Icon sizes.
- Input heights.
- Sidebar widths.
- Dialog widths.
- Popover dimensions.
- Content maximum widths.
- Layout constraints.

### G. Motion System

Define:

- Fast transition.
- Standard transition.
- Slow transition.
- Easing functions.
- Hover transitions.
- Focus transitions.
- Dialog transitions.
- Loading transitions.

Respect reduced-motion preferences.

Do not animate everything.

### H. Component Showcase

Show all relevant components in different states:

- Buttons.
- Icon buttons.
- Inputs.
- Textareas.
- Selects.
- Checkboxes.
- Radio groups.
- Switches.
- Badges.
- Avatars.
- Tooltips.
- Dropdowns.
- Popovers.
- Dialogs.
- Confirmation dialogs.
- Tabs.
- Tables.
- Cards.
- Alerts.
- Toasts.
- Progress indicators.
- Skeleton loaders.
- Empty states.
- Error states.
- Navigation items.
- Breadcrumbs.
- Pagination.
- Search controls.

Show default, hover, focus, active, disabled, loading, error, and success states wherever applicable.

### I. Theme Demonstration

Provide a way to inspect the design system in:

- Light mode.
- Dark mode.
- English LTR.
- Persian RTL.

The design system page must prove that the entire system supports both directions and themes.

## 7.3. Nuxt UI First

Nuxt UI is the mandatory first choice for UI components.

Before implementing a custom component:

1. Inspect the current Nuxt UI component catalog.
2. Check whether an appropriate component already exists.
3. Use the official component when suitable.
4. Customize it through supported props, slots, variants, and design tokens.

Only implement custom components when Nuxt UI does not provide the required functionality or when its component cannot satisfy a genuine product requirement.

Do not unnecessarily replace the Nuxt UI component system.

Do not introduce another UI component framework.

## 7.4. Avoid Custom CSS

Custom styles must be minimized.

Prefer:

- Nuxt UI components.
- Tailwind utility classes.
- Centralized design tokens.
- Component variants.
- Reusable layout primitives.

Avoid:

- Repeated custom CSS.
- Arbitrary CSS values.
- Inline styling.
- Component-specific style hacks.
- Large global stylesheets.

If a custom style is necessary, document why it cannot be achieved using the existing design system.

Never sacrifice maintainability for visual shortcuts.

## 7.5. Light and Dark Themes

Both themes are mandatory.

They must be independently designed and fully supported.

Do not implement dark mode by simply inverting light colors.

Ensure that both themes have appropriate:

- Background hierarchy.
- Surface hierarchy.
- Text contrast.
- Borders.
- Shadows.
- Focus states.
- Status colors.
- Interactive states.
- Disabled states.

The selected theme must persist across application restarts.

Theme switching must not cause layout flickering.

## 7.6. Accessibility

Follow WCAG 2.2 AA principles wherever applicable.

Ensure:

- Keyboard navigation.
- Visible focus indicators.
- Semantic controls.
- Accessible labels.
- Appropriate contrast.
- Screen-reader support.
- Reduced-motion support.
- Meaningful tooltips.
- Proper dialog focus management.

---

# PART 8 — APPLICATION INFORMATION ARCHITECTURE

Design the application navigation around these areas:

1. Overview.
2. Inbox.
3. Tenants.
4. Projects.
5. Channels.
6. Settings.
7. Design System.

The primary experience must focus on Inbox.

The interface should make it immediately obvious:

- Which tenant is active.
- Which project is active.
- Which environment is active.
- Which channels are running.
- Whether local capture services are operational.

Use a compact, professional desktop layout.

Do not overcomplicate navigation.

## 8.1. Overview

Show:

- Active tenant.
- Active project.
- Channel health.
- SMTP status.
- HTTP server status.
- Total received notifications.
- Recent notifications.
- Quick actions.

## 8.2. Inbox

Provide:

- Notification list.
- Channel filters.
- Search.
- Environment filter.
- Notification details.
- Realtime updates.

## 8.3. Settings

Include:

- Appearance.
- Language.
- Server configuration.
- Database information.
- Application information.

---

# PART 9 — TESTING CONSTITUTION

Testing is mandatory.

A feature is not complete simply because it compiles or works manually.

Every meaningful piece of business logic must have automated tests.

The testing strategy must be comprehensive and systematic.

## 9.1. Table-Driven Testing

Use table-driven tests wherever multiple input/output combinations exist.

This is especially important for:

- Credential validation.
- Tenant isolation.
- Project ownership validation.
- Environment resolution.
- Channel validation.
- Notification parsing.
- Payload validation.
- SMTP message parsing.
- MIME parsing.
- HTTP request handling.
- Error mapping.
- Localization validation.
- Database repository behavior.

Tests should use descriptive test case names.

Example Rust structure:

```rust
struct CredentialValidationTestCase {
    name: &'static str,
    provided_credential: &'static str,
    expected_result: bool,
}

#[test]
fn validates_channel_credentials() {
    let test_cases = vec![
        CredentialValidationTestCase {
            name: "accepts_valid_credential",
            provided_credential: "valid-secret",
            expected_result: true,
        },
        CredentialValidationTestCase {
            name: "rejects_invalid_credential",
            provided_credential: "invalid-secret",
            expected_result: false,
        },
        CredentialValidationTestCase {
            name: "rejects_empty_credential",
            provided_credential: "",
            expected_result: false,
        },
    ];

    for test_case in test_cases {
        // Execute and assert the expected result.
    }
}
```

This is illustrative.

Use idiomatic Rust and appropriate testing libraries.

## 9.2. Matrix-Based Testing

Create test matrices for combinations of relevant dimensions.

Examples:

### Channel Matrix

- Email.
- SMS.
- Push.
- Webhook.

Test:

- Valid payload.
- Invalid payload.
- Missing required fields.
- Invalid credentials.
- Disabled channel.
- Deleted channel.
- Wrong environment.
- Persistence failure.

### Locale Matrix

- English LTR.
- Persian RTL.

Verify that every relevant page and component works in both directions.

### Theme Matrix

- Light.
- Dark.

Verify all important states in both themes.

### Tenant Isolation Matrix

- Same tenant, same project.
- Same tenant, different project.
- Different tenants.
- Same channel type across tenants.
- Invalid project/channel combination.
- Wrong environment credential.

Ensure that cross-tenant notification access is impossible.

### HTTP Matrix

Test:

- GET.
- POST.
- Unsupported methods.
- Valid JSON.
- Invalid JSON.
- Empty payload.
- Oversized payload.
- Missing credentials.
- Invalid credentials.
- Unknown endpoint.

### SMTP Matrix

Test:

- Plain text email.
- HTML email.
- Multipart email.
- Multiple recipients.
- Attachments.
- Invalid MIME.
- Missing subject.
- Empty body.
- Malformed message.
- Authentication failures.

### Persistence Matrix

Test:

- Insert.
- Read.
- Update.
- Delete.
- Missing entity.
- Invalid foreign key.
- Database unavailable.
- Migration failure.
- Transaction rollback.

## 9.3. Property-Based Testing

Use property-based testing for suitable domain logic.

Prioritize:

- Payload normalization.
- Credential validation.
- Parser edge cases.
- Identifier validation.
- Serialization/deserialization.
- MIME parsing invariants.

Use an appropriate Rust testing library when beneficial.

## 9.4. Integration Testing

Integration tests must cover actual behavior rather than merely mocking every dependency.

At minimum:

- Start the local HTTP server.
- Send a real HTTP request.
- Verify authentication.
- Verify database persistence.
- Verify emitted events.
- Start the SMTP server.
- Submit an actual SMTP message.
- Verify parsed message persistence.
- Verify project and tenant isolation.

Use temporary database instances or isolated test databases.

Tests must not depend on the developer's personal environment.

## 9.5. Frontend Testing

Test:

- Component rendering.
- User interactions.
- Form validation.
- Loading states.
- Error states.
- Empty states.
- Theme behavior.
- Locale switching.
- RTL behavior.
- Search and filtering.
- Notification detail rendering.

## 9.6. End-to-End Testing

Create critical end-to-end scenarios:

1. Launch application.
2. Create tenant.
3. Create project.
4. Create Email channel.
5. Copy SMTP credentials.
6. Submit email through SMTP.
7. Verify email appears in Inbox.
8. Open email inspector.
9. Switch to Persian.
10. Verify RTL layout.
11. Switch to dark mode.
12. Verify persistence after restarting the application.

Create corresponding scenarios for SMS, Push, and Webhook capture.

## 9.7. Test Quality Rules

- No meaningless tests.
- No tests that only assert that a value is not null without meaningful justification.
- No tests that simply repeat implementation details.
- No arbitrary sleeps in asynchronous tests.
- Use deterministic fixtures.
- Avoid shared mutable test state.
- Keep tests independent.
- Use descriptive test names.
- Test failure paths as carefully as success paths.

The test suite must run locally and in CI.

Document the test commands.

---

# PART 10 — SECURITY REQUIREMENTS

Even though NotifyTrap is local-first, security matters.

Implement:

- Loopback-only server binding by default.
- Per-channel credentials.
- Credential rotation.
- Secure secret storage.
- Input validation.
- Request size limits.
- Safe error responses.
- Safe HTML preview isolation.
- Protection against accidental external network delivery.
- No arbitrary code execution from captured payloads.
- No automatic execution of HTML or JavaScript from received emails.
- Safe attachment handling.
- Tenant and project isolation.

Treat notification content as untrusted input.

Email HTML must be rendered in a sandboxed environment with scripts disabled and remote resources blocked by default.

Do not expose captured notification content through unsafe HTML injection.

Do not log secrets or sensitive payloads unnecessarily.

---

# PART 11 — PERFORMANCE AND RELIABILITY

The application should feel lightweight and responsive.

Requirements:

- Fast application startup.
- Efficient database queries.
- Indexed search fields.
- Efficient notification list rendering.
- Realtime updates without unnecessary full-page refreshes.
- Graceful error handling.
- Background service health reporting.
- Clean shutdown.
- No blocking operations on the UI thread.
- No uncontrolled memory growth.

Do not load every stored notification into memory.

Use pagination or incremental loading.

Do not prematurely optimize without profiling.

---

# PART 12 — DEVELOPMENT WORKFLOW

You must work in controlled, verifiable phases.

Do not implement the entire application in one massive operation.

## Phase 0 — Repository and Skill Audit

- Inspect the repository.
- Inspect existing files.
- Inspect installed skills.
- Discover and install relevant skills.
- Read official technical documentation.
- Verify dependency compatibility.
- Identify existing constraints.
- Create an implementation plan.

Do not overwrite existing project work without understanding it.

## Phase 1 — Architecture and Foundation

- Establish project structure.
- Configure Tauri 2.
- Configure Rust.
- Configure Nuxt 4.
- Configure TypeScript strict mode.
- Configure Nuxt UI.
- Configure Tailwind.
- Establish design tokens.
- Establish localization infrastructure.
- Establish SQLite migrations.
- Create AGENTS.md.
- Create architecture documentation.

Deliver a working desktop application shell.

## Phase 2 — Design System

Implement the full design system before building feature pages.

Create:

- Color tokens.
- Typography tokens.
- Spacing tokens.
- Radius tokens.
- Shadow tokens.
- Sizing tokens.
- Motion tokens.
- Semantic component variants.
- Light theme.
- Dark theme.
- English LTR.
- Persian RTL.

Implement the Design System showcase page.

Verify all components visually.

## Phase 3 — Domain and Persistence

Implement:

- Tenant.
- Project.
- Environment.
- Channel.
- Notification.

Implement database migrations and repositories.

Add table-driven and matrix-based tests.

## Phase 4 — Local Capture Infrastructure

Implement:

- Local SMTP server.
- Local HTTP server.
- Channel credentials.
- Email capture.
- SMS capture.
- Push capture.
- Webhook capture.

Add real integration tests.

## Phase 5 — Desktop User Experience

Implement:

- Tenant management.
- Project management.
- Channel management.
- Connection configuration.
- Unified Inbox.
- Notification Inspector.
- Realtime updates.
- Settings.

Implement all screens in both Persian and English and in both themes.

## Phase 6 — Hardening

- Security audit.
- Performance review.
- Error handling review.
- Test matrix completion.
- Accessibility review.
- Localization completeness review.
- RTL review.
- Dark/light theme review.
- Application packaging.

## Phase 7 — Documentation and Release

Prepare:

- README.
- Installation guide.
- Architecture documentation.
- Development guide.
- API examples.
- SMTP configuration examples.
- Testing guide.
- Contribution guide.

---

# PART 13 — DEFINITION OF DONE

A feature is not complete until all applicable requirements are satisfied.

Before marking any task complete, verify:

1. Implementation follows the architecture.
2. Naming conventions are respected.
3. No unexplained hardcoded values exist.
4. No God files or components were introduced.
5. Business logic is properly separated.
6. Reusable components are used where appropriate.
7. Nuxt UI is used wherever suitable.
8. English translation exists.
9. Persian translation exists.
10. RTL behavior is verified.
11. Light theme is verified.
12. Dark theme is verified.
13. Loading state exists where needed.
14. Empty state exists where needed.
15. Error state exists where needed.
16. Relevant table-driven tests exist.
17. Relevant matrix tests exist.
18. Integration tests exist where required.
19. Documentation is updated.
20. No regressions were introduced.

Never claim a feature is complete based only on successful compilation.

Run the relevant tests and report their actual results.

---

# PART 14 — AGENT WORKING RULES

These instructions apply throughout the entire project.

- Treat this document as the permanent engineering constitution.
- Read `AGENTS.md` before beginning every new implementation task.
- If `AGENTS.md` does not exist, create it using the relevant rules in this document.
- Preserve these rules across future conversations and sessions.
- Do not silently weaken engineering requirements.
- Do not remove tests simply to make a build pass.
- Do not suppress compiler warnings without understanding their cause.
- Do not use `unwrap()` or `expect()` in production Rust paths without a compelling, documented invariant.
- Do not introduce dependencies without evaluating their purpose and maintenance status.
- Do not introduce unnecessary abstractions.
- Do not make unrelated changes.
- Do not silently change architectural decisions.
- Document meaningful trade-offs.
- Prefer official documentation over outdated tutorials.
- Verify APIs and dependency versions.
- Run formatting, linting, type checking, and tests after meaningful changes.

If a requirement is technically impossible or incompatible with another requirement, explain the conflict and propose the smallest reasonable alternative.

Never pretend that an unimplemented capability exists.

Do not claim that an API is compatible with SMTP, FCM, APNs, or a specific provider unless that compatibility is genuinely implemented and tested.

---

# PART 15 — FIRST EXECUTION INSTRUCTIONS

Start now.

Do not immediately generate a large amount of application code.

First perform these actions:

1. Inspect the current repository and its existing structure.
2. Inspect the available agent skills.
3. Install the relevant Rust, Tauri, Nuxt, UI/UX, design system, i18n, and testing skills.
4. Read the official documentation relevant to the selected technology versions.
5. Create or update `AGENTS.md` with the permanent engineering constitution.
6. Create an implementation roadmap.
7. Define the initial architecture.
8. Define the database entities and their relationships.
9. Define the API contracts for Email, SMS, Push, and Webhook capture.
10. Define the complete initial design token system.
11. Define the English and Persian localization structure.
12. Identify technical risks and unresolved decisions.

Then begin implementation incrementally.

Prioritize a high-quality foundation over superficial feature completeness.

After every implementation phase:

- Run relevant tests.
- Run formatting.
- Run linting.
- Run type checking.
- Verify translations.
- Verify theme behavior.
- Verify RTL behavior.
- Update documentation.
- Report completed work and remaining work.

The final objective is a polished, reliable, maintainable, open-source desktop application.

**Build a small product exceptionally well. Do not turn a small product into a large platform.**