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

const ID_MS: &str = include_str!("clientid.microsoft");
const SECRET_MS: &str = include_str!("secret.microsoft");
const ID_GG: &str = include_str!("clientid.google");
const SECRET_GG: &str = include_str!("secret.google");

pub mod filters {
    use super::*;
    use std::convert::Infallible;
    use std::path::PathBuf;
    use warp::filters::{cookie, header};
    use warp::Filter;

    pub fn static_file(path: PathBuf) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("static").and(warp::fs::dir(path))
    }

    pub fn userinfos() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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

    pub fn auth() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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
                    eprintln!("{0} != {1}", htoken, ctoken);
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
                let lock = sessions.read().expect("Failed due to poisoned lock");

                match lock.get(&id) {
                    Some(session) if session.is_expired() => {
                        drop(lock);
                        eprintln!("userinfos: Session expirée ou pas authentifiée");
                        sessions.write().expect("Failed due to poisoned lock").remove(&id);
                        reply_redirect_fournisseur(fournisseur, origine, sessions)
                    }
                    Some(session) => {
                        match session {
                            Session::Authenticated(client, f, token) if f == fournisseur => {
                                let http = reqwest::Client::new();
                                let userinfo = match client.request_userinfo(&http, token) {
                                    Ok(userinfo) => userinfo,
                                    Err(e) => {
                                        eprintln!("{0}", e.to_string());
                                        return Ok(reply_error(StatusCode::INTERNAL_SERVER_ERROR));
                                    }
                                };
                                drop(lock);

                                let value = serde_json::to_value(&userinfo).unwrap_or_default();
                                let map = value.as_object().unwrap_or(&LOL_MAP);
                                let infos = Value::Array(
                                    map.into_iter()
                                        .filter_map(|(k, v)| match v.is_null() {
                                            true => None,
                                            false => {
                                                let mut map = serde_json::Map::new();
                                                map.insert("propriété".into(), Value::String(k.to_owned()));
                                                map.insert("valeur".into(), v.to_owned());
                                                Some(Value::Object(map))
                                            }
                                        })
                                        .collect::<Vec<Value>>(),
                                );

                                Response::builder()
                                    .status(StatusCode::OK)
                                    .body(serde_json::to_string(&infos).unwrap_or_default())
                            }
                            _ => {
                                // Changement de fournisseur
                                drop(lock);
                                sessions.write().expect("Failed due to poisoned lock").remove(&id);
                                reply_redirect_fournisseur(fournisseur, origine, sessions)
                            }
                        }
                    }
                    None => {
                        drop(lock);
                        eprintln!("userinfos: Pas de session");
                        reply_redirect_fournisseur(fournisseur, origine, sessions)
                    }
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
        use oidc::{issuer, Client, Options};
        use reqwest::Url;

        let (id, secret, issuer) = match fournisseur {
            "Google" => (ID_GG, SECRET_GG, issuer::google()),
            "Microsoft" => (ID_MS, SECRET_MS, issuer::microsoft_tenant("consumers/v2.0")),
            _ => {
                eprintln!("{0}", "Fournisseur invalide");
                return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let url_redirect = origine.to_string() + "/auth";
        let redirect = match Url::parse(&url_redirect) {
            Ok(redirect) => redirect,
            Err(e) => {
                eprintln!("{0}", e.to_string());
                return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let client = match Client::discover(id.into(), secret.into(), redirect, issuer) {
            Ok(client) => client,
            Err(e) => {
                eprintln!("{0}", e.to_string());
                return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut options = Options::default();
        let nonce = random_token(64);
        options.nonce = Some(nonce.clone());
        options.scope = Some("email profile".into());
        let auth_url = client.auth_url(&options);

        let sessionid = SessionId::new();
        let response = Response::builder()
            .status(StatusCode::OK)
            // Lax temporairement nécessaire pour l'envoi du cookie Session-Id avec le redirect par OP
            .header("Set-Cookie", format!("Session-Id={0}; SameSite=Lax", sessionid.as_ref()))
            .header("Set-Cookie", format!("Csrf-Token={0}; SameSite=Strict", random_token(64)))
            .body(format!(r#"{{ "redirectOP": "{0}" }}"#, auth_url.to_string()));

        let session = Session::new(client, fournisseur.into(), nonce);
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
                let session = match sessions.write().expect("Failed due to poisoned lock").remove(&id) {
                    Some(session) => session,
                    None => {
                        eprintln!("auth: session inexistante");
                        return Ok(reply_error(StatusCode::BAD_REQUEST));
                    }
                };

                let code = match params.get("code") {
                    Some(code) => code,
                    None => {
                        eprintln!("auth: auth code manquant");
                        return Ok(reply_error(StatusCode::BAD_REQUEST));
                    }
                };

                let (client, nonce) = match session {
                    Session::AuthenticationRequested(ref c, _, ref n) => (c, n),
                    _ => {
                        eprintln!("auth: session déjà authentifiée");
                        return Ok(reply_error(StatusCode::BAD_REQUEST));
                    }
                };

                let token = match client.authenticate(code, Some(nonce), None) {
                    Ok(token) => token,
                    Err(e) => {
                        eprintln!("{0}", e.to_string());
                        return Ok(reply_error(StatusCode::INTERNAL_SERVER_ERROR));
                    }
                };

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
