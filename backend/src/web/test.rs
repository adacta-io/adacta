use std::collections::HashMap;

use rocket::http::{ContentType, Status};

struct Server {
    pub authenticator: crate::auth::Authenticator,
    pub repository: crate::repository::Repository,
    pub index: crate::index::MockIndex,
    pub juicer: crate::juicer::MockJuicer,
    pub suggester: crate::suggester::MockSuggester,
}

impl Server {
    pub async fn new() -> Self {
        let mut api_keys = HashMap::new();
        api_keys.insert(String::from("test"), String::from("$2y$12$8X8eghlzYFEKYhcOdqIU6OqaC5oACEYpfLzJChXPPIHBO6aRmzXaC")); // "testkey"

        let authenticator = crate::auth::Authenticator::from_config(crate::config::Auth {
            passhash: "$2y$12$/luV8edFPQFt7Vc3O9MgReHsFoQUD0Vu4g9nkjFb/fK0ib3HwJ9/G".to_string(), // "pass"
            secret: "my dirty secret".to_string(),
            api_keys,
        }).await.unwrap();

        let repository = crate::repository::Repository::with_path(tempfile::tempdir().unwrap()).await.unwrap();

        let index = crate::index::MockIndex::new();
        let juicer = crate::juicer::MockJuicer::new();
        let suggester = crate::suggester::MockSuggester::new();

        return Server {
            authenticator,
            repository,
            index,
            juicer,
            suggester,
        };
    }

    pub async fn client(self) -> rocket::local::asynchronous::Client {
        let config = crate::config::Web { address: "127.0.0.1".to_string(), port: 0 };

        let rocket = crate::web::server(
            config,
            self.authenticator,
            self.repository,
            Box::new(self.index),
            Box::new(self.juicer),
            Box::new(self.suggester),
        ).unwrap();

        return rocket::local::asynchronous::Client::new(rocket).await.unwrap();
    }
}

mod frontend {
    use super::*;

    #[tokio::test]
    async fn test_index() {
        let server = Server::new().await;
        let client = server.client().await;

        let response_root = client.get("/").dispatch().await;
        assert_eq!(response_root.status(), Status::Ok);

        let response_root = response_root.into_string().await;
        assert_eq!(response_root.is_some(), true);

        let response_index = client.get("/index.html").dispatch().await;
        assert_eq!(response_index.status(), Status::Ok);

        let response_index = response_index.into_string().await;
        assert_eq!(response_index.is_some(), true);

        assert_eq!(response_index, response_root);
    }
}

mod api {
    use rocket::http::Header;

    use super::*;

    macro_rules! json_payload {
        ($($json:tt)+) => {
            serde_json::to_vec(&serde_json::json_internal!($($json)+)).unwrap()
        };
    }

