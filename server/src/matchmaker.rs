use axum::extract::ws::WebSocket;
use futures_util::stream::SplitStream;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::Player;

lazy_static! {
    static ref WAITING: Mutex<Option<(Player, SplitStream<WebSocket>)>> = Mutex::new(None);
}

#[derive(Copy, Clone, Debug)]
pub struct Matchmaker {
    waiting: &'static Mutex<Option<(Player, SplitStream<WebSocket>)>>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Matchmaker { waiting: &WAITING }
    }

    pub async fn find_match(
        &self,
        player: Player,
        recv: SplitStream<WebSocket>,
    ) -> (Player, SplitStream<WebSocket>) {
        let mut lock = self.waiting.lock().await;
        if lock.is_some() {
            lock.take().unwrap()
        } else {
            *lock = Some((player, recv));
            drop(lock);
            todo!()
        }
    }
}
