use std::sync::Arc;

use proptest::prelude::*;
use serde_json::json;

use notifytrap_lib::application::application_state::ApplicationServices;
use notifytrap_lib::configuration::server_configuration::ServerConfiguration;
use notifytrap_lib::domain::channel::{ChannelType, CreateChannelInput};
use notifytrap_lib::domain::environment::EnvironmentKind;
use notifytrap_lib::domain::errors::ApplicationError;
use notifytrap_lib::domain::notification::{
    CaptureAttachmentInput, CaptureNotificationInput, NotificationQuery, NotificationStatus,
};
use notifytrap_lib::domain::project::CreateProjectInput;
use notifytrap_lib::domain::validation::{
    derive_slug_or_validate, normalize_unicode_digits, validate_bind_address, validate_description,
    validate_name, validate_pagination, validate_retention_days, validate_search_query,
    validate_slug,
};
use notifytrap_lib::domain::workspace::CreateWorkspaceInput;
use notifytrap_lib::infrastructure::database::connection_manager::DatabaseConnectionManager;
use notifytrap_lib::infrastructure::security::credential_crypto::{
    decrypt_secret, encrypt_secret, hash_secret_hex, verify_secret_constant_time,
};
use notifytrap_lib::infrastructure::security::html_sanitizer::sanitize_email_html;
use notifytrap_lib::infrastructure::security::secret_key_provider::EphemeralSecretKeyProvider;

fn create_test_services() -> ApplicationServices {
    let database_manager =
        DatabaseConnectionManager::open_in_memory().expect("in-memory database should initialize");
    let key_provider = Arc::new(EphemeralSecretKeyProvider::new());
    ApplicationServices::new(database_manager, key_provider)
}

#[test]
fn table_driven_validation_rules() {
    let valid_names = ["Ab", "Arsam Studio", "NotifyTrap Workspace 01"];
    for name in valid_names {
        assert!(validate_name(name).is_ok(), "expected valid name: {name}");
    }

    let invalid_names = ["", " ", "A"];
    for name in invalid_names {
        assert!(
            matches!(validate_name(name), Err(ApplicationError::Validation(_))),
            "expected validation error for name: {name}"
        );
    }

    let invalid_slugs = [
        "",
        "a",
        "-abc",
        "abc-",
        "ab--cd",
        "ab_cd",
        "ab cd",
        "Upper-Case",
        "سلام",
        "app-۱۲",
    ];
    for slug in invalid_slugs {
        assert!(
            matches!(validate_slug(slug), Err(ApplicationError::Validation(_))),
            "expected validation error for slug: {slug}"
        );
    }

    assert_eq!(
        normalize_unicode_digits("۰۱۲۳۴۵۶۷۸۹٠١٢٣٤٥٦٧٨٩०१२३४५६७८९"),
        "012345678901234567890123456789"
    );
    assert_eq!(
        validate_bind_address("۱۲۷.۰.۰.۱").expect("Persian IPv4 digits should normalize"),
        "127.0.0.1"
    );
    assert_eq!(
        validate_bind_address("١٩٢.١٦٨.١.٢٥").expect("Arabic IPv4 digits should normalize"),
        "192.168.1.25"
    );
    assert_eq!(
        validate_bind_address("१२७.०.०.१").expect("Hindi IPv4 digits should normalize"),
        "127.0.0.1"
    );
    assert!(validate_bind_address("لوکال").is_err());
    assert_eq!(
        derive_slug_or_validate(None, "سرویس ۱۲")
            .expect("Unicode digits in name should derive valid ASCII slug"),
        "12"
    );
}

