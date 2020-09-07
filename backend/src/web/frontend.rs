use std::io::Cursor;
use std::path::PathBuf;

use rocket::handler::{Handler, Outcome};
use rocket::http::{ContentType, Method};
use rocket::{Data, Request, Response, Route};
use rust_embed::RustEmbed;

use async_trait::async_trait;

#[derive(RustEmbed, Debug, Clone)]
#[folder = "../frontend/dist/adacta"]
pub struct Frontend;

impl Into<Vec<Route>> for Frontend {
    fn into(self) -> Vec<Route> {
        vec![
            Route::ranked(99, Method::Get, "/", self.clone()),
            Route::ranked(99, Method::Get, "/<path..>", self),
        ]
    }
}

#[async_trait]
impl Handler for Frontend {
    async fn handle<'r, 's: 'r>(&'s self, request: &'r Request<'_>, _data: Data) -> Outcome<'r> {
        // TODO: Im pretty sure this can be more readable...
        let (path, file) = request.get_segments::<PathBuf>(0)
                                  .map(|path| path.expect("segments"))
                                  .and_then(|path| Self::get(&path.to_string_lossy()).map(|file| (path, file)))
                                  .unwrap_or_else(|| (PathBuf::from("index.html"), Self::get("index.html").expect("no index")));

        let mut response = Response::build()
            .sized_body(file.len(), Cursor::new(file))
            .finalize();

        if let Some(content_type) = path.extension()
                                        .and_then(|ext| ContentType::from_extension(&ext.to_string_lossy())) {
            response.set_header(content_type);
        }

        Outcome::Success(response)
    }
}
