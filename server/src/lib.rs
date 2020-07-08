mod session;
use lazy_static::lazy_static;
use session::{random_token, Session, SessionId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<SessionId, Session>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref MS: String = "Microsoft".into();
}

const ID_MS: &str = include_str!("clientid.microsoft");
const SECRET_MS: &str = include_str!("secret.microsoft");
const ID_GG: &str = include_str!("clientid.google");
const SECRET_GG: &str = include_str!("secret.google");

pub mod filters {
    use super::*;
    use std::convert::Infallible;
    use std::path::PathBuf;
    use warp::filters::{cookie, header, reply};
    use warp::Filter;

    pub fn static_file(path: PathBuf) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("static").and(warp::fs::dir(path))
    }

    pub fn userinfos() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("userinfos")
            .and(warp::post())
            .and(cookie::optional("Csrf-Token"))
            .and(header::optional("X-Csrf-Token"))
            .and(cookie::optional("Session-Id"))
            .and(json_body())
            .and(clone_sessions())
            .and_then(handlers::userinfos)
            .with(reply::header("Set-Cookie", format!("Csrf-Token={0}; SameSite=Strict", random_token(64))))
    }

    pub fn auth() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("auth")
            .and(warp::get())
            .and(cookie::optional("Session-Id"))
            .and(warp::path::param().map(|code: String| code))
            .and(clone_sessions())
            .and_then(handlers::auth)
    }

    fn clone_sessions() -> impl Filter<Extract = (Arc<Mutex<HashMap<SessionId, Session>>>,), Error = Infallible> + Clone {
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
        sessions: Arc<Mutex<HashMap<SessionId, Session>>>,
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

        let fournisseur = body.get("fournisseur").unwrap_or(&MS);

        let response = match session_cookie {
            Some(stoken) => {
                let id = SessionId::from(stoken);
                let lock = sessions.lock().expect("Failed due to poisoned lock");

                match lock.get(&id) {
                    None => {
                        eprintln!("userinfos: Pas de session");
                        drop(lock);
                        reply_redirect_fournisseur(fournisseur, sessions)
                    }
                    Some(session) if !session.is_authenticated() => {
                        eprintln!("userinfos: Session pas authentifiée");
                        reply_error(StatusCode::BAD_REQUEST)
                    }
                    Some(session) if session.is_expired() => {
                        eprintln!("userinfos: Session expirée");
                        drop(lock);
                        sessions.lock().expect("Failed due to poisoned lock").remove(&id);
                        reply_redirect_fournisseur(fournisseur, sessions)
                    }
                    Some(session) => reply_userinfos(session),
                }
            }
            None => reply_redirect_fournisseur(fournisseur, sessions),
        };

        Ok(response)
    }

    fn reply_error(sc: StatusCode) -> Result<Response<String>, Error> {
        Response::builder().status(sc).body(String::default())
    }

    fn reply_userinfos(session: &Session) -> Result<Response<String>, Error> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("<h1>Csrf Token Mismatch!</h1>".to_string())
    }

    fn reply_redirect_fournisseur(fournisseur: &str, sessions: Arc<Mutex<HashMap<SessionId, Session>>>) -> Result<Response<String>, Error> {
        use oidc::{Client, Options};
        use url::Url;

        let (id, secret, issuer) = match fournisseur {
            "Google" => (ID_GG, SECRET_GG, oidc::issuer::google()),
            "Microsoft" | _ => (ID_MS, SECRET_MS, oidc::issuer::microsoft()),
        };

        let redirect = match Url::parse("http://localhost/auth") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{0}", e.to_string());
                return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let client = match Client::discover(id.into(), secret.into(), redirect, issuer) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{0}", e.to_string());
                return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut options = Options::default();
        options.nonce = Some(random_token(64));
        let auth_url = client.auth_url(&Options::default());

        let sessionid = SessionId::new();
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Set-Cookie", format!("Session-Id={0}; SameSite=Strict", sessionid.0))
            .body(format!(r#"{{ "redirectOP": "{0}" }}"#, auth_url.to_string()));

        let session = Session::new(client, options.nonce.clone().unwrap());
        sessions.lock().expect("Failed due to poisoned lock").insert(sessionid, session);

        response
    }

    pub async fn auth(
        session_cookie: Option<String>,
        code: String,
        sessions: Arc<Mutex<HashMap<SessionId, Session>>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = match session_cookie {
            Some(stoken) => {
                let id = SessionId::from(stoken);
                let lock = sessions.lock().expect("Failed due to poisoned lock");

                match lock.get(&id) {
                    None => {
                        eprintln!("auth: Session inexistante");
                        reply_error(StatusCode::FORBIDDEN)
                    }
                    Some(session) if session.is_authenticated() => {
                        eprintln!("auth: Session déjà authentifiée");
                        drop(lock);
                        sessions.lock().expect("Failed due to poisoned lock").remove(&id);
                        reply_error(StatusCode::BAD_REQUEST)
                    }
                    Some(_) => {
                        drop(lock);
                        reply_redirect_userinfos(&id, sessions, &code)
                    }
                }
            }
            None => {
                eprintln!("auth: Cookie de session inexistant");
                reply_error(StatusCode::BAD_REQUEST)
            }
        };

        Ok(response)
    }

    fn reply_redirect_userinfos(id: &SessionId, sessions: Arc<Mutex<HashMap<SessionId, Session>>>, code: &str) -> Result<Response<String>, Error> {
        let response;
        let mut lock = sessions.lock().expect("Failed due to poisoned lock");

        match lock.get_mut(id) {
            None => {
                eprintln!("auth: Session inexistante");
                return reply_error(StatusCode::FORBIDDEN);
            }
            Some(session) => {
                let (client, nonce) = match session {
                    Session::AuthenticationRequested(c, n) => (c.take().unwrap(), n.clone()),
                    _ => return reply_error(StatusCode::BAD_REQUEST),
                };
                let token = match client.authenticate(&code, Some(&nonce), None) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{0}", e.to_string());
                        return reply_error(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };
                response = Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "http://localhost/userinfos")
                    .body(String::default());
                session.authentication_completed(client, token);
            }
        };

        response
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
            .reply(&filters::static_file(PathBuf::from("./static")))
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn csrf_mismatch() {
        let resp = request()
            .method("POST")
            .path("userinfos")
            .header("X-Csrf-Token", "LOL")
            .body(r#"{"fournisseur": "Google"}"#)
            .reply(&filters::userinfos())
            .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn no_session_cookie() {
        let resp = request()
            .method("POST")
            .path("userinfos")
            .body(r#"{"fournisseur": "Google"}"#)
            .reply(&filters::userinfos())
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.body().starts_with(b"{ \"redirectOP\": \"https://"));
    }
}
