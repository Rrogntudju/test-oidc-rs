use std::time::{Duration, Instant};
use oauth2::basic::BasicClient;
use rand::distributions::Alphanumeric;
use rand::Rng;
use oauth2::AccessToken;

const ID_MS: &str = include_str!("clientid.microsoft");
const SECRET_MS: &str = include_str!("secret.microsoft");
const ID_GG: &str = include_str!("clientid.google");
const SECRET_GG: &str = include_str!("secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";

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

pub enum Session {
    AuthenticationRequested(Fournisseur, Box<BasicClient>),
    Authenticated(Fournisseur, Token),
}

impl Session {
    pub fn new(f: Fournisseur, c: BasicClient) -> Self {
        Session::AuthenticationRequested(f, Box::new(c))
    }

    pub fn authentication_completed(self, t: Token) -> Self {
        match self {
            Session::AuthenticationRequested(f, _) => Session::Authenticated(f, t),
            _ => self,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Session::Authenticated(.., token) = self {
            token.is_expired()
        } else {
            true
        }
    }
}

pub struct Token {
    token: AccessToken,
    creation: Instant,
    expired_in: Duration,
}

impl Token {
    pub fn new(token: AccessToken, expired_in: Duration) -> Self {
        let creation = Instant::now();
        Self { token, creation, expired_in }
    }

    pub fn is_expired(&self) -> bool {
        self.creation.elapsed() >= self.expired_in
    }

    pub fn secret(&self) -> &String {
        self.token.secret()
    }
}

#[derive(PartialEq, Eq)]
pub enum Fournisseur {
    Microsoft,
    Google,
}

impl From<&str> for Fournisseur {
    fn from(value: &str) -> Self {
        match value {
            "Microsoft" => Fournisseur::Microsoft,
            "Google" => Fournisseur::Google,
            _ => Fournisseur::Microsoft,
        }
    }
}

impl Fournisseur {
    pub fn endpoints(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (AUTH_MS, TOKEN_MS),
            Self::Google => (AUTH_GG, TOKEN_GG),
        }
    }

    pub fn secrets(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (ID_MS, SECRET_MS),
            Self::Google => (ID_GG, SECRET_GG),
        }
    }

    pub fn userinfos(&self) -> &str {
        match self {
            Self::Microsoft => INFOS_MS,
            Self::Google => INFOS_GG,
        }
    }
}