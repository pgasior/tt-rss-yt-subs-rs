mod opml_converter;
mod ttrss;
mod youtube;
use indicatif::{ProgressBar, ProgressStyle};

use crate::opml_converter::convert_to_opml_string;
use crate::youtube::get_subscribed_channels;
use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}/{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap(),
    );
    let subs = get_subscribed_channels(&get_config_path(), &pb).await?;

    // for s in subs {
    //     println!("{:?}", s);
    // }

    println!("{}", convert_to_opml_string("Youtube subscriptions", &subs));
    Ok(())
}

fn get_config_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().expect("Unable to get home dir"));
    path.push(".ytsubs");
    path
}
