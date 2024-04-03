mod youtube;
mod ttrss;
mod opml_converter;

use hyper::{client::HttpConnector, Client};
use hyper_rustls::HttpsConnector;

use crate::youtube::get_subscriptions;
use std::path::PathBuf;
use crate::opml_converter::convert_to_opml_string;

pub type TlsClient = Client<HttpsConnector<HttpConnector>>;
// pub type TlsConnector = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;

#[tokio::main]
async fn main() {
    let https = https_client();

    let subs = get_subscriptions(&get_config_path(), https).await;

    // for s in subs {
    //     println!("{}", s.title)
    // }

    println!("{}", convert_to_opml_string("Youtube subscriptions", &subs));
}

fn https_client() -> Client<HttpsConnector<HttpConnector>>{
    let conn = hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().expect("no native root CA certificates found").https_or_http().enable_http1().build();
    Client::builder().build(conn)
}

fn get_config_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().expect("Unable to get home dir"));
    path.push(".ytsubs");
    path
}
