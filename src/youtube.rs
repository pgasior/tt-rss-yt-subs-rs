use crate::TlsClient;
use google_youtube3::api::Subscription;
use google_youtube3::YouTube;
use std::path::Path;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub async fn get_subscriptions(config_path: &Path, https: TlsClient) -> Vec<YoutubeSubscription> {
    let secret =
        yup_oauth2::read_application_secret(Path::new(config_path).join("client_secret.json"))
            .await
            .expect("unable to load secret");

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .hyper_client(https.clone())
        .build()
        .await
        .expect("InstalledFlowAuthenticator failed to build");

    let hub = YouTube::new(https, auth);
    return fetch_all_subscriptions(&hub).await;
}

pub async fn fetch_all_subscriptions(
    service: &google_youtube3::YouTube,
) -> Vec<YoutubeSubscription> {
    let mut page_token = None;
    let mut subscriptions: Vec<YoutubeSubscription> = Vec::new();
    loop {
        let mut page = fetch_subscriptions_page(service, &page_token).await;
        page_token = page.next_page_token;
        subscriptions.append(&mut page.subs);
        println!("\rFetching... {} / {}", subscriptions.len(), page.total);
        if page_token.is_none() {
            break;
        }
    }

    subscriptions
}

async fn fetch_subscriptions_page(
    service: &google_youtube3::YouTube,
    page: &Option<String>,
) -> SubscriptionPageResponse {
    let mut call = service
        .subscriptions()
        .list(&vec!["snippet".into(), "contentDetails".into()])
        .mine(true)
        .max_results(50)
        .order("alphabetical");
    if let Some(p) = page {
        call = call.page_token(p);
    }

    let (_, result) = call.doit().await.expect("Failed to fetch subscriptions");

    let subscriptions: Vec<YoutubeSubscription> = result
        .items
        .expect("No response")
        .into_iter()
        .filter_map(|sub| YoutubeSubscription::from_response(&sub))
        .collect();

    SubscriptionPageResponse {
        subs: subscriptions,
        next_page_token: result.next_page_token,
        total: result.page_info.unwrap().total_results.unwrap(),
    }
}

struct SubscriptionPageResponse {
    subs: Vec<YoutubeSubscription>,
    next_page_token: Option<String>,
    total: i32,
}

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

    fn from_response(response: &Subscription) -> Option<YoutubeSubscription> {
        let title = response.snippet.as_ref()?.title.as_ref()?;
        let channel = response
            .snippet
            .as_ref()?
            .resource_id
            .as_ref()?
            .channel_id
            .as_ref()?;

        Some(YoutubeSubscription {
            title: title.to_string(),
            channel: channel.to_string(),
        })
    }
}
