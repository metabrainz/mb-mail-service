use config::Config;
use git_testament::{git_testament, render_testament};
use render::EngineError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod components;
mod render;
mod send;
mod serve;
mod templates;

mf1::load_locales!();

git_testament!(TESTAMENT);

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub(crate) struct Settings {
    #[serde(default)]
    pub listen: serve::ListenerConfig,
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
    let testament = render_testament!(TESTAMENT);
    eprintln!("{}", testament);

    let mut build = match TESTAMENT.commit {
        git_testament::CommitKind::NoRepository(version, date)
        | git_testament::CommitKind::NoCommit(version, date) => format!("{}", date),
        git_testament::CommitKind::NoTags(hash, date) => (&hash[..9]).to_string(),
        git_testament::CommitKind::FromTag(tag, hash, date, distance) => (&hash[..9]).to_string(),
    };

    if !TESTAMENT.modifications.is_empty() {
        build.push_str("-dirty-");
        build.push_str(&TESTAMENT.modifications.len().to_string());
    }

    let sentry_release = format!(
        "{}@{}+{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        build
    )
    .leak();

    let _guard = sentry::init(sentry::ClientOptions {
        release: Some(std::borrow::Cow::Borrowed(sentry_release)),
        session_mode: sentry::SessionMode::Request,
        ..sentry::ClientOptions::default()
    });

    let config = Config::builder()
        .add_source(
            config::Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_")
                .convert_case(config::Case::Snake),
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

    let mut args = std::env::args().skip(1);
    if args.next() == Some("healthcheck".to_owned()) {
        rt.block_on(async {
            let endpoint = args.next().unwrap_or(
                "http://".to_owned()
                    + &match settings.listen {
                        serve::ListenerConfig::FileDescriptor { mode: _ } => {
                            "localhost:3000".to_owned()
                        }
                        serve::ListenerConfig::TcpListener {
                            mode: _,
                            port,
                            host,
                        } => host.to_string() + ":" + &port.to_string(),
                        serve::ListenerConfig::AutomaticSelection { port, host } => {
                            host.to_string() + ":" + &port.to_string()
                        }
                    }
                    + "/healthcheck",
            );
            let res = reqwest::get(&endpoint).await;

            if res.is_err() {
                panic!("Can't reach route {endpoint} {res:?}");
            };
        });
        return Ok(());
    };

    rt.block_on(serve::serve(settings.listen, settings.smtp));
    Ok(())
}
