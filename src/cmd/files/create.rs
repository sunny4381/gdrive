use std::collections::HashMap;

use std::fs::File;
use std::path::Path;

use clap::ArgMatches;
use mime_guess;
use reqwest;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::goauth::USER_AGENT;
use crate::config::Config;
use crate::error::Error;

const FILES_FILE_API: &'static str = "https://www.googleapis.com/upload/drive/v3/files";
const FILES_METADATA_API: &'static str = "https://www.googleapis.com/drive/v3/files";
// const FILES_UPDATE_METADATA_API: &'static str = "https://content.googleapis.com/drive/v3/files";

// fn create_metadata_part(args: &ArgMatches) -> Result<multipart::Part, Error> {
//     let mut map = HashMap::new();
//     if let Some(name) = args.value_of("name") {
//         map.insert(String::from("name"), json!(name));
//     }
//     if let Some(description) = args.value_of("description") {
//         map.insert(String::from("description"), json!(description));
//     }
//     if let Some(drive_id) = args.value_of("drive_id") {
//         map.insert(String::from("drive_id"), json!(drive_id));
//     }
//     let json = json!(map);

//     let mut part = multipart::Part::text(json.to_string());
//     part = part.mime_str("application/json; charset=UTF-8")?;

//     Ok(part)
// }

// fn create_file_part(args: &ArgMatches) -> Result<multipart::Part, Error> {
//     if let Some(filepath) = args.value_of("file") {
//         let path = Path::new(filepath);
//         Ok(multipart::Part::file(&path)?)
//     } else {
//         panic!("specify file");
//     }
// }

fn guess_mime_type(path: &Path) -> Result<mime_guess::Mime, Error> {
    //let file_name = path.file_name().map(|filename| filename.to_string_lossy().into_owned());
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(ext).first_or_octet_stream();

    Ok(mime)
}

fn upload_file(client: &Client, access_token: &str, file_id: &Option<&str>, filepath: &Path) -> Result<String, Error> {
    let mime_type = guess_mime_type(&filepath)?;
    let file: File = File::open(&filepath)?;

    let req = if let Some(file_id) = file_id {
        client.patch(format!("{}/{}?uploadType=media", FILES_FILE_API, file_id).as_str())
    } else {
        client.post(format!("{}?uploadType=media", FILES_FILE_API).as_str())
    };

    let res = req.bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(CONTENT_TYPE, mime_type.to_string())
        .body(file)
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    response["id"].as_str().map_or_else(
        || Err(Error::MalformedResponse(String::from("id is not obtained"))),
        |x| Ok(String::from(x))
    )
}

fn create_or_update_file(client: &Client, access_token: &str, file_id: &Option<&str>, args: &ArgMatches) -> Result<String, Error> {
    let mut map = HashMap::new();
    if let Some(name) = args.value_of("name") {
        map.insert(String::from("name"), json!(name));
    }
    if let Some(description) = args.value_of("description") {
        map.insert(String::from("description"), json!(description));
    }
    if let Some(drive_id) = args.value_of("drive_id") {
        map.insert(String::from("drive_id"), json!(drive_id));
    }
    if let Some(parent) = args.value_of("parent") {
        let parents: Vec<&str> = parent.split("/").map(|x| x.trim()).collect();
        map.insert(String::from("parents"), json!(parents));
    }
    let json = json!(map);

    let request_json: String = json.to_string();
    println!("request_json={}", request_json);

    let req = if let Some(file_id) = file_id {
        client.patch(format!("{}/{}", FILES_METADATA_API, file_id).as_str())
    } else {
        client.post(FILES_METADATA_API)
    };
    let res = req.bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_json)
        .send()?;
    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    response["id"].as_str().map_or_else(
        || Err(Error::MalformedResponse(String::from("id is not obtained"))),
        |x| Ok(String::from(x))
    )
}

pub fn execute_files_create(args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;
    // let access_token: &str = &config.access_token;

    let filepath = match args.value_of("file") {
        Some(file) => Path::new(file),
        _ => panic!("specify file"),
    };

    // let first_part = create_metadata_part(&args)?;
    // let second_part = create_file_part(&args)?;

    // let form = multipart::Form::new();
    // let form = form.part("metadata", first_part);
    // let form = form.part("file", second_part);
    // let boundary = String::from(form.boundary());

    let client = Client::new();

    // let req = client.post(FILES_CREATE_API)
    //     .bearer_auth(access_token)
    //     .header(reqwest::header::USER_AGENT, USER_AGENT)
    //     .multipart(form)
    //     .header(CONTENT_TYPE, format!("multipart/related; boundary={}", boundary).as_str());

    // let res = req.send()?;
    // if !res.status().is_success() {
    //     return Err(Error::from(res));
    // }

    // let response: serde_json::Value = serde_json::from_reader(res)?;
    // puts_file(&response);
    let file_id = create_or_update_file(&client, &config.access_token, &None, &args)?;
    println!("file_id={}", file_id);

    upload_file(&client, &config.access_token, &Some(&file_id), &filepath)?;

    Ok(())
}
