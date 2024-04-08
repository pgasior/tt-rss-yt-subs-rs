use anyhow::{Context, Result};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::path::Path;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const CHANNELS_URL: &str = "https://youtube.googleapis.com/youtube/v3/subscriptions";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListSubscriptionResponse {
    items: Vec<Item>,
    page_info: PageInfo,
    next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    total_results: i32,
    results_per_page: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    snippet: Snippet,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snippet {
    title: String,
    resource_id: Resource,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Resource {
    kind: String,
    channel_id: String,
}

pub async fn get_api_key(config_path: &Path) -> Result<String> {
    let secret =
        yup_oauth2::read_application_secret(Path::new(config_path).join("client_secret.json"))
            .await
            .expect("unable to load secret");

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(Path::new(config_path).join("tokencache.json"))
        // .hyper_client(https.clone())
        .build()
        .await?;
    let token = auth
        .token(&["https://www.googleapis.com/auth/youtube.readonly"])
        .await?;
    Ok(token.token().context("Failed to get token")?.to_string())
}

pub async fn get_subscribed_channels(
    config_path: &Path,
    progress: &ProgressBar,
) -> Result<Vec<YoutubeSubscription>> {
    let mut all_items: Vec<YoutubeSubscription> = Vec::new();
    let mut next_page_token: Option<String> = None;

    let api_key = get_api_key(config_path).await?;

    loop {
        let client = reqwest::Client::new();
        let url = match next_page_token {
            Some(ref token) => format!(
                "{}?part=snippet&mine=true&maxResults=50&order=alphabetical&pageToken={}",
                CHANNELS_URL, token
            ),
            None => format!(
                "{}?part=snippet&mine=true&order=alphabetical&maxResults=50",
                CHANNELS_URL
            ),
        };
        let res = client
            .get(&url)
            .bearer_auth(&api_key)
            // .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;
        let status = res.status();
        let body = res.text().await?;
        // println!("{}", body);
        if !status.is_success() {
            println!("Failed to fetch subscriptions: {}", body);
            return Err(anyhow::anyhow!("Failed to fetch subscriptions"));
        }
        let response: ListSubscriptionResponse = serde_json::from_str(&body)?;

        all_items.extend(response.items.iter().map(|item| YoutubeSubscription {
            title: item.snippet.title.clone(),
            channel: item.snippet.resource_id.channel_id.clone(),
        }));
        
        progress.set_length(response.page_info.total_results as u64);
        progress.set_position(all_items.len() as u64);

        next_page_token = response.next_page_token;

        if next_page_token.is_none() {
            break;
        }
    }

    Ok(all_items)
}

#[derive(Debug)]
pub struct YoutubeSubscription {
    pub title: String,
    pub channel: String,
}

impl YoutubeSubscription {
    pub fn channel_feed_url(&self) -> String {
        format!(
            "https://www.youtube.com/feeds/videos.xml?channel_id={}",
            self.channel
        )
    }

    pub fn channel_url(&self) -> String {
        format!("https://www.youtube.com/channel/{}", self.channel)
    }

    // fn from_response(response: &Subscription) -> Option<YoutubeSubscription> {
    //     let title = response.snippet.as_ref()?.title.as_ref()?;
    //     let channel = response
    //         .snippet
    //         .as_ref()?
    //         .resource_id
    //         .as_ref()?
    //         .channel_id
    //         .as_ref()?;

    //     Some(YoutubeSubscription {
    //         title: title.to_string(),
    //         channel: channel.to_string(),
    //     })
    // }
}
