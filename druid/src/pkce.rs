use crate::Fournisseur;
use anyhow::{anyhow, Error};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
pub use oauth2::AccessToken;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

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
    pub fn endpoints(&self) -> (&str, &str, &str) {
        match &self {
            Self::Microsoft => (AUTH_MS, TOKEN_MS, INFOS_MS),
            Self::Google => (AUTH_GG, TOKEN_GG, INFOS_GG),
        }
    }

    fn secrets(&self) -> (&str, &str) {
        match &self {
            Self::Microsoft => (ID_MS, SECRET_MS),
            Self::Google => (ID_GG, SECRET_GG),
        }
    }
}

pub fn get_authorization_token(f: Fournisseur) -> Result<AccessToken, Error> {
    let (id, secret) = f.secrets();
    let id = ClientId::new(id.to_owned());
    let secret = ClientSecret::new(secret.to_owned());

    let (url_auth, url_token, ..) = f.endpoints();
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
    webbrowser::open(authorize_url.as_ref())?;

    let mut request_line = String::new();
    let code;
    let state;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let reader = BufReader::new(&stream);
                reader.read_line(&mut request_line)?;

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&(format!("http://localhost{redirect_url}")))?;

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .expect("Le code d'autorisation doit être présent");

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .expect("Le jeton csrf doit être présent");

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());

                let message = "Retournez dans l'application 😎";
                let response = format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{message}", message.len());
                stream.write_all(response.as_bytes())?;

                break;
            }
            _ => return Err(anyhow!("La requête d'autorisation a échouée")),
        };
    }

    let token = client.exchange_code(code).set_pkce_verifier(pkce_code_verifier).request(http_client)?;
    Ok(token.access_token().to_owned())
}