#[test]
fn sanitizes_untrusted_email_html_against_xss_vectors() {
    let malicious = r#"
        <div>
            <h1>Welcome</h1>
            <script>alert('xss')</script>
            <img src="https://tracker.example.com/pixel.png" onerror="fetch('evil')" />
            <a href="javascript:alert(1)" onclick="steal()">Click</a>
            <a href="https://example.com/docs">Safe Link</a>
            <iframe src="https://evil.example.com"></iframe>
        </div>
    "#;

    let cleaned = sanitize_email_html(malicious);
    assert!(!cleaned.contains("<script"));
    assert!(!cleaned.contains("onerror"));
    assert!(!cleaned.contains("onclick"));
    assert!(!cleaned.contains("javascript:"));
    assert!(!cleaned.contains("<iframe"));
    assert!(!cleaned.contains("<img"));
    assert!(cleaned.contains("<h1>Welcome</h1>"));
    assert!(cleaned.contains("https://example.com/docs"));
}

#[test]
fn creates_project_with_default_environments_and_switches_default() {
    let services = create_test_services();
    let workspace = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Arsam Studio".to_string(),
            slug: None,
            description: Some("Primary local workspace".to_string()),
        })
        .expect("workspace should be created");
    assert_eq!(workspace.slug, "arsam-studio");

    let project = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace.id.clone(),
            name: "Backend API".to_string(),
            slug: None,
            description: None,
        })
        .expect("project should be created");

    let environments = services
        .environments
        .list_environments_by_project(&project.id)
        .expect("environments should list");
    assert_eq!(environments.len(), 4);

    let default_environments: Vec<_> = environments.iter().filter(|item| item.is_default).collect();
    assert_eq!(default_environments.len(), 1);
    assert_eq!(default_environments[0].kind, EnvironmentKind::Local);

    let staging = environments
        .iter()
        .find(|item| item.kind == EnvironmentKind::Staging)
        .expect("staging environment must exist");

    let updated_environments = services
        .environments
        .set_default_environment(&project.id, &staging.id)
        .expect("switching default environment should succeed");

    let new_defaults: Vec<_> = updated_environments
        .iter()
        .filter(|item| item.is_default)
        .collect();
    assert_eq!(new_defaults.len(), 1);
    assert_eq!(new_defaults[0].id, staging.id);
}

