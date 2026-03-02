use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use microvm_blueprint_lib::{JobRunner, LifecycleJob, MockVmProvider, VmQuery};
use serde::Serialize;
use tokio::{signal, sync::mpsc, time::interval};

#[derive(Clone)]
struct AppState {
    query: MockVmProvider,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = MockVmProvider::default();
    let runner = JobRunner::new(provider.clone());
    let (job_tx, mut job_rx) = mpsc::channel::<LifecycleJob>(64);

    let runner_task = tokio::spawn({
        let runner = runner.clone();
        async move {
            while let Some(job) = job_rx.recv().await {
                if let Err(error) = runner.execute(job.clone()) {
                    eprintln!("lifecycle job failed ({job:?}): {error}");
                }
            }
        }
    });

    enqueue_bootstrap_jobs(&job_tx).await;

    tokio::spawn({
        let query = provider.clone();
        async move {
            let mut ticker = interval(Duration::from_secs(10));
            loop {
                ticker.tick().await;
                match query.list_vms() {
                    Ok(vms) => println!("query-monitor: {} vm(s)", vms.len()),
                    Err(error) => eprintln!("query-monitor failed: {error}"),
                }
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/vms", get(list_vms))
        .route("/vms/{vm_id}", get(get_vm))
        .route("/vms/{vm_id}/snapshots", get(list_snapshots))
        .with_state(AppState {
            query: provider.clone(),
        });

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    println!("query service listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    drop(job_tx);
    let _ = runner_task.await;

    Ok(())
}

async fn enqueue_bootstrap_jobs(job_tx: &mpsc::Sender<LifecycleJob>) {
    let jobs = [
        LifecycleJob::Create {
            vm_id: "demo-vm".to_owned(),
        },
        LifecycleJob::Start {
            vm_id: "demo-vm".to_owned(),
        },
        LifecycleJob::Snapshot {
            vm_id: "demo-vm".to_owned(),
            snapshot_id: "initial".to_owned(),
        },
    ];

    for job in jobs {
        if job_tx.send(job).await.is_err() {
            eprintln!("job queue was closed before bootstrap jobs were enqueued");
            break;
        }
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn list_vms(State(state): State<AppState>) -> Response {
    match state.query.list_vms() {
        Ok(vms) => Json(vms).into_response(),
        Err(error) => internal_error(error.to_string()),
    }
}

async fn get_vm(Path(vm_id): Path<String>, State(state): State<AppState>) -> Response {
    match state.query.get_vm(&vm_id) {
        Ok(Some(vm)) => Json(vm).into_response(),
        Ok(None) => not_found(format!("vm '{vm_id}' not found")),
        Err(error) => internal_error(error.to_string()),
    }
}

async fn list_snapshots(Path(vm_id): Path<String>, State(state): State<AppState>) -> Response {
    match state.query.list_snapshots(&vm_id) {
        Ok(Some(snapshots)) => Json(snapshots).into_response(),
        Ok(None) => not_found(format!("vm '{vm_id}' not found")),
        Err(error) => internal_error(error.to_string()),
    }
}

fn not_found(message: String) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody {
            error: message,
        }),
    )
        .into_response()
}

fn internal_error(message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: message,
        }),
    )
        .into_response()
}

async fn shutdown_signal() {
    if signal::ctrl_c().await.is_ok() {
        println!("shutdown signal received");
    }
}
