use anyhow::Error;
use rocket::{Route, routes, request::FromParam, http::RawStr};

use crate::model::DocId;

mod repo;
mod search;
mod upload;

pub fn routes() -> Vec<Route> {
    return routes![
        repo::bundle,
        repo::fragment,

        search::search,
        search::inbox,

        upload::upload_pdf,
    ];
}

impl FromParam<'_> for DocId {
    type Error = Error;

    fn from_param(param: &'_ RawStr) -> Result<Self, Self::Error> {
        return Ok(param.parse()?);
    }
}
