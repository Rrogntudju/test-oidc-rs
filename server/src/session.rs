use oidc::{token::Token, Client};
use rand::distributions::Alphanumeric;
use rand::Rng;
use chrono::{Utc, Duration, DateTime};

pub fn random_token(len: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(len).collect::<String>()
}

#[derive(PartialEq, Eq, Hash)]
pub struct SessionId(String);

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

pub enum SessionState {
    AuthenticationRequested(Client),
    Authenticated(Client, Token, DateTime<Utc>),
}

impl SessionState {
    pub fn new(c: Client) -> Self {
        SessionState::AuthenticationRequested(c)
    }

    pub fn authentication_completed(&mut self, c: Client, t: Token) -> Self {
        assert!(!self.is_authenticated());
        SessionState::Authenticated(c,  t, Utc::now() + Duration::days(1))
    }
    
    pub fn is_authenticated(&self) -> bool {
        match self {
            SessionState::Authenticated(..) => true,
            _ => false,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self {
            SessionState::Authenticated(.., expires) if expires < &Utc::now() => true,
            _ => false,
        }
    }

}

pub struct Session {
    pub state: SessionState,
    nonce: String,
}

impl Session {
    pub fn new(client: Client) -> Self {
        Session {
            state: SessionState::new(client),
            nonce: random_token(64),
        }
    }
}
