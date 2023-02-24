use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{FromRef, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, Server};
use axum_extra::extract::cookie::Key;
use futures_util::{SinkExt, StreamExt};
use tracing::info;

use crate::matchmaker::Matchmaker;
use crate::player::Player;

mod matchmaker;
mod player;

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
        .route(player::CREATE_SESSION_ENDPOINT, get(player::create_session))
        .with_state(state);

    info!("listening on http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn connect(
    ws: WebSocketUpgrade,
    player: Player,
    State(matchmaker): State<Matchmaker>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(player, socket, matchmaker))
}

async fn handle_socket(player: Player, socket: WebSocket, matchmaker: Matchmaker) {
    let (mut send, recv) = socket.split();

    let (player, mut recv) = matchmaker.find_match(player, recv).await;

    send.send(Message::Text(format!("connected with {}", player)))
        .await
        .unwrap();

    while let Some(Ok(message)) = recv.next().await {
        send.send(message).await.unwrap();
    }
}
