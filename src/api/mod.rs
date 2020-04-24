use std::io::Cursor;

use anyhow::Error;
use rocket::{http::RawStr, http::Status, Request, request::FromParam, Response, response::status::{BadRequest, NotFound}, Route, routes};
use rocket::response::Responder;

use async_trait::async_trait;
pub use auth::Authentication;

use crate::model::DocId;

mod auth;
mod repo;
mod search;
mod inbox;
mod upload;

pub fn routes() -> Vec<Route> {
    return routes![
        auth::auth,

        repo::bundle,
        repo::fragment,

        search::search,
        inbox::inbox,

        upload::upload_pdf,
    ];
}

impl FromParam<'_> for DocId {
    type Error = Error;

    fn from_param(param: &'_ RawStr) -> Result<Self, Self::Error> {
        return Ok(param.parse()?);
    }
}

#[derive(Debug)]
pub(self) struct InternalError(pub Error);

#[async_trait]
impl <'r> Responder<'r> for InternalError {
    async fn respond_to(self, _request: &'r Request<'_>) -> Result<Response<'r>, Status> {
        return Response::build()
            .status(Status::InternalServerError)
            .sized_body(Cursor::new(format!("{:#?}", self.0))).await
            .ok();
    }
}

impl From<Error> for InternalError {
    fn from(err: Error) -> Self {
        return Self(err);
    }
}

#[derive(Debug, Responder)]
pub(self) enum ApiError {
    NotFound(NotFound<String>),
    BadRequest(BadRequest<String>),
    InternalError(InternalError),
}

impl ApiError {
    pub fn not_found(s: String) -> Self {
        return Self::NotFound(NotFound(s));
    }

    pub fn bad_request(s: String) -> Self {
        return Self::BadRequest(BadRequest(Some(s)));
    }
}

impl From<NotFound<String>> for ApiError {
    fn from(r: NotFound<String>) -> Self {
        return ApiError::NotFound(r);
    }
}

impl From<BadRequest<String>> for ApiError {
    fn from(r: BadRequest<String>) -> Self {
        return ApiError::BadRequest(r);
    }
}

impl From<InternalError> for ApiError {
    fn from(r: InternalError) -> Self {
        return ApiError::InternalError(r);
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        return ApiError::InternalError(err.into());
    }
}
