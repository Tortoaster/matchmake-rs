use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::info;

const ENV_HOST: &str = "HOST";
const ENV_PORT: &str = "PORT";
const ENV_SERVER_URL: &str = "MATCHMAKER_URL";

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 9000;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let server_url = env::var(ENV_SERVER_URL).unwrap_or_else(|_| {
        let host = env::var(ENV_HOST)
            .map(|host| host.parse().expect("invalid host"))
            .unwrap_or(DEFAULT_HOST);
        let port = env::var(ENV_PORT)
            .map(|port| port.parse().expect("invalid port"))
            .unwrap_or(DEFAULT_PORT);
        let addr = SocketAddr::new(host, port);
        format!("ws://{addr}/connect")
    });

    let (ws_stream, _) = tokio_tungstenite::connect_async(server_url)
        .await
        .expect("failed to connect");
    info!("connected to server");

    let (send, recv) = ws_stream.split();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let stdin_to_ws = stdin_rx.map(Ok).forward(send);
    let ws_to_stdout = {
        recv.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
