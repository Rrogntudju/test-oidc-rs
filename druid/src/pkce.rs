use crate::Fournisseur;
use anyhow::Error;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const ID_MS: &str = include_str!("clientid.microsoft");
const SECRET_MS: &str = include_str!("secret.microsoft");
const ID_GG: &str = include_str!("clientid.google");
const SECRET_GG: &str = include_str!("secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";

impl Fournisseur {
    fn get_endpoints(&self) -> (&str, &str, &str) {
        match &self {
            Self::Microsoft => (AUTH_MS, TOKEN_MS, INFOS_MS),
            Self::Google => (AUTH_GG, TOKEN_GG, INFOS_GG),
        }
    }

    fn get_secrets(&self) -> (&str, &str) {
        match &self {
            Self::Microsoft => (ID_MS, SECRET_MS),
            Self::Google => (ID_GG, SECRET_GG),
        }
    }
}

fn get_authorization_token(f: Fournisseur) -> Result<String, Error> {
    let (id, secret) = f.get_secrets();
    let id = ClientId::new(id.to_owned());
    let secret = ClientSecret::new(secret.to_owned());

    let (url_auth, url_token, url_infos) = f.get_endpoints();
    let url_auth = AuthUrl::new(url_auth.to_owned())?;
    let url_token = TokenUrl::new(url_token.to_owned())?;

    let client = BasicClient::new(id, Some(secret), url_auth, Some(url_token))
        .set_auth_type(AuthType::RequestBody)
        .set_redirect_uri(RedirectUrl::new("http://localhost:6666".to_owned())?);

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
    .authorize_url(CsrfToken::new_random)
    .add_scope(Scope::new("openid".to_owned()))
    .add_scope(Scope::new("profile".to_owned()))
    .set_pkce_challenge(pkce_code_challenge)
    .url();

    let listener = TcpListener::bind("[::1]:6666")?;
    let mut request_line = String::new();

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let reader = BufReader::new(&stream);
            reader.read_line(&mut request_line)?;
            break;
        }
    }

    Ok(String::new())
}
