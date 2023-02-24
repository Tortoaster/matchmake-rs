use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::ws::WebSocket;
use axum::extract::{FromRef, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, Server};
use axum_extra::extract::cookie::Key;
use futures_util::{SinkExt, StreamExt};
use tracing::info;

use crate::matchmaker::Matchmaker;

mod matchmaker;

const ENV_HOST: &str = "HOST";
const ENV_PORT: &str = "PORT";

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 9000;

#[derive(Clone)]
pub struct AppState {
    matchmaker: Matchmaker,
    key: Key,
}

impl FromRef<AppState> for Matchmaker {
    fn from_ref(input: &AppState) -> Self {
        input.matchmaker
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(input: &AppState) -> Self {
        input.key.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host = env::var(ENV_HOST)
        .map(|host| host.parse().expect("invalid host"))
        .unwrap_or(DEFAULT_HOST);
    let port = env::var(ENV_PORT)
        .map(|port| port.parse().expect("invalid port"))
        .unwrap_or(DEFAULT_PORT);
    let addr = SocketAddr::new(host, port);

    let state = AppState {
        matchmaker: Matchmaker::new(),
        key: Key::generate(),
    };

    let app = Router::new()
        .route("/connect", get(connect))
        .with_state(state);

    info!("listening on http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn connect(ws: WebSocketUpgrade, State(matchmaker): State<Matchmaker>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, matchmaker))
}

async fn handle_socket(socket: WebSocket, matchmaker: Matchmaker) {
    let (mut send, recv) = socket.split();

    let mut recv = matchmaker.find_match(recv).await;

    while let Some(Ok(message)) = recv.next().await {
        send.send(message).await.unwrap();
    }
}
