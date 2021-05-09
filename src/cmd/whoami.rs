use clap::ArgMatches;

use crate::goauth::user_info;
use crate::config::Config;
use crate::error::Error;

pub fn execute_whoami(_args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;
    let info = user_info(&config.access_token)?;

    println!("{}", info.email);

    Ok(())
}
