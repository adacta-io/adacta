use anyhow::Result;

use crate::auth::Authenticator;
use crate::config::Web as Config;
use crate::index::Index;
use crate::juicer::Juicer;
use crate::repository::Repository;
use crate::suggester::Suggester;

mod api;
mod frontend;

#[cfg(test)]
mod test;

pub fn server(config: Config,
              auth: Authenticator,
              repository: Repository,
              index: Box<dyn Index + Send + Sync>,
              juicer: Box<dyn Juicer + Send + Sync>,
              suggester: Box<dyn Suggester + Send + Sync>) -> Result<rocket::Rocket> {
    let figment = rocket::figment::Figment::from(rocket::Config::default())
        .merge(("address", config.address))
        .merge(("port", config.port));

    Ok(rocket::custom(figment)
        .attach(api::Authorization {})
        .manage(auth)
        .manage(repository)
        .manage(index)
        .manage(juicer)
        .manage(suggester)
        .mount("/api", api::routes())
        .mount("/", frontend::Frontend {}))
}
