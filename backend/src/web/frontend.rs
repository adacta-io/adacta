use std::io::Cursor;
use std::path::PathBuf;

use rocket::{Data, Handler, Request, Response, Route};
use rocket::handler::{HandlerFuture, Outcome};
use rocket::http::{ContentType, Method};
use rust_embed::RustEmbed;


#[derive(RustEmbed, Debug, Clone)]
#[folder = "../frontend/dist/adacta"]
pub struct Frontend;

impl Into<Vec<Route>> for Frontend {
    fn into(self) -> Vec<Route> {
        return vec![
            Route::ranked(99, Method::Get, "/", self.clone()),
            Route::ranked(99, Method::Get, "/<path..>", self),
        ];
    }
}

impl Handler for Frontend {
    fn handle<'r>(&self, request: &'r Request<'_>, _data: Data) -> HandlerFuture<'r> {
        return Box::pin(async move {
            // TODO: Im pretty sure this can be more readable...
            let (path, file) = request.get_segments::<PathBuf>(0)
                .map(|path| path.expect("segments"))
                .and_then(|path| Self::get(&path.to_string_lossy()).map(|file| (path, file)))
                .unwrap_or_else(|| (PathBuf::from("index.html"), Self::get("index.html").expect("no index")));

            let mut response = Response::build()
                .sized_body(Cursor::new(file)).await
                .finalize();

            if let Some(content_type) = path.extension()
                .and_then(|ext| ContentType::from_extension(&ext.to_string_lossy())) {
                response.set_header(content_type);
            }

            return Outcome::Success(response);
        });
    }
}