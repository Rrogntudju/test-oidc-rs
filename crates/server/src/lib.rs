mod session;
use lazy_static::lazy_static;
use serde_json::{Map, Value};
use session::{random_token, Session, SessionId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref SESSIONS: Arc<RwLock<HashMap<SessionId, Session>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref LOL: String = String::default();
    static ref LOL_MAP: Map<String, Value> = Map::default();
}

pub mod filters {

    use super::*;
    use std::convert::Infallible;
    use std::path::PathBuf;
    use warp::filters::{cookie, header};
    use warp::Filter;

    pub fn static_file(path: PathBuf) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("static").and(warp::fs::dir(path))
    }

    pub fn userinfos() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("userinfos")
            .and(warp::path::end())
            .and(warp::post())
            .and(cookie::optional("Csrf-Token"))
            .and(header::optional("X-Csrf-Token"))
            .and(cookie::optional("Session-Id"))
            .and(json_body())
            .and(clone_sessions())
            .and_then(handlers::userinfos)
    }

    pub fn auth() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("auth")
            .and(warp::path::end())
            .and(warp::get())
            .and(cookie::optional("Session-Id"))
            .and(warp::query::<HashMap<String, String>>())
            .and(clone_sessions())
            .and_then(handlers::auth)
    }

    fn clone_sessions() -> impl Filter<Extract = (Arc<RwLock<HashMap<SessionId, Session>>>,), Error = Infallible> + Clone {
        warp::any().map(move || SESSIONS.clone())
    }

    fn json_body() -> impl Filter<Extract = (HashMap<String, String>,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024).and(warp::body::json())
    }
}

mod handlers {
    use crate::session::Fournisseur;
    use oauth2::reqwest::async_http_client;
    use oauth2::{basic::BasicClient, AuthType, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl};
    use oauth2::{AuthorizationCode, TokenResponse};
    use session::Token;
    use std::time::Duration;

    use super::*;
    use std::convert::Infallible;
    use warp::http::{Error, Response, StatusCode};

    pub async fn userinfos(
        csrf_cookie: Option<String>,
        csrf_header: Option<String>,
        session_cookie: Option<String>,
        body: HashMap<String, String>,
        sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    ) -> Result<impl warp::Reply, Infallible> {
        // Validation Csrf si le cookie Csrf est présent
        if let Some(ctoken) = csrf_cookie {
            match csrf_header {
                Some(htoken) if htoken == ctoken => (),
                Some(htoken) if htoken != ctoken => {
                    eprintln!("{htoken} != {ctoken}");
                    return Ok(reply_error(StatusCode::FORBIDDEN));
                }
                _ => {
                    eprintln!("X-Csrf-Token est absent");
                    return Ok(reply_error(StatusCode::FORBIDDEN));
                }
            }
        };

        let fournisseur = body.get("fournisseur").unwrap_or(&LOL);
        let origine = body.get("origine").unwrap_or(&LOL);

        let response = match session_cookie {
            Some(stoken) => {
                let id: SessionId = stoken.into();
                // MutexGuard n'est pas Send
                let session = sessions.read().expect("Failed due to poisoned lock").get(&id).cloned();
                match session {
                    Some(session) => {
                        match session {
                            session if session.is_expired() => {
                                eprintln!("userinfos: session expirée");
                                sessions.write().expect("Failed due to poisoned lock").remove(&id);
                                reply_redirect_fournisseur(fournisseur, origine, sessions)
                            }
                            Session::Authenticated(f, token) if &f.to_string() == fournisseur => {
                                let client = reqwest::Client::new();
                                let response = match client.get(f.userinfos()).bearer_auth(token.secret()).send().await {
                                    Ok(response) => response,
                                    Err(e) => {
                                        eprintln!("{e}");
                                        return Ok(reply_error(StatusCode::INTERNAL_SERVER_ERROR));
                                    }
                                };
                                let userinfo = response.json::<Value>().await.unwrap_or_default();
                                let map = userinfo.as_object().unwrap_or(&LOL_MAP);
                                let infos = map
                                    .iter()
                                    .map(|(k, v)| {
                                        let mut map = serde_json::Map::new();
                                        map.insert("propriété".into(), Value::String(k.to_owned()));
                                        map.insert("valeur".into(), v.to_owned());
                                        Value::Object(map)
                                    })
                                    .collect::<Vec<Value>>();

                                Response::builder()
                                    .status(StatusCode::OK)
                                    .body(serde_json::to_string(&infos).unwrap_or_default())
                            }
                            _ => {
                                // Changement de fournisseur
                                sessions.write().expect("Failed due to poisoned lock").remove(&id);
                                reply_redirect_fournisseur(fournisseur, origine, sessions)
                            }
                        }
                    }
                    None => reply_redirect_fournisseur(fournisseur, origine, sessions),
                }
            }
            None => reply_redirect_fournisseur(fournisseur, origine, sessions),
        };

        Ok(response)
    }

    fn reply_error(sc: StatusCode) -> Result<Response<String>, Error> {
        Response::builder().status(sc).body(String::default())
    }

