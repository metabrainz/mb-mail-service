use tracing::warn;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpListener, signal};

use crate::{
    render::{render_html_route, render_text_route},
    send::{send_mail_route, MailTransport},
};
use axum::routing::get;

use std::str::FromStr;
use strum_macros::EnumString;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::render::render_html_route, crate::render::render_text_route,
        crate::send::send_mail_route
    ),
    tags(
        (name = "mb-mail-service", description = "MusicBrains Mail Service API")
    )
)]
struct ApiDoc;

pub(crate) async fn serve() {
    // If possible, we want to use a socket passed to the app by, eg, SystemD or ListenFD.
    // Otherwise, we will use [address] to get a socket.
    let mut listenfd = listenfd::ListenFd::from_env();
    let listener = if let Ok(Some(listener)) = listenfd.take_tcp_listener(0) {
        tracing::info!("server listening on socket");
        listener.set_nonblocking(true).unwrap();
        TcpListener::from_std(listener).unwrap()
    } else {
        let addr = address();
        tracing::info!("server listening on {addr}");
        TcpListener::bind(addr).await.unwrap()
    };

    let app = axum::Router::new()
        // OpenAPI docs
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Our routes
        .route("/templates/:template_id/html", get(render_html_route))
        .route("/templates/:template_id/text", get(render_text_route))
        .route("/send/:template_id", get(send_mail_route))
        .with_state(mailer())
        .layer((
            // Logging
            TraceLayer::new_for_http(),
            // Give a universal timeout to prevent
            // DOS and for graceful shutdown
            TimeoutLayer::new(Duration::from_secs(60)),
        ));

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

/// Get the socket address for the server.
/// This will try to get an IP address from the
/// `HOST` environment variable and a port from
/// the `PORT` variable, but will otherwise
/// default to `127.0.0.1:3000`.
fn address() -> SocketAddr {
    let host = std::env::var("HOST")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(3000);

    SocketAddr::from((host, port))
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
enum SmtpConnectionMode {
    Plaintext,
    Startls,
    Tls,
}

/// Get the configuration for the mailer.
/// Options:
///
/// `SMTP_MODE`: `plaintext`, `startls` or `tls`.
/// Defaults to `plaintext`. \
/// `SMTP_HOST`: The SMTP relay to connect to.
/// Defaults to localhost \
/// `SMTP_PORT`: The port on the relay to
/// connect to. Defaults to 25
fn mailer() -> MailTransport {
    let host = std::env::var("SMTP_HOST").ok();
    let port = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(25);
    let mode = std::env::var("SMTP_MODE").ok().and_then(|v| {
        SmtpConnectionMode::from_str(&v)
            .inspect_err(|_| eprintln!("Unrecognised connection mode: {v}"))
            .ok()
    });

    match mode {
        Some(SmtpConnectionMode::Plaintext) => {
            MailTransport::builder_dangerous(host.as_deref().unwrap_or("localhost"))
        }
        Some(SmtpConnectionMode::Startls) => {
            MailTransport::starttls_relay(host.as_deref().unwrap_or("localhost")).unwrap()
        }
        Some(SmtpConnectionMode::Tls) => {
            MailTransport::relay(host.as_deref().unwrap_or("localhost")).unwrap()
        }
        None => {
            if host.is_some() {
                warn!("SMTP connection mode not specified, defaulting to unsafe plain text!")
            }
            MailTransport::builder_dangerous(host.as_deref().unwrap_or("localhost"))
        }
    }
    .timeout(Some(Duration::from_secs(10)))
    .port(port)
    .build()
}

/// This future resolves when either
/// Ctrl+C or SIGTERM is recieved. It is
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