#[test]
fn matrix_channel_credentials_auth_rotation_and_scope_isolation() {
    let services = create_test_services();

    let workspace_alpha = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Alpha Corp".to_string(),
            slug: Some("alpha".to_string()),
            description: None,
        })
        .expect("alpha workspace");
    let workspace_beta = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Beta Corp".to_string(),
            slug: Some("beta".to_string()),
            description: None,
        })
        .expect("beta workspace");

    // Identical project slug across distinct workspaces is allowed cleanly.
    let project_alpha = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace_alpha.id.clone(),
            name: "Core App".to_string(),
            slug: Some("core-app".to_string()),
            description: None,
        })
        .expect("alpha project");
    let project_beta = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace_beta.id.clone(),
            name: "Core App".to_string(),
            slug: Some("core-app".to_string()),
            description: None,
        })
        .expect("beta project");

    // Duplicate project slug inside the same workspace must fail with Conflict.
    let duplicate_project = services.projects.create_project(CreateProjectInput {
        workspace_id: workspace_alpha.id.clone(),
        name: "Core App Copy".to_string(),
        slug: Some("core-app".to_string()),
        description: None,
    });
    assert!(matches!(
        duplicate_project,
        Err(ApplicationError::Conflict(_))
    ));

    let alpha_environments = services
        .environments
        .list_environments_by_project(&project_alpha.id)
        .expect("alpha envs");
    let beta_environments = services
        .environments
        .list_environments_by_project(&project_beta.id)
        .expect("beta envs");
    let alpha_local = &alpha_environments[0];
    let beta_local = &beta_environments[0];

    // Cross-workspace channel creation must fail validation.
    let mismatched_channel = services.channels.create_channel(CreateChannelInput {
        workspace_id: workspace_alpha.id.clone(),
        project_id: project_beta.id.clone(),
        environment_id: beta_local.id.clone(),
        name: "Invalid Cross Scope".to_string(),
        channel_type: ChannelType::Webhook,
        credential_identifier: None,
        is_enabled: None,
    });
    assert!(matches!(
        mismatched_channel,
        Err(ApplicationError::Validation(_))
    ));

    // Create channels across all 4 ChannelType variants and test credential reveal/rotate/disable.
    for channel_type in ChannelType::ALL {
        let created = services
            .channels
            .create_channel(CreateChannelInput {
                workspace_id: workspace_alpha.id.clone(),
                project_id: project_alpha.id.clone(),
                environment_id: alpha_local.id.clone(),
                name: format!("{channel_type:?} Capture"),
                channel_type,
                credential_identifier: None,
                is_enabled: Some(true),
            })
            .expect("channel must be created");

        let revealed = services
            .channels
            .reveal_channel_credential(&created.channel.id)
            .expect("credential reveal must succeed");
        assert_eq!(revealed.secret_token, created.credential.secret_token);

        let authenticated = services
            .channels
            .authenticate_composite_key(&revealed.composite_key)
            .expect("initial credential must authenticate");
        assert_eq!(authenticated.id, created.channel.id);

        let rotated = services
            .channels
            .rotate_channel_credential(&created.channel.id)
            .expect("credential rotation must succeed");
        assert_ne!(rotated.secret_token, revealed.secret_token);

        // Old key is immediately invalidated; new key works.
        assert!(matches!(
            services
                .channels
                .authenticate_composite_key(&revealed.composite_key),
            Err(ApplicationError::Unauthorized)
        ));
        assert!(
            services
                .channels
                .authenticate_composite_key(&rotated.composite_key)
                .is_ok()
        );

        // Disabled channel rejects authentication cleanly.
        services
            .channels
            .set_channel_enabled(&created.channel.id, false)
            .expect("disable channel");
        assert!(matches!(
            services
                .channels
                .authenticate_composite_key(&rotated.composite_key),
            Err(ApplicationError::Unauthorized)
        ));
        services
            .channels
            .set_channel_enabled(&created.channel.id, true)
            .expect("re-enable channel");
    }

    // Create notification in Alpha and verify Beta inbox stays completely isolated.
    let alpha_channels = services
        .channels
        .list_channels_by_scope(&workspace_alpha.id, &project_alpha.id, &alpha_local.id)
        .expect("list alpha channels");
    assert_eq!(alpha_channels.len(), 4);

    let email_channel = alpha_channels
        .iter()
        .find(|channel| channel.channel_type == ChannelType::Email)
        .expect("email channel exists");

    let detail = services
        .notifications
        .capture_notification(CaptureNotificationInput {
            workspace_id: workspace_alpha.id.clone(),
            project_id: project_alpha.id.clone(),
            environment_id: alpha_local.id.clone(),
            channel_id: email_channel.id.clone(),
            channel_type: ChannelType::Email,
            recipient: "user@example.com".to_string(),
            sender: "noreply@alpha.local".to_string(),
            subject: "100% Verification_Code".to_string(),
            preview_text: "Code 482-910".to_string(),
            payload_json: json!({ "code": "482-910" }),
            raw_payload: "Subject: Verification".to_string(),
            test_run_identifier: Some("suite-alpha-1".to_string()),
            received_at: None,
            attachments: vec![CaptureAttachmentInput {
                filename: "invoice.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                content_id: None,
                is_inline: false,
                content_blob: b"%PDF-1.4 sample".to_vec(),
            }],
        })
        .expect("capture notification with attachment");

    assert_eq!(detail.attachments.len(), 1);
    let stored_blob = services
        .notifications
        .get_attachment(&detail.attachments[0].id)
        .expect("attachment blob readable");
    assert_eq!(stored_blob.content_blob, b"%PDF-1.4 sample");

    // Beta inbox sees 0 items.
    let beta_page = services
        .notifications
        .query_notifications(NotificationQuery {
            workspace_id: workspace_beta.id.clone(),
            project_id: project_beta.id.clone(),
            environment_id: beta_local.id.clone(),
            channel_id: None,
            channel_type: None,
            status: None,
            search_query: None,
            limit: Some(25),
            offset: Some(0),
            ..NotificationQuery::default()
        })
        .expect("query beta inbox");
    assert_eq!(beta_page.total_count, 0);

    // Alpha search with literal '%' and '_' works safely with LIKE escaping.
    let alpha_search = services
        .notifications
        .query_notifications(NotificationQuery {
            workspace_id: workspace_alpha.id.clone(),
            project_id: project_alpha.id.clone(),
            environment_id: alpha_local.id.clone(),
            channel_id: None,
            channel_type: Some(ChannelType::Email),
            status: Some(NotificationStatus::Captured),
            search_query: Some("100% Verification_Code".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..NotificationQuery::default()
        })
        .expect("query alpha inbox");
    assert_eq!(alpha_search.total_count, 1);
    assert_eq!(alpha_search.items[0].id, detail.notification.id);

    // Status update and cascading workspace delete.
    let marked_read = services
        .notifications
        .update_notification_status(&detail.notification.id, NotificationStatus::Read)
        .expect("mark read");
    assert_eq!(marked_read.status, NotificationStatus::Read);

    services
        .workspaces
        .delete_workspace(&workspace_alpha.id)
        .expect("delete alpha workspace");
    assert!(
        services
            .notifications
            .get_notification_detail(&detail.notification.id)
            .is_err()
    );
}