    fn reply_redirect_fournisseur(
        fournisseur: &str,
        origine: &str,
        sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    ) -> Result<Response<String>, Error> {
        let f: Fournisseur = fournisseur.into();
        let (id, secret) = f.secrets();
        let id = ClientId::new(id.to_owned());
        let secret = ClientSecret::new(secret.to_owned());

        let (url_auth, url_token) = f.endpoints();
        let auth_url = AuthUrl::new(url_auth.to_owned()).unwrap();
        let token_url = TokenUrl::new(url_token.to_owned()).unwrap();

        let client = BasicClient::new(id, Some(secret), auth_url, Some(token_url))
            .set_auth_type(AuthType::RequestBody)
            .set_redirect_uri(RedirectUrl::new(origine.to_string() + "/auth").unwrap());

        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_owned()))
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .url();

        let sessionid = SessionId::new();
        let response = Response::builder()
            .status(StatusCode::OK)
            // Lax temporairement nécessaire pour l'envoi du cookie Session-Id avec le redirect par OP
            .header("Set-Cookie", format!("Session-Id={0}; SameSite=Lax", sessionid.as_ref()))
            .header("Set-Cookie", format!("Csrf-Token={0}; SameSite=Strict", random_token(64)))
            .body(format!(r#"{{ "redirectOP": "{}" }}"#, authorize_url.as_str()));

        let session = Session::new(f, client, csrf_state);
        sessions.write().expect("Failed due to poisoned lock").insert(sessionid, session);

        response
    }

    pub async fn auth(
        session_cookie: Option<String>,
        params: HashMap<String, String>,
        sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = match session_cookie {
            Some(stoken) => {
                let id = SessionId::from(stoken);
                let session = if let Some(session) = sessions.write().expect("Failed due to poisoned lock").remove(&id) {
                    session
                } else {
                    eprintln!("auth: session inexistante");
                    return Ok(reply_error(StatusCode::BAD_REQUEST));
                };

                let code = if let Some(code) = params.get("code") {
                    code
                } else {
                    eprintln!("auth: auth code manquant");
                    return Ok(reply_error(StatusCode::BAD_REQUEST));
                };

                let state = if let Some(state) = params.get("state") {
                    state
                } else {
                    eprintln!("auth: csrf manquant");
                    return Ok(reply_error(StatusCode::BAD_REQUEST));
                };

                let (client, csrf) = match session {
                    Session::AuthenticationRequested(_, ref c, ref csrf) => (c, csrf),
                    _ => {
                        eprintln!("auth: session déjà authentifiée");
                        return Ok(reply_error(StatusCode::BAD_REQUEST));
                    }
                };

                if state != csrf.secret() {
                    eprintln!("auth: csrf invalide");
                    return Ok(reply_error(StatusCode::BAD_REQUEST));
                }

                let token = match client
                    .exchange_code(AuthorizationCode::new(code.to_owned()))
                    .request_async(async_http_client)
                    .await
                {
                    Ok(token) => token,
                    Err(e) => {
                        eprintln!("{e}");
                        return Ok(reply_error(StatusCode::BAD_REQUEST));
                    }
                };

                let expired_in = token.expires_in().unwrap_or(Duration::from_secs(60));
                let token = Token::new(token.access_token().to_owned(), expired_in);

                let response = Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/static/userinfos.htm")
                    // Après le redirect par OP, réécrire le cookie Session-Id avec Strict
                    .header("Set-Cookie", format!("Session-Id={0}; SameSite=Strict", id.as_ref()))
                    .body(String::default());

                sessions
                    .write()
                    .expect("Failed due to poisoned lock")
                    .insert(id, session.authentication_completed(token));

                response
            }
            None => {
                eprintln!("auth: Cookie de session inexistant");
                reply_error(StatusCode::BAD_REQUEST)
            }
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use warp::http::StatusCode;
    use warp::test::request;

    #[tokio::test]
    async fn static_file() {
        let resp = request()
            .method("GET")
            .path("/static/userinfos.htm")
            .reply(&filters::static_file(PathBuf::from("../static")))
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn csrf_mismatch1() {
        let resp = request()
            .method("POST")
            .path("/userinfos")
            .header("Cookie", "Csrf-Token=LOL")
            .body(r#"{"fournisseur": "Google", "origine": "http://localhost"}"#)
            .reply(&filters::userinfos())
            .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn csrf_mismatch2() {
        let resp = request()
            .method("POST")
            .path("/userinfos")
            .header("Cookie", "Csrf-Token=LOL")
            .header("X-Csrf-Token", "BOUH!")
            .body(r#"{"fournisseur": "Google", "origine": "http://localhost"}"#)
            .reply(&filters::userinfos())
            .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn no_session_cookie1() {
        let resp = request()
            .method("POST")
            .path("/userinfos")
            .body(r#"{"fournisseur": "Microsoft", "origine": "http://localhost"}"#)
            .reply(&filters::userinfos())
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.body().starts_with(b"{ \"redirectOP\": \"https://"));
    }

    #[tokio::test]
    async fn no_session_cookie2() {
        let resp = request().method("GET").path("/auth?code=LOL").reply(&filters::auth()).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
