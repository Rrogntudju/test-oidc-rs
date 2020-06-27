mod session;
use lazy_static::lazy_static;
use session::{Session, SessionId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<SessionId, Session>>> = Arc::new(Mutex::new(HashMap::new()));
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
            .and(warp::post())
            .and(cookie::optional("Csrf-Token"))
            .and(header::optional("X-Csrf-Token"))
            .and(cookie::optional("Session-Id"))
            .and(json_body())
            .and(clone_sessions())
            .and_then(handlers::userinfos)
    }

    pub fn auth() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("auth").and(warp::get()).and(clone_sessions()).and_then(handlers::auth)
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
    use warp::http::{Response, StatusCode, Error};

    pub async fn userinfos(
        csrf_cookie: Option<String>,
        csrf_header: Option<String>,
        session_cookie: Option<String>,
        body: HashMap<String, String>,
        sessions: Arc<Mutex<HashMap<SessionId, Session>>>,
    ) -> Result<impl warp::Reply, Infallible> {
    
        // Validation Csrf si le cookie Csrf est présent
        if let Some(c_token) = csrf_cookie {
            match csrf_header {
                Some(h_token) if h_token != c_token => return Ok(reply_csrf_mismatch()),
                None => return Ok(reply_csrf_mismatch())
            }
        };

        if let Some(s_token) = session_cookie {

        }
        
        let result = sessions
            .lock()
            .expect("Failed due to poisoned lock")
            .insert(k, v);

       
        Ok(response)
    }

    fn reply_csrf_mismatch() -> Result<Response<String>, Error> {
        Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body("<h1>Csrf Token Mismatch!</h1>".to_string())
    }
    
    pub async fn auth(sessions: Arc<Mutex<HashMap<SessionId, Session>>>) -> Result<impl warp::Reply, Infallible> {
        Ok(reply_csrf_mismatch())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use warp::http::StatusCode;
    use warp::test::request;

    #[tokio::test]
    async fn static_file_ok() {
        let resp = request()
            .method("GET")
            .path("/static/userinfos.htm")
            .reply(&filters::static_file(PathBuf::from("./static")))
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
