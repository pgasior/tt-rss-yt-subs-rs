mod youtube;
mod ttrss;
mod opml_converter;

use crate::youtube::get_subscriptions;
use std::path::PathBuf;
use crate::opml_converter::convert_to_opml_string;

pub type TlsClient = hyper::Client<TlsConnector, hyper::Body>;
pub type TlsConnector = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;

#[tokio::main]
async fn main() {
    let https = https_client();

    let subs = get_subscriptions(&get_config_path(), https).await;

    // for s in subs {
    //     println!("{}", s.title)
    // }

    println!("{}", convert_to_opml_string("Youtube subscriptions", &subs));
}

fn https_client() -> TlsClient {
    let conn = hyper_rustls::HttpsConnector::with_native_roots();
    hyper::Client::builder().build(conn)
}

fn get_config_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().expect("Unable to get home dir"));
    path.push(".ytsubs");
    path
}
