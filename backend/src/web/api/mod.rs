use anyhow::Error;
use rocket::{Route, routes};
use rocket::http::RawStr;
use rocket::request::FromParam;

use crate::model::DocId;

pub(self) mod auth;
pub(self) mod error;

mod upload;
mod inbox;
mod archive;
mod labels;

pub(self) use error::{ApiError, InternalError};
pub(self) use auth::Token;

pub(super) use auth::Authorization;

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

impl FromParam<'_> for DocId {
    type Error = Error;

    fn from_param(param: &'_ RawStr) -> Result<Self, Self::Error> {
        Ok(param.parse()?)
    }
}
