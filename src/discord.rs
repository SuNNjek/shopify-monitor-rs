use std::env;

use anyhow::Result;
use derive_builder::Builder;
use serde::Serialize;

#[derive(Serialize, Builder)]
pub struct Webhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(each(name = "add_embed", into)))]
    embeds: Vec<EmbedData>,
}

impl Webhook {
    pub fn builder() -> WebhookBuilder {
        WebhookBuilder::default()
    }
}

#[derive(Serialize, Clone, Builder)]
pub struct EmbedData {
    #[builder(setter(into))]
    title: String,
    #[builder(setter(into))]
    url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    thumbnail: Option<EmbedThumbnail>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(setter(each(name = "add_field", into)))]
    fields: Vec<EmbedField>,
}

#[derive(Serialize, Clone, Builder)]
pub struct EmbedField {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into))]
    value: String,
    inline: bool,
}

#[derive(Serialize, Clone)]
pub struct EmbedThumbnail {
    url: String,
}

impl EmbedThumbnail {
    pub fn new<S: Into<String>>(url: S) -> EmbedThumbnail {
        EmbedThumbnail { url: url.into() }
    }
}

pub struct WebhookClient {
    req_client: reqwest::Client,
    url: String,
}

impl WebhookClient {
    pub fn new<I: Into<String>, T: Into<String>>(id: I, token: T) -> Self {
        let url = format!(
            "https://discord.com/api/webhooks/{}/{}",
            id.into(),
            token.into()
        );

        Self {
            req_client: reqwest::Client::new(),
            url: url,
        }
    }

    pub fn from_env() -> Result<Self> {
        let id = env::var("WEBHOOK_ID")?;
        let token = env::var("WEBHOOK_TOKEN")?;

        Ok(Self::new(id, token))
    }

    pub async fn send(&self, webhook: &Webhook) -> Result<()> {
        self.req_client
            .post(&self.url)
            .json(webhook)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())?;

        Ok(())
    }
}
