use std::env;
use std::fs;
use std::io::Write;
use std::path;

use serde_json;
use serde_json::json;

use crate::error::Error;

#[derive(Debug)]
pub struct Config {
    pub access_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

fn home_dir() -> Result<path::PathBuf, env::VarError> {
    match env::var("GDRIVE_HOME") {
        Ok(path) => return Ok(path::PathBuf::from(path)),
        Err(_) => (),
    };

    let home = env::var("HOME")?;
    let mut homepath = path::PathBuf::from(home);
    homepath.push(".gdrive");
    return Ok(homepath);
}

impl Config {
    pub fn load(profile: &str) -> Result<Config, Error> {
        let gdrive_dir = home_dir()?;
        let filepath = gdrive_dir.as_path().join(profile);
        let file = fs::File::open(filepath)?;

        let token_json: serde_json::Value = serde_json::from_reader(file)?;
        let access_token = token_json["access_token"].as_str().map(String::from);
        let client_id = token_json["client_id"].as_str().map(String::from);
        let client_secret = token_json["client_secret"].as_str().map(String::from);
        let expires_in = token_json["expires_in"].as_u64();
        let refresh_token = token_json["refresh_token"].as_str().map(String::from);

        return Ok(Config {
            access_token: access_token.ok_or(Error::ConfigError(String::from("access_token")))?,
            client_id: client_id.ok_or(Error::ConfigError(String::from("client_id")))?,
            client_secret: client_secret.ok_or(Error::ConfigError(String::from("client_secret")))?,
            expires_in: expires_in.ok_or(Error::ConfigError(String::from("expires_in")))?,
            refresh_token: refresh_token.ok_or(Error::ConfigError(String::from("refresh_token")))?,
        });
    }

    pub fn save(&self, profile: &str) -> Result<(), Error> {
        let cfg = json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "access_token": self.access_token,
            "expires_in": self.expires_in,
            "refresh_token": self.refresh_token,
        });

        let gdrive_dir = home_dir()?;
        fs::create_dir_all(gdrive_dir.as_path())?;

        let filepath = gdrive_dir.as_path().join(profile);
        let mut file = fs::File::create(filepath)?;

        file.write_all(cfg.to_string().as_bytes())?;

        return Ok(());
    }
}
