mod cmd;
mod config;
mod error;
mod goauth;

use std::io::{self, Write};

use clap::clap_app;

use cmd::execute;
use error::Error;

fn main() {
    let args = clap_app!(gdrive =>
        (author: "NAKANO Hideo. <pinarello.marvel@gmail.com>")
        (about: "Google Drive Manager")
        (version: "1.0.0")
        (@subcommand init =>
            (about: "initialize environment")
            (@arg client_id: +required "client id")
            (@arg client_secret: +required "client secret")
        )
        (@subcommand refresh =>
            (about: "refresh access token")
        )
        (@subcommand whoami =>
            (about: "print who am I")
        )
        (@subcommand files =>
            (@subcommand list =>
                (about: "show all files")
                (@arg drive_id: --drive_id +takes_value "drive id to show")
                (@arg query: --query +takes_value "query for filtering the file results. see: https://developers.google.com/drive/api/v3/search-files")
            )
            (@subcommand create =>
                (about: "create file")
                (@arg file: +required "file to create")
                (@arg name: --name +takes_value "name of file")
                (@arg description: --description +takes_value "file description")
                (@arg parent: --parent +takes_value "comma separatered parent list")
            )
            (@subcommand create_folder =>
                (about: "create folder")
                (@arg name: --name +takes_value "name of folder")
                (@arg description: --description +takes_value "file description")
                (@arg parent: --parent +takes_value "comma separatered parent list")
            )
        )
    ).get_matches();

    env_logger::init();

    match execute(&args) {
        Ok(_) => (),
        Err(ref e) => abort(e),
    };
}

pub fn abort(e: &Error) {
    writeln!(&mut io::stderr(), "{}", e).unwrap();
    ::std::process::exit(1)
}
