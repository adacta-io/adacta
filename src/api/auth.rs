use std::ops::Add;
use std::time::{Duration, Instant, SystemTime};

use rocket::{Data, post, Request, Responder, Response, State};
use rocket::http::{Header, HeaderMap, RawStr, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::fairing::{Fairing, Info, Kind};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::api::{ApiError, InternalError};
use crate::auth::{Authenticator, Token};
use crate::index::Index;
use crate::model::DocId;
use crate::repo::Repository;
use rocket::response::ResponseBuilder;

pub struct Authentication {}

#[async_trait::async_trait]
impl Fairing for Authentication {
    fn info(&self) -> Info {
        return Info {
            name: "Authentication",
            kind: Kind::Request | Kind::Response
        };
    }

    async fn on_request<'a>(&'a self, request: &'a mut Request<'_>, data: &'a Data) {
        request.local_cache_async::<Option<Token>, _>(async {
            let auth = request.guard::<State<Authenticator>>().await.expect("No Authenticator");

            let bearer = request.headers().get_one("Authentication")?;
            let token = auth.verify_token(bearer).await.ok()?;

            return Some(token);
        }).await;
    }

    async fn on_response<'a>(&'a self, request: &'a Request<'_>, response: &'a mut Response<'_>) {
        let token = request.local_cache_async::<Option<Token>, _>(async {
            return None;
        }).await;

        if let Some(token) = token {
            let auth = request.guard::<State<Authenticator>>().await.expect("No Authenticator");

            let bearer = auth.sign_token(token).await.expect("Can not sign token");
            response.set_header(Header::new("Authentication", bearer));
        }
    }
}

#[async_trait::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for &'a Token {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        return match request.local_cache_async::<Option<Token>, _>(async {
            return None;
        }).await {
            Some(token) => Outcome::Success(token),
            None => Outcome::Failure((Status::Unauthorized, ())),
        };
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthDetails {
    pub username: String,
}

#[derive(Responder, Debug)]
#[response(status = 200, content_type = "json")]
pub struct AuthResponse {
    pub details: Json<AuthDetails>,
    pub authentication: Header<'static>,
}

#[post("/auth", data = "<request>")]
pub(super) async fn auth(auth: State<'_, Authenticator>,
                         request: Json<AuthRequest>) -> Result<AuthResponse, ApiError> {
    if let Some(token) = auth.login(&request.username, &request.password).await {
        let bearer = auth.sign_token(&token).await.expect("Can not sign token");

        return Ok(AuthResponse {
            details: Json(AuthDetails {
                username: request.username.clone(),
            }),
            authentication: Header::new("Authentication", bearer)
        });

    } else {
        return Err(ApiError::bad_request("Authentication failed".to_string()));
    }
}
