use inth_oauth2::token::{Lifetime, Token as OauthToken};
use oidc::{token::Token, Client};
use rand::distributions::Alphanumeric;
use rand::Rng;

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

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

type Fournisseur = String;
type Nonce = String;

pub enum Session {
    AuthenticationRequested(Client, Fournisseur, Nonce),
    Authenticated(Client, Fournisseur, Token),
}

impl Session {
    pub fn new(c: Client, f: Fournisseur, n: Nonce) -> Self {
        Session::AuthenticationRequested(c, f, n)
    }

    pub fn authentication_completed(self, t: Token) -> Self {
        match self {
            Session::AuthenticationRequested(c, f, _) => Session::Authenticated(c, f, t),
            _ => self,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Session::Authenticated(.., token) = self {
            token.lifetime().expired()
        } else {
            true
        }
    }
}
