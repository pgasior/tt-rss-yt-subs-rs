use anyhow::Result;
use base64::prelude::*;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;

pub async fn send_opml(opml: &str, config: Config) -> Result<()> {
    let ttrss_config = &config.app.ttrss;
    let token = ttrss_login(
        &ttrss_config.username,
        &ttrss_config.password,
        &ttrss_config.url,
    )
    .await?;
    let response = ttrss_import_opml(opml, &token, &ttrss_config.url).await?;
    let added = filter_events(&response.message, &response.added_message);
    let duplicated = filter_events(&response.message, &response.duplicate_message);
    println!("{}", response.message.join("\n"));
    println!("Total: {}", response.message.len() - 2);
    println!("Duplicated: {}", duplicated.len());
    println!("Added: {}\n{}", added.len(), added.join("\n"));
    Ok(())
}

fn filter_events(lines: &[String], starting_message: &str) -> Vec<String> {
    lines
        .iter()
        .filter_map(|s| {
            if s.trim_start().starts_with(starting_message) {
                Some(s.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

async fn ttrss_import_opml(opml: &str, token: &str, url: &str) -> Result<ImportOpml> {
    let client = reqwest::Client::new();
    let api_url = Url::parse(url)?.join("api/")?;
    let encoded_opml = BASE64_STANDARD.encode(opml);
    let api_data = json!({"op": "importOPML", "opml": encoded_opml, "sid": token});
    let res = client
        .post(api_url)
        .timeout(std::time::Duration::from_secs(60))
        .json(&api_data)
        .send()
        .await?;
    let body = res.text().await?;
    let response: ResponseResult = serde_json::from_str(&body)?;
    if response.status != 0 {
        return Err(anyhow::anyhow!(
            "Failed to import opml:\n{}",
            serde_json::to_string_pretty(&response.content).unwrap()
        ));
    }
    if let Content::ImportOpml(import_opml) = response.content {
        Ok(import_opml)
    } else {
        Err(anyhow::anyhow!(
            "Failed to import opml: {}",
            serde_json::to_string_pretty(&response.content).unwrap()
        ))
    }
}

async fn ttrss_login(username: &str, password: &str, url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let api_url = Url::parse(url)?.join("api/")?;
    let api_data = json!({"op": "login", "user": username, "password": password});
    let res = client.post(api_url).json(&api_data).send().await?;
    let body = res.text().await?;
    let response: ResponseResult = serde_json::from_str(&body)?;
    if response.status != 0 {
        return Err(anyhow::anyhow!(
            "Failed to login: {}",
            serde_json::to_string_pretty(&response.content).unwrap()
        ));
    }
    if let Content::Login(login) = response.content {
        Ok(login.session_id)
    } else {
        Err(anyhow::anyhow!(
            "Failed to login: {}",
            serde_json::to_string_pretty(&response.content).unwrap()
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseResult {
    seq: i32,
    status: i32,
    content: Content,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Content {
    Login(Login),
    ImportOpml(ImportOpml),
    Error(Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub session_id: String,
    pub api_level: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportOpml {
    pub message: Vec<String>,
    pub duplicate_message: String,
    pub added_message: String,
}
