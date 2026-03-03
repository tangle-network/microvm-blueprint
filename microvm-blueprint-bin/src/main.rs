//! MicroVM Blueprint Runner
//!
//! Main entry point wiring the lifecycle jobs and query service onto the
//! Tangle EVM producer/consumer via BlueprintRunner.

use std::net::SocketAddr;
use std::sync::Arc;

use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::{TangleConsumer, TangleProducer};
use blueprint_sdk::{error, info};

use microvm_blueprint_lib::{
    JOB_CREATE, JOB_DESTROY, JOB_SNAPSHOT, JOB_START, JOB_STOP, QueryService, VmRuntime,
    init_provider, router,
};
#[cfg(feature = "firecracker")]
use microvm_runtime::FirecrackerVmProvider;
use microvm_runtime::InMemoryVmProvider;

/// Initialize tracing from RUST_LOG env var.
fn setup_log() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::from_default_env();
    fmt().with_env_filter(filter).init();
}

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), blueprint_sdk::Error> {
    setup_log();

    // Initialize VM provider from env. Defaults to in-memory for local/dev.
    let provider_kind = std::env::var("MICROVM_PROVIDER")
        .unwrap_or_else(|_| "in-memory".to_string())
        .to_lowercase();
    let provider: Arc<dyn VmRuntime> = match provider_kind.as_str() {
        "in-memory" | "memory" => Arc::new(InMemoryVmProvider::default()),
        "firecracker" => {
            #[cfg(feature = "firecracker")]
            {
                Arc::new(FirecrackerVmProvider::from_env())
            }
            #[cfg(not(feature = "firecracker"))]
            {
                return Err(blueprint_sdk::Error::Other(
                    "MICROVM_PROVIDER=firecracker requires --features firecracker".to_string(),
                ));
            }
        }
        other => {
            return Err(blueprint_sdk::Error::Other(format!(
                "Unknown MICROVM_PROVIDER '{other}' (expected in-memory or firecracker)"
            )));
        }
    };
    init_provider(provider);

    info!(provider = %provider_kind, "Starting microvm-blueprint");

    let env = BlueprintEnvironment::load()?;

    let tangle_client = env
        .tangle_client()
        .await
        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;

    let service_id = env
        .protocol_settings
        .tangle()
        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?
        .service_id
        .ok_or_else(|| blueprint_sdk::Error::Other("No service ID configured".to_string()))?;

    let tangle_producer = TangleProducer::new(tangle_client.clone(), service_id);
    let tangle_consumer = TangleConsumer::new(tangle_client);

    let tangle_config = TangleConfig::default();

    info!("Connected to Tangle. Service ID: {service_id}");
    info!("Registered lifecycle jobs:");
    info!("  Job {JOB_CREATE}: create_vm");
    info!("  Job {JOB_START}: start_vm");
    info!("  Job {JOB_STOP}: stop_vm");
    info!("  Job {JOB_SNAPSHOT}: snapshot_vm");
    info!("  Job {JOB_DESTROY}: destroy_vm");

    let query_addr: SocketAddr = "127.0.0.1:3000".parse().expect("valid address");

    let result = BlueprintRunner::builder(tangle_config, env)
        .router(router())
        .background_service(QueryService::new(query_addr))
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .with_shutdown_handler(async {
            info!("Shutting down microvm-blueprint");
        })
        .run()
        .await;

    if let Err(e) = result {
        error!("Runner failed: {e:?}");
    }

    Ok(())
}
