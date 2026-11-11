use std::sync::{Arc, RwLock};

use crate::application::capture_servers::CaptureServerCoordinator;
use crate::configuration::application_configuration::ApplicationConfiguration;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::events::{BroadcastEventPublisher, EventPublisher};
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::security::secret_key_provider::SecretKeyProvider;
use crate::services::channel_service::ChannelService;
use crate::services::environment_service::EnvironmentService;
use crate::services::notification_service::NotificationService;
use crate::services::project_service::ProjectService;
use crate::services::settings_service::SettingsService;
use crate::services::workspace_service::WorkspaceService;

/// Assembled domain service registry shared by desktop IPC commands and background servers.
#[derive(Clone)]
pub struct ApplicationServices {
    pub workspaces: WorkspaceService,
    pub projects: ProjectService,
    pub environments: EnvironmentService,
    pub channels: ChannelService,
    pub notifications: NotificationService,
    pub settings: SettingsService,
}

impl ApplicationServices {
    pub fn new(
        database_manager: DatabaseConnectionManager,
        secret_key_provider: Arc<dyn SecretKeyProvider>,
    ) -> Self {
        Self {
            workspaces: WorkspaceService::new(database_manager.clone()),
            projects: ProjectService::new(database_manager.clone()),
            environments: EnvironmentService::new(database_manager.clone()),
            channels: ChannelService::new(database_manager.clone(), secret_key_provider),
            notifications: NotificationService::new(database_manager.clone()),
            settings: SettingsService::new(database_manager),
        }
    }
}

/// Shared application state managed by Tauri and injected into thin command boundaries.
#[derive(Clone)]
pub struct ApplicationState {
    pub application_name: String,
    pub application_version: String,
    pub server_configuration: Arc<RwLock<ServerConfiguration>>,
    pub database_manager: DatabaseConnectionManager,
    pub services: ApplicationServices,
    pub capture_servers: CaptureServerCoordinator,
    pub event_publisher: Arc<dyn EventPublisher>,
}

impl ApplicationState {
    pub fn new(
        configuration: ApplicationConfiguration,
        database_manager: DatabaseConnectionManager,
        secret_key_provider: Arc<dyn SecretKeyProvider>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        let services = ApplicationServices::new(database_manager.clone(), secret_key_provider);
        let capture_servers = CaptureServerCoordinator::new(&configuration.server);
        let publisher =
            event_publisher.unwrap_or_else(|| Arc::new(BroadcastEventPublisher::default()));

        Self {
            application_name: configuration.application_name,
            application_version: configuration.application_version,
            server_configuration: Arc::new(RwLock::new(configuration.server)),
            database_manager,
            services,
            capture_servers,
            event_publisher: publisher,
        }
    }

    pub fn current_server_configuration(&self) -> ServerConfiguration {
        self.server_configuration
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }
}
