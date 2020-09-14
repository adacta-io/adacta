use anyhow::Error;
use log::error;
use rocket::{Request, Response};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::response::status::NotFound;

#[derive(Debug)]
pub(super) struct InternalError(pub Error);

impl<'r, 'o: 'r> Responder<'r, 'o> for InternalError {
    fn respond_to(self, request: &'r Request<'_>) -> Result<Response<'o>, Status> {
        error!("Error processing request: {:?}", request);
        error!("{:?}", self.0);

        let message = format!("{:#}", self.0);

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
    InternalError(InternalError),
}

impl ApiError {
    pub const fn not_found(s: String) -> Self { Self::NotFound(NotFound(s)) }
}

impl From<NotFound<String>> for ApiError {
    fn from(r: NotFound<String>) -> Self { Self::NotFound(r) }
}

impl From<InternalError> for ApiError {
    fn from(r: InternalError) -> Self { Self::InternalError(r) }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self { Self::InternalError(err.into()) }
}