#[test]
fn settings_service_validates_and_persists_server_configuration() {
    let services = create_test_services();
    let default_config = ServerConfiguration::default();

    let saved = services
        .settings
        .save_server_configuration(ServerConfiguration {
            bind_address: "127.0.0.1".to_string(),
            http_port: 5080,
            smtp_port: 2525,
            ..default_config.clone()
        })
        .expect("valid configuration should save");
    assert_eq!(saved.http_port, 5080);

    let reloaded = services
        .settings
        .load_server_configuration_or_default(&default_config)
        .expect("saved configuration should reload");
    assert_eq!(reloaded, saved);

    let conflict = services
        .settings
        .save_server_configuration(ServerConfiguration {
            http_port: 4080,
            smtp_port: 4080,
            ..default_config.clone()
        });
    assert!(matches!(conflict, Err(ApplicationError::Validation(_))));

    let payload_too_small = services
        .settings
        .save_server_configuration(ServerConfiguration {
            maximum_http_payload_bytes: 512,
            ..default_config
        });
    assert!(matches!(
        payload_too_small,
        Err(ApplicationError::Validation(_))
    ));

    assert!(
        services
            .settings
            .set_setting("inbox.retention_days", json!(30))
            .is_ok()
    );
    assert!(matches!(
        services
            .settings
            .set_setting("inbox.retention_days", json!(0)),
        Err(ApplicationError::Validation(_))
    ));
    assert!(matches!(
        services
            .settings
            .set_setting("inbox.retention_days", json!("invalid")),
        Err(ApplicationError::Validation(_))
    ));
    assert!(matches!(
        services.settings.set_setting(
            "workspace.active_selection",
            json!({ "activeWorkspaceId": "not-a-uuid" })
        ),
        Err(ApplicationError::Validation(_))
    ));
    assert!(
        services
            .settings
            .set_setting("app.auto_update_check_enabled", json!(false))
            .is_ok()
    );
    assert!(
        services
            .settings
            .set_setting("app.auto_update_check_enabled", json!(true))
            .is_ok()
    );
    assert!(matches!(
        services
            .settings
            .set_setting("app.auto_update_check_enabled", json!("yes")),
        Err(ApplicationError::Validation(_))
    ));
}

