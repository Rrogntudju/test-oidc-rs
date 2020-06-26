use oidc::{token::Token, Client};
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn random_token(len: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(len).collect::<String>()
}

pub type SessionId = String;

pub enum SessionState {
    AuthenticationRequested(Client),
    Authenticated(Token),
}

impl SessionState {
    fn new(c: Client) -> Self {
        SessionState::AuthenticationRequested(c)
    }

    fn AuthenticationCompleted(&mut self, t: Token) -> Self {
        assert!(match self {
            SessionState::AuthenticationRequested(..) => true,
            _ => false,
        });
        SessionState::Authenticated(t)
    }
}

pub struct Session {
    state: SessionState,
    nonce: String,
    maxAge: String,
}

impl Session {
    fn new(client: Client, maxAge: String) -> Self {
        Session {
            state: SessionState::new(client),
            nonce: random_token(32),
            maxAge,
        }
    }
}
