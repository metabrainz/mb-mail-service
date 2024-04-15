use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    time::Duration,
};

use axum::{response::IntoResponse, routing::get, Json};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod render;
mod serve;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(serve::serve());
    Ok(())
}
