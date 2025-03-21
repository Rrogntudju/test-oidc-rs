use crate::Fournisseur;
use anyhow::{anyhow, Context, Error};
use oauth2::basic::BasicClient;
use oauth2::{
    AccessToken, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
    TokenUrl,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::spawn_blocking;
use url::Url;

#[derive(Debug, Clone)]
pub struct Pkce {
    token: AccessToken,
    creation: Instant,
    expired_in: Duration,
}

impl Pkce {
    pub async fn new(f: &Fournisseur) -> Result<Self, Error> {
        let (id, secret) = f.secrets();
        let id = ClientId::new(id.to_owned());
        let secret = ClientSecret::new(secret.to_owned());

        let (url_auth, url_token) = f.endpoints();
        let url_auth = AuthUrl::new(url_auth.to_owned())?;
        let url_token = TokenUrl::new(url_token.to_owned())?;

        let client = BasicClient::new(id)
            .set_client_secret(secret)
            .set_auth_uri(url_auth)
            .set_token_uri(url_token)
            .set_auth_type(AuthType::RequestBody)
            .set_redirect_uri(RedirectUrl::new("http://localhost:86".to_owned())?);

        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_owned()))
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        let listener = TcpListener::bind("[::1]:86").context("TCP bind")?;
        let (rx, stop_signal) = start_listening(listener, csrf)?;
        if let Err(e) = webbrowser::open(authorize_url.as_ref()).context("open browser") {
            stop_signal.store(true, Ordering::Relaxed);
            return Err(e);
        }

        let code = spawn_blocking(move || match rx.recv() {
            Ok(code) => Ok(code),
            Err(_) => Err(anyhow!("Vous devez vous authentifier")),
        })
        .await??;

        let creation = Instant::now();
        let token = client.exchange_code(code).set_pkce_verifier(pkce_code_verifier).request(&ureq::agent())?;
        let expired_in = token.expires_in().unwrap_or(Duration::from_secs(3600));
        let token = token.access_token().to_owned();
        Ok(Self { token, creation, expired_in })
    }

    pub fn is_expired(&self) -> bool {
        self.creation.elapsed() >= self.expired_in
    }

    pub fn secret(&self) -> &String {
        self.token.secret()
    }
}

fn start_listening(listener: TcpListener, csrf: CsrfToken) -> Result<(Receiver<AuthorizationCode>, Arc<AtomicBool>), Error> {
    let (tx, rx) = sync_channel::<AuthorizationCode>(1);
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal2 = stop_signal.clone();
    listener.set_nonblocking(true).expect("Erreur set_nonblocking");

    std::thread::spawn(move || {
        let now = Instant::now();
        while !stop_signal2.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request_line = String::new();
                    let mut reader = BufReader::new(&stream);
                    reader.read_line(&mut request_line).unwrap();

                    let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                    let url = Url::parse(&(format!("http://localhost{redirect_url}"))).unwrap();
                    let code = url
                        .query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
                        .expect("Le code d'autorisation doit être présent");

                    let state = url
                        .query_pairs()
                        .find(|(key, _)| key == "state")
                        .map(|(_, state)| state)
                        .expect("Le jeton csrf doit être présent");

                    assert_eq!(csrf.secret(), state.as_ref());

                    let message = "<p>Retournez dans l'application &#128526;</p>";
                    let response = format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{message}", message.len());
                    stream.write_all(response.as_bytes()).expect("Erreur write_all");

                    let _ = tx.send(code);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if now.elapsed().as_secs() >= 150 {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => panic!("accept IO error: {e}"),
            }
        }
    });

    Ok((rx, stop_signal))
}
