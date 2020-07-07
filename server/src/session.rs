use chrono::{DateTime, Duration, Utc};
use oidc::{token::Token, Client};
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn random_token(len: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(len).collect::<String>()
}

#[derive(PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        SessionId(random_token(32))
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        SessionId(s)
    }
}

pub enum Session {
    AuthenticationRequested(Option<Client>, String),
    Authenticated(Client, Token, DateTime<Utc>),
}

impl Session {
    pub fn new(c: Option<Client>, nonce: String) -> Self {
        Session::AuthenticationRequested(Some(c), nonce)
    }

    pub fn authentication_completed(&mut self, c: Client, t: Token) -> Self {
        assert!(!self.is_authenticated());
        Session::Authenticated(c, t, Utc::now() + Duration::days(1))
    }

    pub fn is_authenticated(&self) -> bool {
        match self {
            Session::Authenticated(..) => true,
            _ => false,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self {
            Session::Authenticated(.., expires) if expires < &Utc::now() => true,
            _ => false,
        }
    }
}

/* pub struct Session {
    pub state: SessionState,
    pub nonce: String,
}

impl Session {
    pub fn new(client: Client) -> Self {
        Session {
            state: SessionState::new(client),
            nonce: random_token(64),
        }
    }
} */