#[test]
fn backend_rejects_control_characters_overlength_queries_and_bad_pagination() {
    assert!(validate_name("Acme\nStudio").is_err());
    assert!(validate_name("استودیو توسعه").is_ok());
    assert!(validate_description("Line 1\nLine 2").is_ok());
    assert!(validate_description("Bad\u{0007}Text").is_err());

    assert_eq!(validate_search_query(Some("   ")).unwrap(), None);
    assert_eq!(
        validate_search_query(Some("  invoice  ")).unwrap(),
        Some("invoice".to_string())
    );
    let too_long = "a".repeat(65);
    assert!(validate_search_query(Some(&too_long)).is_err());
    assert!(validate_search_query(Some("abc\u{0000}")).is_err());

    assert_eq!(validate_pagination(None, None).unwrap(), (50, 0));
    assert!(validate_pagination(Some(0), Some(0)).is_err());
    assert!(validate_pagination(Some(201), Some(0)).is_err());

    assert!(validate_retention_days(1).is_ok());
    assert!(validate_retention_days(3650).is_ok());
    assert!(validate_retention_days(0).is_err());
    assert!(validate_retention_days(3651).is_err());
}

#[test]
fn retention_cleanup_deletes_expired_notifications_and_preserves_recent_records() {
    let services = create_test_services();
    assert_eq!(
        services
            .settings
            .load_retention_days_or_default()
            .expect("default retention days"),
        30
    );

    let workspace = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Retention Workspace".to_string(),
            slug: None,
            description: None,
        })
        .expect("workspace");
    let project = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace.id.clone(),
            name: "Retention Project".to_string(),
            slug: None,
            description: None,
        })
        .expect("project");
    let environment = services
        .environments
        .list_environments_by_project(&project.id)
        .expect("environments")
        .into_iter()
        .next()
        .expect("default environment");
    let channel = services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: workspace.id.clone(),
            project_id: project.id.clone(),
            environment_id: environment.id.clone(),
            name: "Retention Email".to_string(),
            channel_type: ChannelType::Email,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("channel");

    let expired = services
        .notifications
        .capture_notification(CaptureNotificationInput {
            workspace_id: workspace.id.clone(),
            project_id: project.id.clone(),
            environment_id: environment.id.clone(),
            channel_id: channel.channel.id.clone(),
            channel_type: ChannelType::Email,
            recipient: "old@example.com".to_string(),
            sender: "noreply@example.com".to_string(),
            subject: "Expired".to_string(),
            preview_text: "Expired record".to_string(),
            payload_json: json!({ "textBody": "old" }),
            raw_payload: "old".to_string(),
            test_run_identifier: None,
            received_at: Some("2020-01-01T00:00:00Z".to_string()),
            attachments: vec![CaptureAttachmentInput {
                filename: "expired.txt".to_string(),
                content_type: "text/plain".to_string(),
                content_id: None,
                is_inline: false,
                content_blob: b"old-blob".to_vec(),
            }],
        })
        .expect("capture expired notification");

    let recent = services
        .notifications
        .capture_notification(CaptureNotificationInput {
            workspace_id: workspace.id.clone(),
            project_id: project.id.clone(),
            environment_id: environment.id.clone(),
            channel_id: channel.channel.id.clone(),
            channel_type: ChannelType::Email,
            recipient: "fresh@example.com".to_string(),
            sender: "noreply@example.com".to_string(),
            subject: "Fresh".to_string(),
            preview_text: "Recent record".to_string(),
            payload_json: json!({ "textBody": "fresh" }),
            raw_payload: "fresh".to_string(),
            test_run_identifier: None,
            received_at: None,
            attachments: Vec::new(),
        })
        .expect("capture recent notification");

    let pruned_count = services
        .notifications
        .prune_expired_by_retention_days(7)
        .expect("prune expired notifications");
    assert_eq!(pruned_count, 1);

    assert!(matches!(
        services
            .notifications
            .get_notification_detail(&expired.notification.id),
        Err(ApplicationError::NotFound(_))
    ));
    assert!(matches!(
        services
            .notifications
            .get_attachment(&expired.attachments[0].id),
        Err(ApplicationError::NotFound(_))
    ));
    assert!(
        services
            .notifications
            .get_notification_detail(&recent.notification.id)
            .is_ok()
    );
}

