use std::env;
use std::io;
use std::io::Read;
use std::fmt;

use reqwest;
use serde_json;

#[derive(Debug)]
pub enum Error {
    EnvError(env::VarError),
    IoError(io::Error),
    ReqwestError(reqwest::Error),
    HttpError(reqwest::StatusCode, String),
    SerdeError(serde_json::error::Error),
    ConfigError(String),
    MalformedResponse(String),
    UnknownCommandError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EnvError(ref err) => write!(f, "Env error: {}", err),
            Error::IoError(ref err) => write!(f, "IO error: {}", err),
            Error::ReqwestError(ref err) => write!(f, "Reqwest error: {}", err),
            Error::HttpError(ref status, ref msg) => write!(f, "HTTP error: {}\n{}", status, msg),
            Error::SerdeError(ref err) => write!(f, "Serde error: {}", err),
            Error::ConfigError(ref msg) => write!(f, "Config error: {}", msg),
            Error::MalformedResponse(ref msg) => write!(f, "Malformed Response error: {}", msg),
            Error::UnknownCommandError => write!(f, "Unknown Command"),
        }
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Error {
        Error::EnvError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<reqwest::blocking::Response> for Error {
    fn from(mut res: reqwest::blocking::Response) -> Error {
        let mut body = String::new();
        let result = res.read_to_string(&mut body);
        if result.is_ok() {
            Error::HttpError(res.status(), body)
        } else {
            Error::from(result.unwrap_err())
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::SerdeError(err)
    }
}
