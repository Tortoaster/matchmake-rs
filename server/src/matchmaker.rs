use axum::extract::ws::WebSocket;
use futures_util::stream::SplitStream;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

lazy_static! {
    static ref WAITING: Mutex<Option<SplitStream<WebSocket>>> = Mutex::new(None);
}

#[derive(Copy, Clone, Debug)]
pub struct Matchmaker {
    waiting: &'static Mutex<Option<SplitStream<WebSocket>>>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Matchmaker { waiting: &WAITING }
    }

    pub async fn find_match(&self, recv: SplitStream<WebSocket>) -> SplitStream<WebSocket> {
        let mut lock = self.waiting.lock().await;
        if lock.is_some() {
            lock.take().unwrap()
        } else {
            *lock = Some(recv);
            drop(lock);
            todo!()
        }
    }
}
