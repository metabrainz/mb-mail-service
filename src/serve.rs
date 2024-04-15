use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(crate::render::render_html, crate::render::render_text))]

struct ApiDoc;

use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{net::TcpListener, signal};

use crate::render::{render_html, render_text};
use axum::routing::get;

pub(crate) async fn serve() {
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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/templates/:template_id/html", get(render_html))
        .route("/templates/:template_id/text", get(render_text))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(60)),
        ));

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

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
