use axum::extract::ws::WebSocket;
use futures_util::stream::SplitStream;
use lazy_static::lazy_static;
use std::mem;
use std::ops::DerefMut;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

lazy_static! {
    static ref STATE: Mutex<MatchState> = Mutex::new(MatchState::Waiting);
}

#[derive(Copy, Clone, Debug)]
pub struct Matchmaker {
    state: &'static Mutex<MatchState>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Matchmaker { state: &STATE }
    }

    pub async fn find_match(&self, recv: SplitStream<WebSocket>) -> SplitStream<WebSocket> {
        let mut lock = self.state.lock().await;
        match *lock {
            MatchState::Waiting => {
                *lock = MatchState::P1Joined(recv);
                drop(lock);

                loop {
                    time::sleep(Duration::from_secs(1)).await;
                    let mut lock = self.state.lock().await;
                    if let MatchState::P2Joined(_) = *lock {
                        match mem::replace(lock.deref_mut(), MatchState::Waiting) {
                            MatchState::P2Joined(recv) => break recv,
                            _ => unreachable!(),
                        }
                    }
                }
            }
            MatchState::P1Joined(_) => {
                match mem::replace(lock.deref_mut(), MatchState::P2Joined(recv)) {
                    MatchState::P1Joined(recv) => recv,
                    _ => unreachable!(),
                }
            }
            MatchState::P2Joined(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Default)]
enum MatchState {
    #[default]
    Waiting,
    P1Joined(SplitStream<WebSocket>),
    P2Joined(SplitStream<WebSocket>),
}
