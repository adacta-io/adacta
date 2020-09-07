use anyhow::Error;
use rocket::{Request, Response};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::response::status::{BadRequest, NotFound};

#[derive(Debug)]
pub(super) struct InternalError(pub Error);

impl<'r, 'o: 'r> Responder<'r, 'o> for InternalError {
    fn respond_to(self, request: &'r Request<'_>) -> Result<Response<'o>, Status> {
        let message = format!("{:#?}", self.0);

        Response::build_from(message.respond_to(request)?)
            .status(Status::InternalServerError)
            .ok()
    }
}

impl From<Error> for InternalError {
    fn from(err: Error) -> Self { Self(err) }
}

#[derive(Debug, Responder)]
pub(super) enum ApiError {
    NotFound(NotFound<String>),
    BadRequest(BadRequest<String>),
    InternalError(InternalError),
}

impl ApiError {
    pub const fn not_found(s: String) -> Self { Self::NotFound(NotFound(s)) }

    pub const fn bad_request(s: String) -> Self { Self::BadRequest(BadRequest(Some(s))) }
}

impl From<NotFound<String>> for ApiError {
    fn from(r: NotFound<String>) -> Self { Self::NotFound(r) }
}

impl From<BadRequest<String>> for ApiError {
    fn from(r: BadRequest<String>) -> Self { Self::BadRequest(r) }
}

impl From<InternalError> for ApiError {
    fn from(r: InternalError) -> Self { Self::InternalError(r) }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self { Self::InternalError(err.into()) }
}
