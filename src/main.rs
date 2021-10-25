#![feature(process_exitcode_placeholder)]

mod service;

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = try_setup_tracing() {
        eprintln!("Failed to setup tracing: {}", e);
    }

    tracing::info!("Welcome to proxie {}!", env!("CARGO_PKG_VERSION"));

    if let Err(e) = serve().await {
        tracing::error!("{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn try_setup_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter({
            match EnvFilter::try_from_default_env() {
                Ok(filter) => {
                    println!(
                        "Using tracing filter from ${}: \"{}\"",
                        EnvFilter::DEFAULT_ENV,
                        std::env::var(EnvFilter::DEFAULT_ENV).unwrap(),
                    );
                    filter
                }
                Err(e) => {
                    eprintln!("Failed to parse ${}: {}", EnvFilter::DEFAULT_ENV, e);
                    println!("Using default tracing filter");
                    EnvFilter::default()
                }
            }
        })
        .with_thread_names(false)
        .try_init()
}

async fn serve() -> Result<(), hyper::Error> {
    let addr = ([0, 0, 0, 0], 80).into();
    tracing::info!("Binding to {}", addr);

    Server::bind(&addr)
        .serve(make_service_fn(|sock: &AddrStream| {
            tracing::trace!("Incoming request from {}", sock.remote_addr());
            async move { Ok::<_, http::Error>(service_fn(service::respond)) }
        }))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install shutdown signal handler")
        })
        .await
}