#[test]
fn database_storage_statistics_and_vacuum_lifecycle() {
    let services = create_test_services();

    let initial_statistics = services
        .notifications
        .get_database_storage_statistics()
        .expect("must read database storage statistics");

    assert!(initial_statistics.file_size_bytes > 0);
    assert_eq!(initial_statistics.total_notifications_count, 0);

    let vacuumed_statistics = services
        .notifications
        .vacuum_database()
        .expect("database vacuum must succeed");

    assert!(vacuumed_statistics.file_size_bytes > 0);
    assert_eq!(vacuumed_statistics.total_notifications_count, 0);
}

#[test]
fn channel_forwarding_rules_persistence_and_matrix_update() {
    let services = create_test_services();

    let workspace = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Forwarding Workspace".to_string(),
            slug: None,
            description: None,
        })
        .expect("workspace");

    let project = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace.id.clone(),
            name: "Billing Pipeline".to_string(),
            slug: None,
            description: None,
        })
        .expect("project");

    let environments = services
        .environments
        .list_environments_by_project(&project.id)
        .expect("environments");
    let environment = &environments[0];

    let created_channel = services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: workspace.id.clone(),
            project_id: project.id.clone(),
            environment_id: environment.id.clone(),
            name: "Webhook Pipeline Channel".to_string(),
            channel_type: ChannelType::Webhook,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("create channel");

    assert!(!created_channel.channel.forwarding_rule.is_enabled);

    let mut custom_headers = std::collections::BTreeMap::new();
    custom_headers.insert("X-Forward-Service".to_string(), "billing".to_string());

    let updated = services
        .channels
        .update_channel_rules(notifytrap_lib::domain::channel::UpdateChannelRulesInput {
            channel_id: created_channel.channel.id.clone(),
            webhook_mock: notifytrap_lib::domain::channel::WebhookMockConfiguration::default(),
            chaos_rule: notifytrap_lib::domain::channel::ChannelChaosConfiguration::default(),
            forwarding_rule: notifytrap_lib::domain::forwarding::ChannelForwardingConfiguration {
                is_enabled: true,
                target_url: "http://127.0.0.1:9000/api/hook".to_string(),
                custom_headers,
                include_original_headers: true,
                filter_event_name: Some("invoice.paid".to_string()),
                timeout_seconds: 8,
            },
        })
        .expect("update rules");

    assert!(updated.forwarding_rule.is_enabled);
    assert_eq!(
        updated.forwarding_rule.target_url,
        "http://127.0.0.1:9000/api/hook"
    );
    assert_eq!(updated.forwarding_rule.timeout_seconds, 8);
    assert_eq!(
        updated.forwarding_rule.filter_event_name.as_deref(),
        Some("invoice.paid")
    );
}

proptest! {
    #[test]
    fn property_derived_slug_is_always_valid_for_alphanumeric_inputs(
        raw_name in "[A-Za-z0-9][A-Za-z0-9 _-]{1,40}[A-Za-z0-9]"
    ) {
        let slug = derive_slug_or_validate(None, &raw_name);
        prop_assert!(slug.is_ok());
        if let Ok(normalized) = slug {
            prop_assert!(validate_slug(&normalized).is_ok());
        }
    }

    #[test]
    fn property_secret_encryption_and_constant_time_hash_round_trip(
        secret in "[A-Za-z0-9_.-]{8,64}",
        other_secret in "[A-Za-z0-9_.-]{8,64}",
        master_key in any::<[u8; 32]>(),
    ) {
        let encrypted = encrypt_secret(&master_key, &secret).expect("encryption works");
        let decrypted = decrypt_secret(&master_key, &encrypted).expect("decryption works");
        prop_assert_eq!(decrypted, secret.clone());

        let expected_hash = hash_secret_hex(&secret);
        prop_assert!(verify_secret_constant_time(&secret, &expected_hash));
        if other_secret != secret {
            prop_assert!(!verify_secret_constant_time(&other_secret, &expected_hash));
        }
    }
}
