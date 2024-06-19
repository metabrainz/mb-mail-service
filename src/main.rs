use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod components;
mod render;
mod send;
mod serve;
mod templates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        // .with(console_layer)
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,html5ever=warn,lettre::transport::smtp::client::async_connection=warn,runtime=warn,tokio::task=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(serve::serve());
    Ok(())
}
