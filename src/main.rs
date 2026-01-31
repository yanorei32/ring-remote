use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{any, get, post},
};
use clap::Parser;
use humantime::Duration as HDuration;
use humantime_serde::Serde as HTSerde;
use tokio::net::{TcpListener, UdpSocket};

#[derive(Clone)]
struct AppState {
    tx: tokio::sync::mpsc::Sender<Duration>,
}

#[derive(Parser, Debug)]
struct Cli {
    #[arg(env, long)]
    listen: SocketAddr,

    #[arg(env, long)]
    target: SocketAddr,

    #[arg(env, long, default_value = "20ms")]
    minimum_ring_request_length: HDuration,

    #[arg(env, long, default_value = "1s")]
    maximum_ring_request_length: HDuration,
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        match msg {
            Message::Text(length) => {
                if let Ok(length) = length.parse::<HDuration>() {
                    let _ = state.tx.send(*length).await;
                }
            }
            _ => {}
        }
    }
}

#[axum::debug_handler]
async fn ring_handler(
    Path(length): Path<HTSerde<Duration>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = state.tx.send(*length).await;
    (StatusCode::OK, "ok")
}

async fn root_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_bytes!("../assets/index.html"),
    )
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Duration>(16);

    tokio::spawn(async move {
        let sock = match &cli.target {
            SocketAddr::V4(_) => UdpSocket::bind("0.0.0.0:0").await,
            SocketAddr::V6(_) => UdpSocket::bind("[::]:0").await,
        }
        .unwrap();

        loop {
            let stop_after = rx.recv().await.unwrap();

            // Cap length
            let stop_after = stop_after
                .min(*cli.maximum_ring_request_length)
                .max(*cli.minimum_ring_request_length);

            let mut stop_at = tokio::time::Instant::now() + stop_after;

            // Start Ringing
            let _ = sock.send_to(&[1], &cli.target).await;

            loop {
                tokio::select! {
                    _ = tokio::time::sleep_until(stop_at) => break,

                    stop_after = rx.recv() => {
                        let stop_after = stop_after.unwrap();

                        // Cap length
                        let stop_after = stop_after
                            .min(*cli.maximum_ring_request_length)
                            .max(*cli.minimum_ring_request_length);

                        // Extends ring time
                        stop_at = stop_at.max(tokio::time::Instant::now() + stop_after);
                    }
                };
            }

            // Stop Ringing
            let _ = sock.send_to(&[0], &cli.target).await;
        }
    });

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/api/ring/{length}", post(ring_handler))
        .route("/api/ws", any(ws_handler))
        .with_state(AppState { tx });

    let listener = TcpListener::bind(cli.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
