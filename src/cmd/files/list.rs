use clap::ArgMatches;
use reqwest;
use reqwest::blocking::Client;
use serde_json;
use url::form_urlencoded;

use crate::goauth::USER_AGENT;
use crate::config::Config;
use crate::error::Error;

const FILES_LIST_API: &'static str = "https://www.googleapis.com/drive/v3/files";

fn puts_files(files: &Vec<serde_json::Value>) {
    for file in files {
        let id = file["id"].as_str();
        let name = file["name"].as_str();
        let parents = if let Some(parents) = file["parents"].as_array() {
            parents.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(",")
        } else {
            String::from("-")
        };
        // let mime_type = file["mimeType"].as_str();
        // let description = file["description"].as_str();
        // let web_content_link = file["webContentLink"].as_str();
        // let web_view_link = file["webViewLink"].as_str();
        // let created_time = file["createdTime"].as_str();
        let modified_time = file["modifiedTime"].as_str();

        println!("{}\t{}\t{}\t{}",
            id.unwrap_or(""),
            parents,
            name.unwrap_or(""),
            modified_time.unwrap_or(""));
    }
}

fn list_all_files(client: &Client, access_token: &str, query: &Option<&str>, page_token: &Option<&str>) -> Result<(), Error> {
    let mut params = form_urlencoded::Serializer::new(String::new());
    params.append_pair("fields", "files,nextPageToken");
    params.append_pair("includeItemsFromAllDrives", "true");
    params.append_pair("supportsAllDrives", "true");
    if let Some(q) = query {
        params.append_pair("q", q);
    }
    if let Some(token) = page_token {
        params.append_pair("pageToken", token);
    }
    let url = format!("{}?{}", FILES_LIST_API, params.finish());

    let res = client.get(&url)
        .bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    if !res.status().is_success() {
        return Err(Error::from(res));
    }

    let response: serde_json::Value = serde_json::from_reader(res)?;
    match response["files"].as_array() {
        Some(files) => puts_files(files),
        _ => {
            if page_token.is_none() {
                println!("no files were found");
            }
        }
    }

    if let Some(next_token) = response["nextPageToken"].as_str() {
        return list_all_files(client, access_token, query, &Some(next_token));
    }

    Ok(())
}

pub fn execute_files_list(args: &ArgMatches) -> Result<(), Error> {
    let config = Config::load("default")?;
    let client = Client::new();

    list_all_files(&client, &config.access_token, &args.value_of("query"), &None)
}
