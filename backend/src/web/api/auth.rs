use rocket::{Data, post, Request, Response, State};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::auth::{Authenticator, Token};
use crate::utils::StrExt;

use super::ApiError;

pub struct Authorization {}

#[async_trait::async_trait]
impl Fairing for Authorization {
    fn info(&self) -> Info {
        return Info {
            name: "Authorization",
            kind: Kind::Request | Kind::Response
        };
    }

    async fn on_request<'a>(&'a self, request: &'a mut Request<'_>, _data: &'a Data) {
        request.local_cache_async::<Option<Token>, _>(async {
            let auth = request.guard::<State<Authenticator>>().await.expect("No Authenticator");

            let header = request.headers().get_one("Authorization")?;
            let (kind, payload) = header.split2(' ')?;

            match kind {
                "Bearer" => {
                    let token = auth.verify_token(payload).await.ok()?;
                    return Some(token);
                }

                "Basic" => {
                    let payload = base64::decode(payload).ok()?;
                    let payload = String::from_utf8(payload).ok()?;

                    let (username, password) = payload.split2(':')?;
                    let token = auth.verify_key(username, password).await?;
                    return Some(token);
                }

                _ => {
                    return None;
                }
            }
        }).await;
    }

    async fn on_response<'a>(&'a self, request: &'a Request<'_>, response: &'a mut Response<'_>) {
        let token = request.local_cache_async::<Option<Token>, _>(async {
            return None;
        }).await;

        if let Some(token) = token {
            let auth = request.guard::<State<Authenticator>>().await.expect("No Authenticator");

            let bearer = auth.sign_token(token).await.expect("Can not sign token");
            response.set_header(Header::new("Authorization", bearer));
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
pub struct AuthResponse {
    pub username: String,
    pub token: String,
}

#[post("/auth", data = "<request>")]
pub(super) async fn auth(auth: State<'_, Authenticator>,
                         request: Json<AuthRequest>) -> Result<Json<AuthResponse>, ApiError> {
    if let Some(token) = auth.login(&request.username, &request.password).await {
        let bearer = auth.sign_token(&token).await.expect("Can not sign token");

        return Ok(Json(AuthResponse {
            username: request.username.clone(),
            token: bearer,
        }));

    } else {
        return Err(ApiError::bad_request("Authorization failed".to_string()));
    }
}
