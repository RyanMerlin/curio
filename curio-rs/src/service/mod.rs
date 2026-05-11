pub mod auth;
pub mod providers;
pub mod registry;
pub mod runtime;
pub mod server;
pub mod types;

pub use auth::AuthState;
pub use providers::{
    GeminiProviderAdapter, OpenAIProviderAdapter, PassthroughProviderAdapter,
    build_provider_adapter, provider_backend_from_env,
};
pub use registry::{
    WorkspaceRegistry, WorkspaceRegistryError, default_registry_path, discover_repo_root,
};
pub use runtime::{
    CommandCurioExecutor, GitMaterializer, JobStore, NoopCurioExecutor, NoopMaterializer,
    ServiceConfig, ServiceRuntime,
};
pub use server::serve;
pub use types::*;
