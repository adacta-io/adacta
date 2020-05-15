use anyhow::Result;

use crate::auth::Authenticator;
use crate::index::Index;
use crate::juicer::Juicer;
use crate::pigeonhole::Pigeonhole;
use crate::repo::Repository;

mod api;
mod frontend;

pub async fn serve(
    auth: Authenticator,
    repo: Repository,
    index: Box<dyn Index + Send + Sync>,
    juicer: Box<dyn Juicer + Send + Sync>,
    pigeonhole: Box<dyn Pigeonhole + Send + Sync>,
) -> Result<()>
{
    return Ok(rocket::ignite()
        .attach(api::Authorization {})
        .manage(auth)
        .manage(repo)
        .manage(index)
        .manage(juicer)
        .manage(pigeonhole)
        .mount("/api", api::routes())
        .mount("/", frontend::Frontend {})
        .serve()
        .await?);
}