    macro_rules! assert_json_eq {
        ($data:expr, $($json:tt)+) => {
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&($data)).unwrap(),
                serde_json::json_internal!($($json)+),
            )
        };
    }

    mod auth {
        use super::*;

        #[tokio::test]
        async fn test_login_success() {
            let server = Server::new().await;
            let client = server.client().await;

            let response = client.post("/api/auth/login")
                .header(ContentType::JSON)
                .body(json_payload!({
                    "password": "pass",
                }))
                .dispatch().await;

            assert_eq!(response.status(), Status::Accepted);
            assert!(response.headers().get("Authorization").next().is_some());
        }

        #[tokio::test]
        async fn test_login_failure() {
            let server = Server::new().await;
            let client = server.client().await;

            let response = client.post("/api/auth/login")
                .header(ContentType::JSON)
                .body(json_payload!({
                    "password": "wrong pass",
                }))
                .dispatch().await;

            assert_eq!(response.status(), Status::BadRequest);
            assert!(response.headers().get("Authorization").next().is_none());
        }
    }

    fn api_key() -> impl Into<Header<'static>> {
        let basic = format!("{}:{}", "test", "testkey");
        let basic = base64::encode(basic);

        return Header::new("Authorization", format!("Basic {}", basic));
    }

    mod upload {
        use mockall::predicate;
        use rand::RngCore;
        use rand::rngs::OsRng;

        use super::*;

        #[tokio::test]
        async fn test_upload() {
            let mut server = Server::new().await;

            server.juicer.expect_extract()
                .with(predicate::always())
                .times(1)
                .return_once(|_| Ok(()));

            let client = server.client().await;

            // Create the document from random data
            let mut doc = [0u8; 1024];
            OsRng.fill_bytes(&mut doc);

            let response = client.post("/api/upload")
                .header(ContentType::PDF)
                .header(api_key())
                .body(doc)
                .dispatch().await;


            assert_eq!(response.status(), Status::Ok);
        }
    }

    mod inbox {
        use std::collections::HashSet;
        use std::iter::FromIterator;

        use chrono::{DateTime, NaiveDateTime, Utc};
        use futures::{stream, StreamExt};
        use proto::model::{Kind, Label};
        use serde_json::json;
        use tokio::io::AsyncWriteExt;
        use tokio::time::Duration;

        use crate::meta::Metadata;

        use super::*;

        #[tokio::test]
        async fn test_list() {
            let server = Server::new().await;

            // Create bundles in inbox (with a short delay between each to have a unique timestamp)
            let ids = tokio::time::throttle(Duration::from_millis(10),
                                            stream::iter(0..13usize))
                .then(|_| async {
                    let bundle = server.repository.stage().await.unwrap();

                    Metadata {
                        uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                        ..Metadata::new()
                    }.save(bundle.write(Kind::Metadata).await.unwrap()).await.unwrap();

                    let bundle = bundle.create().await.unwrap();

                    *bundle.id()
                }).collect::<Vec<_>>().await;

            let client = server.client().await;

            let response = client.get("/api/inbox")
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);

            assert_json_eq!(response.into_bytes().await.unwrap(), {
                "count": 13,
                "docs": ids[0..10].iter().map(|id| json!({
                    "id": id,
                    "metadata": {
                        "archived": (),
                        "uploaded": "2001-09-09T01:46:40Z",
                        "pages": (),
                        "title": (),
                        "labels": [],
                        "properties": {},
                    }
                })).collect::<Vec<_>>(),
            });
        }

        #[tokio::test]
        async fn test_get() {
            let mut server = Server::new().await;

            let doc_id = {
                let staging = server.repository.stage().await.unwrap();

                staging.write(Kind::Document).await.unwrap()
                    .write_all(b"").await.unwrap();

                staging.write(Kind::Plaintext).await.unwrap()
                    .write_all(b"my document plaintext").await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(staging.write(Kind::Metadata).await.unwrap()).await.unwrap();

                *staging.create().await.unwrap().id()
            };

            server.suggester.expect_guess()
                .with(mockall::predicate::eq("my document plaintext"))
                .returning(|_| Ok(HashSet::from_iter(vec![Label::from("suggestion")])));

            let client = server.client().await;

            let response = client.get(format!("/api/inbox/{}", doc_id))
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);

            assert_json_eq!(response.into_bytes().await.unwrap(), {
                "id": doc_id.to_string(),
                "metadata": {
                    "archived": (),
                    "uploaded": "2001-09-09T01:46:40Z",
                    "pages": (),
                    "title": (),
                    "labels": [],
                    "properties": {},
                },
                "suggestions": ["suggestion"],
            });
        }

        #[tokio::test]
        async fn test_delete() {
            let server = Server::new().await;

            let doc_id = {
                let staging = server.repository.stage().await.unwrap();

                staging.write(Kind::Document).await.unwrap()
                    .write_all(b"").await.unwrap();

                staging.write(Kind::Plaintext).await.unwrap()
                    .write_all(b"my document plaintext").await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(staging.write(Kind::Metadata).await.unwrap()).await.unwrap();

                *staging.create().await.unwrap().id()
            };

            let client = server.client().await;

            let response = client.delete(format!("/api/inbox/{}", doc_id))
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);
        }

        #[tokio::test]
        async fn test_archive() {
            let mut server = Server::new().await;

            let doc_id = {
                let staging = server.repository.stage().await.unwrap();

                staging.write(Kind::Document).await.unwrap()
                    .write_all(b"").await.unwrap();

                staging.write(Kind::Plaintext).await.unwrap()
                    .write_all(b"my document plaintext").await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(staging.write(Kind::Metadata).await.unwrap()).await.unwrap();

                *staging.create().await.unwrap().id()
            };

            server.index.expect_index()
                .withf(move |bundle| bundle.id() == &doc_id)
                .returning(|_| Ok(()));

            server.suggester.expect_train()
                .with(mockall::predicate::eq("my document plaintext"),
                      mockall::predicate::eq(HashSet::from_iter(vec![Label::from("expected")])))
                .returning(|_, _| Ok(()));

            let client = server.client().await;

            let response = client.post(format!("/api/inbox/{}", doc_id))
                .header(api_key())
                .body(json_payload!({
                    "labels": [ "expected" ],
                    "properties": {
                        "title": "My little Test",
                        "source": "testsuite",
                    }
                }))
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);
        }
    }

    mod archive {
        use chrono::{DateTime, NaiveDateTime, Utc};
        use futures::{stream, StreamExt};
        use proto::model::Kind;
        use serde_json::json;
        use tokio::io::AsyncWriteExt;

        use crate::index::SearchResponse;
        use crate::meta::Metadata;

        use super::*;

        #[tokio::test]
        async fn test_get_bundle() {
            let server = Server::new().await;

            let doc_id = {
                let staging = server.repository.stage().await.unwrap();

                staging.write(Kind::Document).await.unwrap()
                    .write_all(b"").await.unwrap();

                staging.write(Kind::Plaintext).await.unwrap()
                    .write_all(b"my document plaintext").await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(staging.write(Kind::Metadata).await.unwrap()).await.unwrap();

                let inboxed = staging.create().await.unwrap();
                *inboxed.archive().await.unwrap().id()
            };

            let client = server.client().await;

            let response = client.get(format!("/api/archive/{}", doc_id))
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);

            assert_json_eq!(response.into_bytes().await.unwrap(), {
                "id": doc_id.to_string(),
                "metadata": {
                    "uploaded": "2001-09-09T01:46:40Z",
                    "archived": (),
                    "title": (),
                    "pages": (),
                    "labels": [],
                    "properties": {},
                }
            });
        }

        #[tokio::test]
        async fn test_get_fragment() {
            let server = Server::new().await;

            let doc_id = {
                let staging = server.repository.stage().await.unwrap();

                staging.write(Kind::Document).await.unwrap()
                    .write_all(b"").await.unwrap();

                staging.write(Kind::Plaintext).await.unwrap()
                    .write_all(b"my document plaintext").await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(staging.write(Kind::Metadata).await.unwrap()).await.unwrap();

                let inboxed = staging.create().await.unwrap();
                *inboxed.archive().await.unwrap().id()
            };

            let client = server.client().await;

            let response = client.get(format!("/api/archive/{}/plaintext", doc_id))
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);

            assert_eq!(response.into_bytes().await.unwrap(), b"my document plaintext");
        }

        #[tokio::test]
        async fn test_search() {
            let mut server = Server::new().await;

            // Create bundles in inbox (with a short delay between each to have a unique timestamp)
            let ids = stream::iter(0..10usize).then(|_| async {
                let bundle = server.repository.stage().await.unwrap();

                Metadata {
                    uploaded: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_000_000_000, 0), Utc),
                    ..Metadata::new()
                }.save(bundle.write(Kind::Metadata).await.unwrap()).await.unwrap();

                let bundle = bundle.create().await.unwrap();
                let bundle = bundle.archive().await.unwrap();

                *bundle.id()
            }).collect::<Vec<_>>().await;

            server.index.expect_search()
                .with(mockall::predicate::eq("testquery"))
                .return_once({
                    let ids = ids.clone();
                    move |_| Ok(SearchResponse {
                        count: 387,
                        docs: ids,
                    })
                });

            let client = server.client().await;

            let response = client.get("/api/archive?query=testquery")
                .header(api_key())
                .dispatch().await;

            assert_eq!(response.status(), Status::Ok);

            assert_json_eq!(response.into_bytes().await.unwrap(), {
                "count": 387,
                "docs": ids.iter().map(|id| json!({
                    "id": id,
                    "metadata": {
                        "uploaded": "2001-09-09T01:46:40Z",
                        "archived": (),
                        "title": (),
                        "pages": (),
                        "labels": [],
                        "properties": {},
                    }
                })).collect::<Vec<_>>(),
            });
        }
    }
}
