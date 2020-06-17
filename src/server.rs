pub mod filters {
    use super::*;
    use std::path::PathBuf;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn static_file(path: PathBuf) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("static").and(warp::fs::dir(path))
    }
/*
    pub fn put_mds() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("mds")
            .and(warp::path::end())
            .and(warp::put())
            .and(json_body())
            .and(clone_mmds())
            .and_then(handlers::put_mds)
    }

    pub fn patch_mds() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("mds")
            .and(warp::path::end())
            .and(warp::patch())
            .and(json_body())
            .and(clone_mmds())
            .and_then(handlers::patch_mds)
    }

    fn json_body() -> impl Filter<Extract = (Value,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(10240).and(warp::body::json())
    }

    fn clone_mmds() -> impl Filter<Extract = (Arc<Mutex<Mmds>>,), Error = Infallible> + Clone {
        warp::any().map(move || MMDS.clone())
    }
*/
}
/*
pub mod handlers {
    use super::*;
    use std::convert::Infallible;
    use warp::filters::path::FullPath;
    use warp::http::{Response, StatusCode};

    pub async fn get_mds(fpath: FullPath, mmds: Arc<Mutex<Mmds>>) -> Result<impl warp::Reply, Infallible> {
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

        Ok(response)
    }

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
    }
}
*/
#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;
    use warp::test::request;
    use std::path::PathBuf;

    #[tokio::test]
    async fn static_file_ok() {
        let resp = request()
            .method("GET")
            .path("static/userinfos.htm")
            .reply(&filters::static_file("../../static".parse::<PathBuf>().unwrap()))
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
