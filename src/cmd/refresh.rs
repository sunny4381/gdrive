use clap::ArgMatches;

use crate::goauth;
use crate::config::Config;
use crate::error::Error;

pub fn execute_refresh(_args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;
    let token = goauth::refresh_token(&config.client_id, &config.client_secret, &config.refresh_token)?;

    let new_config = Config {
        client_id: config.client_id,
        client_secret: config.client_secret,
        access_token: token.access_token,
        expires_in: token.expires_in,
        refresh_token: config.refresh_token,
    };
    new_config.save("default")?;

    return Ok(());
}
