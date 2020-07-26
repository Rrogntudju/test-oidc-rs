use inth_oauth2::token::{Lifetime, Token as OauthToken};
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
    Authenticated(Client, Token),
}

impl Session {
    pub fn new(c: Client, nonce: String) -> Self {
        Session::AuthenticationRequested(Some(c), nonce)
    }

    pub fn authentication_completed(&mut self, c: Client, t: Token) -> () {
        *self = Session::Authenticated(c, t);
    }

    pub fn is_authenticated(&self) -> bool {
        match self {
            Session::Authenticated(..) => true,
            _ => false,
        }
    }

    pub fn is_expired(&self) -> Option<bool> {
        if let Session::Authenticated(.., token) = self {
            Some(token.lifetime().expired())
        } else {
            None
        }
    }
}
