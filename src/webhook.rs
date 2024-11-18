use std::env::{self, VarError};

use serde::Serialize;

#[derive(Serialize)]
pub struct Webhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    embeds: Vec<EmbedData>,
}

#[derive(Serialize)]
pub struct EmbedData {
    title: String,
    description: String,
    url: String,
}

impl EmbedData {
    pub fn from_url<T, U, D>(title: T, url: U, desciption: D) -> Self
    where
        T : Into<String>,
        U : Into<String>,
        D : Into<String>,
    {
        EmbedData {
            title: title.into(),
            url: url.into(),
            description: desciption.into()
        }
    }
}

#[derive(Default)]
pub struct WebhookBuilder {
    content: Option<String>,
    embeds: Vec<EmbedData>,
}

impl WebhookBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content<S : Into<String>>(mut self, content: S) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn embeds(mut self, embeds: Vec<EmbedData>) -> Self {
        self.embeds = embeds;
        self
    }

    pub fn build(self) -> Webhook {
        Webhook {
            content: self.content,
            embeds: self.embeds,
        }
    }
}

pub struct WebhookClient {
    req_client: reqwest::Client,
    url: String,
}

impl WebhookClient {
    pub fn new<I : Into<String>, T : Into<String>>(id: I, token: T) -> Self {
        let url = format!("https://discord.com/api/webhooks/{}/{}", id.into(), token.into());

        Self {
            req_client: reqwest::Client::new(),
            url: url,
        }
    }

    pub fn from_env() -> Result<Self, VarError> {
        let id = env::var("WEBHOOK_ID")?;
        let token = env::var("WEBHOOK_TOKEN")?;

        Ok(Self::new(id, token))
    }

    pub async fn send(&self, webhook: &Webhook) -> Result<(), reqwest::Error> {
        self.req_client.post(&self.url)
            .json(webhook)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .and(Ok(()))
    }
}
