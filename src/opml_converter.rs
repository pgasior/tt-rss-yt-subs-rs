use crate::youtube::YoutubeSubscription;
use anyhow::{Context, Result};
use quick_xml::se::to_string;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename = "opml")]
struct Opml {
    #[serde(rename = "@version")]
    version: String,
    body: Body,
}

#[derive(Debug, Serialize)]
struct Body {
    #[serde(rename = "outline", default)]
    category: Category,
}

#[derive(Debug, Serialize)]
struct Category {
    #[serde(rename = "@text")]
    text: String,
    #[serde(rename = "@title")]
    title: String,
    outline: Vec<Outline>,
}

#[derive(Debug, Serialize)]
struct Outline {
    #[serde(rename = "@text")]
    text: String,
    #[serde(rename = "@title")]
    title: String,
    #[serde(rename = "@type", default)]
    outline_type: String,
    #[serde(rename = "@xmlUrl", default)]
    xml_url: String,
    #[serde(rename = "@htmlUrl", default)]
    html_url: String,
}

pub fn convert_to_opml_string(
    category_name: &str,
    subscriptions: &[YoutubeSubscription],
) -> Result<String> {
    let opml = Opml {
        version: "1.1".to_string(),
        body: Body {
            category: Category {
                text: category_name.to_string(),
                title: category_name.to_string(),
                outline: subscriptions
                    .iter()
                    .map(|s| Outline {
                        text: s.title.clone(),
                        title: s.title.clone(),
                        outline_type: "rss".to_string(),
                        xml_url: s.channel_feed_url(),
                        html_url: s.channel_url(),
                    })
                    .collect(),
            },
        },
    };
    to_string(&opml).context("Unable to convert to OPML string")
}
