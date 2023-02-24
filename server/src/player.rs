use axum::extract::{FromRef, FromRequestParts, Query};
use axum::http::request::Parts;
use axum::response::Redirect;
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::SignedCookieJar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub const CREATE_SESSION_ENDPOINT: &str = "/create-session";

const SESSION_COOKIE: &str = "SESSION";

pub async fn create_session(
    Query(params): Query<HashMap<String, String>>,
    jar: SignedCookieJar,
) -> (SignedCookieJar, Redirect) {
    let player = Player::new();
    let value = serde_json::to_string(&player).unwrap();
    let cookie = Cookie::build(SESSION_COOKIE, value)
        .path("/")
        .secure(true)
        .permanent()
        .finish();

    (
        jar.add(cookie),
        Redirect::to(params.get("redirect").unwrap_or(&"/".to_owned())),
    )
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Player {
    id: Uuid,
}

impl Player {
    fn new() -> Self {
        Player { id: Uuid::new_v4() }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "player {}", self.id)
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Player
where
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .unwrap();
        let cookie = jar.get(SESSION_COOKIE).ok_or(Redirect::to(&format!(
            "{}?redirect={}",
            CREATE_SESSION_ENDPOINT, parts.uri
        )))?;
        let player = serde_json::from_str(cookie.value()).unwrap();
        Ok(player)
    }
}
