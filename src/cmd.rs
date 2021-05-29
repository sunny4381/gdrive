pub(crate) mod init;
pub(crate) mod refresh;
pub(crate) mod whoami;
pub(crate) mod files;

use clap::ArgMatches;

use self::init::execute_init;
use self::refresh::execute_refresh;
use self::whoami::execute_whoami;
use self::files::list::execute_files_list;
use self::files::create::{execute_files_create, execute_files_create_folder};
use crate::error::Error;

pub fn execute(args: &ArgMatches) -> Result<(), Error> {
    if let Some(args) = args.subcommand_matches("init") {
        return execute_init(args);
    } else if let Some(args) = args.subcommand_matches("refresh") {
        return execute_refresh(args);
    } else if let Some(args) = args.subcommand_matches("whoami") {
        return execute_whoami(args);
    } else if let Some(args) = args.subcommand_matches("files") {
        if let Some(args) = args.subcommand_matches("list") {
            return execute_files_list(args);
        } else if let Some(args) = args.subcommand_matches("create") {
            return execute_files_create(args);
        } else if let Some(args) = args.subcommand_matches("create_folder") {
            return execute_files_create_folder(args);
        }
    }

    return Err(Error::UnknownCommandError);
}
