use reqwest;
use serde_derive::Deserialize;
use serde_json;
use url::form_urlencoded;

use crate::error::Error;

pub const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &'static str = "https://www.googleapis.com/oauth2/v4/token";
pub const INFO_URL: &'static str = "https://www.googleapis.com/oauth2/v1/userinfo";
pub const REDIRECT_URI: &'static str = "urn:ietf:wg:oauth:2.0:oob";
pub const USER_AGENT: &'static str = "rust-oauth-test/0.1";

#[derive(Debug, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub email: String,
}

pub fn auth_url(client_id: &str) -> String {
    let auth_params: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", "profile email https://www.googleapis.com/auth/drive.file")
        .finish();
    return format!("{}?{}", AUTH_URL, auth_params);
}

pub fn auth_token(client_id: &str, client_secret: &str, code: &str) -> Result<Token, Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.post(TOKEN_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .form(&[
            ("code", code), ("client_id", client_id), ("client_secret", client_secret),
            ("redirect_uri", REDIRECT_URI), ("grant_type", "authorization_code")
        ])
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res))
    };

    let token_body: serde_json::Value = serde_json::from_reader(res)?;

    return Ok(
        Token {
            access_token: String::from(token_body["access_token"].as_str().unwrap()),
            expires_in: token_body["expires_in"].as_u64().unwrap(),
            token_type: String::from(token_body["token_type"].as_str().unwrap()),
            refresh_token: Some(String::from(token_body["refresh_token"].as_str().unwrap())),
        });
}

pub fn refresh_token(client_id: &str, client_secret: &str, refresh_token: &str) -> Result<Token, Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.post(TOKEN_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .form(&[
            ("client_id", client_id), ("client_secret", client_secret), ("refresh_token", refresh_token),
            ("grant_type", "refresh_token")
        ])
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res))
    };

    let refresh_body: serde_json::Value = serde_json::from_reader(res)?;

    return Ok(
        Token {
            access_token: String::from(refresh_body["access_token"].as_str().unwrap()),
            expires_in: refresh_body["expires_in"].as_u64().unwrap(),
            token_type: String::from(refresh_body["token_type"].as_str().unwrap()),
            refresh_token: None,
        });
}

pub fn user_info(access_token: &str) -> Result<UserInfo, Error> {
    let client = reqwest::blocking::Client::new();
    let info_res = client.get(INFO_URL)
        .bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;
    if !info_res.status().is_success() {
        return Err(Error::from(info_res))
    };

    let info_body: serde_json::Value = serde_json::from_reader(info_res)?;

    return Ok(UserInfo {
        id: String::from(info_body["id"].as_str().unwrap()),
        name: String::from(info_body["name"].as_str().unwrap()),
        given_name: String::from(info_body["given_name"].as_str().unwrap()),
        family_name: String::from(info_body["family_name"].as_str().unwrap()),
        picture: String::from(info_body["picture"].as_str().unwrap()),
        email: String::from(info_body["email"].as_str().unwrap()),
    });
}
