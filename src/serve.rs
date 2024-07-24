use monostate::MustBe;
use tower::ServiceBuilder;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpListener, signal};

use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};

use crate::{
    render::{
        render_html_route_get, render_html_route_post, render_text_route_get,
        render_text_route_post,
    },
    send::{send_mail_bulk_route, send_mail_route, MailTransport},
};
use axum::{
    response::Redirect,
    routing::{get, post},
    Json,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        available_locales,
        crate::render::render_html_route_get,
        crate::render::render_html_route_post,crate::render::render_text_route_get,crate::render::render_text_route_post,
        crate::send::send_mail_route,
        crate::send::send_mail_bulk_route,
        healthcheck
    ),
    components(schemas(crate::send::SendItem, crate::send::SendResponse)),
    tags(
        (name = "mb-mail-service", description = "MusicBrains Mail Service API")
    )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/available_locales",
    responses(
        (status = 200, description = "All available locales", body = [String], example = json!(["en", "es"])),
    )
)]
pub async fn available_locales() -> Json<Vec<&'static str>> {
    Json(crate::Locale::VALUES.iter().map(|l| l.as_str()).collect())
}

#[utoipa::path(
    get,
    path = "/healthcheck",
    responses(
        (status = 200, body = String, example = json!("ok")),
    )
)]
pub async fn healthcheck() -> &'static str {
    "ok"
}
/// How the server should listen for requests
///
/// By default the server will use the `AutomaticSelection`
/// mode, which will use a passed file descriptor if available,
/// but otherwise will listen on the TCP port configured
/// (by default 127.0.0.1:3000)
///
/// | Setting name | Value                                                     | Default value                                                                             |
/// | ------------ | --------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
/// | mode         | `FileDescriptor` \| `AutomaticSelection` \| `TcpListener` | `AutomaticSelection`                                                                      |
/// | port         | unsigned integer                                          | `FileDescriptor`: Ignored<br>`AutomaticSelection`: `3000`<br>`TcpListener`: required      |
/// | host         | IP address                                                | `FileDescriptor`: Ignored<br>`AutomaticSelection`: `127.0.0.1`<br>`TcpListener`: required |

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "mode")]
#[serde(rename_all = "snake_case")]
pub(crate) enum ListenerConfig {
    FileDescriptor,
    AutomaticSelection { port: u16, host: IpAddr },
    TcpListener { port: u16, host: IpAddr },
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self::AutomaticSelection {
            port: 3000,
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }
}

pub(crate) async fn serve(config: ListenerConfig, mailer_config: SmtpMailerConfig) {
    // If possible, we want to use a socket passed to the app by, eg, SystemD or ListenFD.
    // Otherwise, we will use [address] to get a socket.
    let mut listenfd = listenfd::ListenFd::from_env();
    let listener = match config {
        ListenerConfig::FileDescriptor => {
            let listener = listenfd.take_tcp_listener(0).unwrap().unwrap();
            tracing::info!("server listening on socket");
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        ListenerConfig::AutomaticSelection { port, host } => {
            if let Ok(Some(listener)) = listenfd.take_tcp_listener(0) {
                tracing::info!("server listening on socket");
                listener.set_nonblocking(true).unwrap();
                TcpListener::from_std(listener).unwrap()
            } else {
                let addr = SocketAddr::from((host, port));
                tracing::info!("server listening on {addr}");
                TcpListener::bind(addr).await.unwrap()
            }
        }
        ListenerConfig::TcpListener { port, host } => {
            let addr = SocketAddr::from((host, port));
            tracing::info!("server listening on {addr}");
            TcpListener::bind(addr).await.unwrap()
        }
    };

    let mailer = mailer(mailer_config);

    let layer = ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    let app = axum::Router::new()
        .route("/", get(|| async { Redirect::temporary("/swagger-ui") }))
        // OpenAPI docs
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Our routes
        .route("/available_locales", get(available_locales))
        .route("/templates/:template_id/html", post(render_html_route_post))
        .route("/templates/:template_id/html", get(render_html_route_get))
        .route("/templates/:template_id/text", get(render_text_route_get))
        .route("/templates/:template_id/text", post(render_text_route_post))
        .route("/send_single", post(send_mail_route))
        .route("/send_bulk", post(send_mail_bulk_route))
        .with_state(mailer)
        .layer(layer)
        .layer((
            // Logging
            TraceLayer::new_for_http(),
            // Give a universal timeout to prevent
            // DOS and for graceful shutdown
            TimeoutLayer::new(Duration::from_secs(60)),
        ))
        // Place the healthcheck last to bypass previously set layers
        .route("/healthcheck", get(healthcheck));

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn default_timeout() -> Option<Duration> {
    Some(Duration::from_secs(5))
}
fn default_host() -> Cow<'static, str> {
    Cow::Borrowed("localhost")
}
fn default_port() -> u16 {
    25
}

/// | Setting name | Value                             | Default value |
/// | ------------ | --------------------------------- | ------------- |
/// | mode         | `Plaintext` \| `Startls` \| `Tls` | `Plaintext`   |
/// | port         | unsigned integer                  | `25`          |
/// | host         | hostname                          | `localhost`   |
/// | timeout      | duration                          | 5 seconds     |

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SmtpMailerConfig {
    Startls {
        #[allow(dead_code)]
        mode: MustBe!("starttls"),
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default = "default_host")]
        host: Cow<'static, str>,
        #[serde(default = "default_timeout")]
        timeout: Option<Duration>,
    },
    Tls {
        #[allow(dead_code)]
        mode: MustBe!("tls"),
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default = "default_host")]
        host: Cow<'static, str>,
        #[serde(default = "default_timeout")]
        timeout: Option<Duration>,
    },
    Plaintext {
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default = "default_host")]
        host: Cow<'static, str>,
        #[serde(default = "default_timeout")]
        timeout: Option<Duration>,
    },
}

impl Default for SmtpMailerConfig {
    fn default() -> Self {
        Self::Plaintext {
            port: 25,
            host: Cow::Borrowed("localhost"),
            timeout: default_timeout(),
        }
    }
}

pub(crate) fn mailer(config: SmtpMailerConfig) -> MailTransport {
    match config {
        SmtpMailerConfig::Plaintext {
            port,
            host,
            timeout,
        } => MailTransport::builder_dangerous(host)
            .port(port)
            .timeout(timeout)
            .build(),
        SmtpMailerConfig::Startls {
            port,
            host,
            timeout,
            mode: _,
        } => MailTransport::starttls_relay(&host)
            .unwrap()
            .port(port)
            .timeout(timeout)
            .build(),
        SmtpMailerConfig::Tls {
            port,
            host,
            timeout,
            mode: _,
        } => MailTransport::relay(&host)
            .unwrap()
            .port(port)
            .timeout(timeout)
            .build(),
    }
}

/// This future resolves when either
/// Ctrl+C or SIGTERM is received. It is
/// intended for Axum's `with_graceful_shutdown`
/// function.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
