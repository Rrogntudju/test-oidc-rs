use oidc::{token::Token, Client};
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn random_token(len: usize) -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(len).collect::<String>()
}

pub struct SessionId(String);

impl SessionId {
    fn new() -> Self {
        SessionId(random_token(32))
    }
}

pub enum SessionState {
    AuthenticationRequested(Client),
    Authenticated(Token),
}

impl SessionState {
    fn new(c: Client) -> Self {
        SessionState::AuthenticationRequested(c)
    }

    fn AuthenticationCompleted(&mut self, t: Token) -> Self {
        assert!(!self.isAuthenticated());
        SessionState::Authenticated(t)
    }

    fn isAuthenticated(&self) -> bool {
        match self {
            SessionState::AuthenticationRequested(_) => true,
            _ => false 
        }
    }
}

pub struct Session {
    state: SessionState,
    nonce: String,
}

impl Session {
    fn new(client: Client, max_age: String) -> Self {
        Session {
            state: SessionState::new(client),
            nonce: random_token(64),
        }
    }
}
