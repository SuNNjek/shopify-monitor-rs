use std::{env, error::Error, fmt::Display};

use shopify::ShopifyClient;
use webhook::{EmbedData, WebhookBuilder, WebhookClient};

pub mod shopify;
pub mod webhook;

#[derive(Debug)]
enum MonitorError {
    NoProductIds,
    ProductNotFound(i64)
}

impl Error for MonitorError { }
impl Display for MonitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoProductIds => write!(f, "no product IDs given"),
            Self::ProductNotFound(id) => write!(f, "product with ID {} not found", id),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let shopify_client = ShopifyClient::from_env()?;
    let response = shopify_client.get_all_products().await?;

    let product_id = env::var("PRODUCT_IDS")?
        .split(",")
        .filter_map(|s| s.parse::<i64>().ok())
        .nth(0).ok_or(MonitorError::NoProductIds)?;

    let product = response.get_product(product_id)
        .ok_or(MonitorError::ProductNotFound(product_id))?;

    let webhook =
        if product.any_variant_available() {
            let product_link = shopify_client.get_product_link(product)?;

            WebhookBuilder::new()
                .content(format!("\"{}\" is available", product.title))
                .embeds(vec![EmbedData::from_url(&product.title, product_link, "Check it out")])
        } else {
            WebhookBuilder::new()
                .content(format!("\"{}\" is not available", product.title))
        }
        .build();

    let webhook_client = WebhookClient::from_env()?;
    webhook_client.send(&webhook).await?;

    Ok(())
}
