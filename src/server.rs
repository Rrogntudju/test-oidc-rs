// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![feature(str_strip)]

pub mod filters {
    use super::*;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn get_mds(test: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("mds")
            .and(warp::get())
            .and(warp::path::full())
            .and(clone_mmds())
            .and_then(handlers::get_mds)
    }

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
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_parse_request() {
        let data = r#"{
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": "43",
            "phones": {
                "home": {
                    "RO": "+40 1234567",
                    "UK": "+44 1234567"
                },
                "mobile": "+44 2345678"
            }
        }"#;
        MMDS.lock().unwrap().put_data(serde_json::from_str(data).unwrap()).unwrap();

        let data = r#"{
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": 43
        }"#;
        assert_eq!(
            MMDS.lock().unwrap().put_data(serde_json::from_str(data).unwrap()),
            Err(MmdsError::UnsupportedValueType)
        );
    }

    #[test]
    fn test_json_patch() {
        let mut data = serde_json::json!({
            "name": {
                "first": "John",
                "second": "Doe"
            },
            "age": "43",
            "phones": {
                "home": {
                    "RO": "+40 1234567",
                    "UK": "+44 1234567"
                },
                "mobile": "+44 2345678"
            }
        });

        let patch = serde_json::json!({
            "name": {
                "second": null,
                "last": "Kennedy"
            },
            "age": "44",
            "phones": {
                "home": "+44 1234567",
                "mobile": {
                    "RO": "+40 2345678",
                    "UK": "+44 2345678"
                }
            }
        });
        json_patch(&mut data, &patch);

        // Test value replacement in target document.
        assert_eq!(data["age"], patch["age"]);

        // Test null value removal from target document.
        assert_eq!(data["name"]["second"], Value::Null);

        // Test add value to target document.
        assert_eq!(data["name"]["last"], patch["name"]["last"]);
        assert!(!data["phones"]["home"].is_object());
        assert_eq!(data["phones"]["home"], patch["phones"]["home"]);
        assert!(data["phones"]["mobile"].is_object());
        assert_eq!(data["phones"]["mobile"]["RO"], patch["phones"]["mobile"]["RO"]);
        assert_eq!(data["phones"]["mobile"]["UK"], patch["phones"]["mobile"]["UK"]);
    }

    use warp::http::StatusCode;
    use warp::test::request;

    #[tokio::test]
    async fn put_patch_get_ok() {
        let resp = request()
            .method("PUT")
            .path("/mds")
            .body(r#"{"c0":{"c1":"12345","c2":"6789"}}"#)
            .reply(&filters::put_mds())
            .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp = request()
            .method("PATCH")
            .path("/mds")
            .body(r#"{"c0":{"c3":["67890","a"]}}"#)
            .reply(&filters::patch_mds())
            .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp = request().method("GET").path("/mds/c0").reply(&filters::get_mds()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "c1\nc2\nc3");

        let resp = request().method("GET").path("/mds/c0/c3/0").reply(&filters::get_mds()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "67890");
    }
}
