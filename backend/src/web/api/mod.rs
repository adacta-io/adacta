use rocket::{Route, routes};

pub(super) use auth::Authorization;
pub(self) use auth::Token;
pub(self) use error::{ApiError, InternalError};

pub(self) mod auth;
pub(self) mod error;

mod upload;
mod inbox;
mod archive;
mod labels;

pub fn routes() -> Vec<Route> {
    routes![
        auth::login,
        upload::upload_pdf,
        inbox::list,
        inbox::get,
        inbox::delete,
        inbox::archive,
        archive::bundle,
        archive::fragment,
        archive::search,
        labels::list,
    ]
}
