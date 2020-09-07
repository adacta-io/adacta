use rocket::{Data, post, Request, Response, State};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket_contrib::json::Json;
use serde::Deserialize;

use crate::auth::Authenticator;
pub use crate::auth::Token;
use crate::utils::StrExt;

use async_trait::async_trait;

pub struct Authorization {}

#[async_trait]
impl Fairing for Authorization {
    fn info(&self) -> Info {
        Info {
            name: "Authorization",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &Data) {
        request.local_cache_async::<Option<Token>, _>(async {
            let auth = request
                .guard::<State<'_, Authenticator>>()
                .await
                .expect("No Authenticator");

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

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let token = request.local_cache_async::<Option<Token>, _>(async {
            return None;
        }).await;

        if let Some(token) = token {
            let auth = request.guard::<State<'_, Authenticator>>().await
                .expect("No Authenticator");

            let bearer = auth.sign_token(token).await
                .expect("Can not sign token");

            response.set_header(Header::new("Authorization", bearer));
        }
    }
}

#[async_trait::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for &'a Token {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let token = request.local_cache_async::<Option<Token>, _>(async {
            return None;
        }).await;

        match token {
            Some(token) => Outcome::Success(token),
            None => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub password: String,
}

#[post("/auth/login", data = "<request>")]
pub(super) async fn login(auth: State<'_, Authenticator>,
                          request: Json<AuthRequest>) -> Response<'_> {
    if let Some(token) = auth.login(&request.password).await {
        let bearer = auth.sign_token(&token).await.expect("Can not sign token");

        return Response::build()
            .header(Header::new("Authorization", bearer))
            .status(Status::Accepted)
            .finalize();

    } else {
        return Response::build()
            .status(Status::BadRequest)
            .finalize();
    }
}
