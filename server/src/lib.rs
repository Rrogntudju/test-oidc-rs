mod session;
use lazy_static::lazy_static;
use session::{random_token, Session, SessionId};
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
    use warp::http::{Response, StatusCode};

    pub async fn userinfos(fpath: FullPath, mmds: Arc<Mutex<Mmds>>) -> Result<impl warp::Reply, Infallible> {
        let path = fpath.as_str().strip_prefix("/mds").unwrap();
        let result = mmds
            .lock()
            .expect("Failed to build MMDS response due to poisoned lock")
            .get_value(path.to_string());

        let response = match result {
            Ok(value) => Response::builder().status(StatusCode::OK).body(value.join("\n")),

            Err(e) => match e {
                MmdsError::NotFound => Response::builder().status(StatusCode::NOT_FOUND).body(format!("{}", e)),
                MmdsError::UnsupportedValueType => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(format!("{}", e)),
            },
        };
        let t = random_token(32);
        Ok(response)
    }
    /*
    pub async fn put_mds(data: Value, mmds: Arc<Mutex<Mmds>>) -> Result<impl warp::Reply, Infallible> {
        let result = mmds.lock().expect("Failed to build MMDS response due to poisoned lock").put_data(data);

        let response = match result {
            Ok(()) => Response::builder().status(StatusCode::NO_CONTENT).body(String::new()),

            Err(e) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(format!("{}", e)),
        };

        Ok(response)
    }

    pub async fn patch_mds(patch: Value, mmds: Arc<Mutex<Mmds>>) -> Result<impl warp::Reply, Infallible> {
        let result = mmds.lock().expect("Failed to build MMDS response due to poisoned lock").patch_data(patch);

        let response = match result {
            Ok(()) => Response::builder().status(StatusCode::NO_CONTENT).body(String::new()),

            Err(e) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(format!("{}", e)),
        };

        Ok(response)
    }*/
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
