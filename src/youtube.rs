use crate::youtube_v3_types as yt;
use async_google_apis_common as common;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub async fn get_subscriptions(
    config_path: &PathBuf,
    https: common::TlsClient,
) -> Vec<YoutubeSubscription> {
    let secret =
        yup_oauth2::read_application_secret(Path::new(config_path).join("client_secret.json"))
            .await
            .expect("unable to load secret");

    let auth =
        InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk("tokencache.json")
            .hyper_client(https.clone())
            .build()
            .await
            .expect("InstalledFlowAuthenticator failed to build");

    let scopes = &["https://www.googleapis.com/auth/youtube.readonly"];

    let mut subscriptions = yt::SubscriptionsService::new(https.clone(), Arc::new(auth));
    subscriptions.set_scopes(scopes);

    let mut params = yt::SubscriptionsListParams::default();
    params.mine = Some(true);
    params.max_results = Some(50);
    params.order = Some(yt::SubscriptionsListOrder::Alphabetical);
    params.part = "snippet,contentDetails".into();

    return fetch_all_subscriptions(&subscriptions).await;
}

pub async fn fetch_all_subscriptions(
    service: &yt::SubscriptionsService,
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

    return subscriptions;
}

async fn fetch_subscriptions_page(
    service: &yt::SubscriptionsService,
    page: &Option<String>,
) -> SubscriptionPageResponse {
    let mut params = yt::SubscriptionsListParams::default();
    params.mine = Some(true);
    params.max_results = Some(50);
    params.order = Some(yt::SubscriptionsListOrder::Alphabetical);
    params.part = "snippet,contentDetails".into();
    params.page_token = page.clone();
    let response = service
        .list(&params)
        .await
        .expect("Failed to fetch subscriptions");
    let subscriptions: Vec<YoutubeSubscription> = response
        .items
        .expect("No response")
        .into_iter()
        .filter_map(|sub| YoutubeSubscription::from_response(&sub))
        .collect();
    //.map(|sub| YoutubeSubscription { title: sub.snippet?.title?, channel: sub.snippet?.resource_id?.channel_id? }).collect();
    SubscriptionPageResponse {
        subs: subscriptions,
        next_page_token: response.next_page_token,
        total: response.page_info.unwrap().total_results.unwrap(),
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
    fn channel_feed_url(&self) -> String {
        format!(
            "https://www.youtube.com/feeds/videos.xml?channel_id={}",
            self.channel
        )
    }

    fn channel_url(&self) -> String {
        format!("https://www.youtube.com/channel/{}", self.channel)
    }

    fn from_response(response: &yt::Subscription) -> Option<YoutubeSubscription> {
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
