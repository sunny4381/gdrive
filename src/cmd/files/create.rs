use std::collections::HashMap;

use std::fs::File;
use std::path::Path;

use clap::ArgMatches;
use mime_guess;
use reqwest;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use url::form_urlencoded;

use crate::goauth::USER_AGENT;
use crate::config::Config;
use crate::error::Error;

const FILES_FILE_API: &'static str = "https://www.googleapis.com/upload/drive/v3/files";
const FILES_METADATA_API: &'static str = "https://www.googleapis.com/drive/v3/files";

#[derive(Debug)]
struct FileMetadata {
    pub id: String,
    pub web_content_link: Option<String>,
    pub web_view_link: Option<String>,
}

impl FileMetadata {
    pub fn from(response: &serde_json::Value) -> Result<FileMetadata, Error> {
        Ok(FileMetadata {
            id: response["id"].as_str().map(|v| String::from(v)).ok_or_else(|| Error::MalformedResponse(String::from("id is not obtained")))?,
            web_content_link: response["webContentLink"].as_str().map(|v| String::from(v)),
            web_view_link: response["webViewLink"].as_str().map(|v| String::from(v)),
        })
    }
}

fn guess_mime_type(path: &Path) -> Result<mime_guess::Mime, Error> {
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(ext).first_or_octet_stream();

    Ok(mime)
}

fn upload_file(client: &Client, access_token: &str, file_id: &str, filepath: &Path, mime_type: &str) -> Result<FileMetadata, Error> {
    let file: File = File::open(&filepath)?;

    let req = client.patch(format!("{}/{}?uploadType=media", FILES_FILE_API, file_id).as_str());
    let res = req.bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(CONTENT_TYPE, mime_type)
        .body(file)
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    FileMetadata::from(&response)
}

fn create_or_update_file(client: &Client, access_token: &str, args: &ArgMatches, mime_type: &str) -> Result<FileMetadata, Error> {
    let mut map = HashMap::new();
    if let Some(name) = args.value_of("name") {
        map.insert(String::from("name"), json!(name));
    }
    if let Some(description) = args.value_of("description") {
        map.insert(String::from("description"), json!(description));
    }
    if let Some(parent) = args.value_of("parent") {
        let parents: Vec<&str> = parent.split("/").map(|x| x.trim()).collect();
        map.insert(String::from("parents"), json!(parents));
    }
    map.insert(String::from("mimeType"), json!(mime_type));

    let json = json!(map);

    let request_json: String = json.to_string();

    let req = client.post(FILES_METADATA_API);
    let res = req.bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_json)
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    FileMetadata::from(&response)
}

fn get_file(client: &Client, access_token: &str, file_id: &str) -> Result<FileMetadata, Error> {
    let mut params = form_urlencoded::Serializer::new(String::new());
    params.append_pair("fields", "id,webContentLink,webViewLink");

    let url = format!("{}/{}?{}", FILES_METADATA_API, file_id, params.finish());

    let res = client.get(&url)
        .bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    FileMetadata::from(&response)
}

pub fn execute_files_create(args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;

    let filepath = match args.value_of("file") {
        Some(file) => Path::new(file),
        _ => panic!("specify file"),
    };
    let _mime_type = guess_mime_type(&filepath)?;
    let mime_type = _mime_type.to_string();

    let client = Client::new();

    let metadata = create_or_update_file(&client, &config.access_token, &args, &mime_type)?;
    upload_file(&client, &config.access_token, &metadata.id, &filepath, &mime_type)?;
    let metadata = get_file(&client, &config.access_token, &metadata.id)?;
    if let Some(url) = metadata.web_content_link.or(metadata.web_view_link) {
        println!("{}", url);
    }

    Ok(())
}

pub fn execute_files_create_folder(args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;
    let client = Client::new();

    let metadata = create_or_update_file(&client, &config.access_token, &args, "application/vnd.google-apps.folder")?;
    println!("Folder ID: {}", metadata.id);

    Ok(())
}
