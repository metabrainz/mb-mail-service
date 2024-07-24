use config::Config;
use render::EngineError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod components;
mod render;
mod send;
mod serve;
mod templates;
mod text;

mf1::load_locales!();

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct Settings {
    #[serde(default)]
    listen: serve::ListenerConfig,
    #[serde(default)]
    smtp: serve::SmtpMailerConfig,
}

fn locale_from_optional_code(lang: Option<String>) -> Result<Locale, EngineError> {
    Ok(lang
        .map(|l| {
            <Locale as std::str::FromStr>::from_str(&l)
                .map_err(|_| EngineError::BadLanguageCode(std::borrow::Cow::Owned(l)))
        })
        .transpose()?
        .unwrap_or_default())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = sentry::init(sentry::ClientOptions {
        ..sentry::ClientOptions::default()
    });

    let config = Config::builder()
        .add_source(
            config::Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_")
                .convert_case(convert_case::Case::Snake),
        )
        .build()
        .unwrap();

    let settings: Settings = config.try_deserialize().unwrap();
    dbg!(&settings);

    // let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        // .with(console_layer)
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,html5ever=warn,lettre::transport::smtp::client::async_connection=warn,runtime=warn,tokio::task=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(sentry::integrations::tracing::layer())
        .init();

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(serve::serve(settings.listen, settings.smtp));
    Ok(())
}
