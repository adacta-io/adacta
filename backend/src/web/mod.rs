use anyhow::Result;

use crate::config::Web as Config;

use crate::auth::Authenticator;
use crate::index::Index;
use crate::juicer::Juicer;
use crate::suggester::Suggester;
use crate::repository::Repository;

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
    let config = rocket::config::ConfigBuilder::new(rocket::config::Environment::active()?)
        .address(config.address)
        .port(config.port)
        .finalize()?;

    Ok(rocket::custom(config)
        .attach(api::Authorization {})
        .manage(auth)
        .manage(repository)
        .manage(index)
        .manage(juicer)
        .manage(suggester)
        .mount("/api", api::routes())
        .mount("/", frontend::Frontend {}))
}
