use anyhow::Error;
use rocket::{Route, routes};
use rocket::http::RawStr;
use rocket::request::FromParam;

use crate::model::DocId;

pub(self) mod auth;
pub(self) mod error;

mod inbox;
mod repo;
mod search;
mod upload;
mod labels;

pub(self) use error::{ApiError, InternalError};
pub(self) use auth::Token;

pub(super) use auth::Authorization;

pub fn routes() -> Vec<Route> {
    routes![
        auth::auth,
        repo::bundle,
        repo::fragment,
        search::search,
        inbox::inbox,
        upload::upload_pdf,
        labels::labels,
        labels::guess,
    ]
}

impl FromParam<'_> for DocId {
    type Error = Error;

    fn from_param(param: &'_ RawStr) -> Result<Self, Self::Error> {
        Ok(param.parse()?)
    }
}

