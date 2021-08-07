mod youtube;
mod youtube_v3_types;

use async_google_apis_common as common;
use crate::youtube::get_subscriptions;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let https = https_client();

    let subs = get_subscriptions(&get_config_path(), https).await;

    for s in subs {
        println!("{}", s.title)
    }
}

fn https_client() -> common::TlsClient {
    let conn = hyper_rustls::HttpsConnector::with_native_roots();
    let cl = hyper::Client::builder().build(conn);
    cl
}

fn get_config_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().expect("Unable to get home dir"));
    path.push(".ytsubs");
    path
}
