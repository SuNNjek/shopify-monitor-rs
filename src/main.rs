use std::{collections::HashMap, env, num::ParseIntError};

use anyhow::{anyhow, Result};
use checker::AvailabilityChecker;
use discord::WebhookClient;
use shopify::ShopifyClient;
use tokio::signal::unix::{signal, SignalKind};
use utils::{get_scheduler_from_env, sleep_until_next, webhook_from_product};

pub mod checker;
pub mod discord;
pub mod shopify;
pub mod utils;

async fn run_checker() -> Result<()> {
    let shopify_client = ShopifyClient::from_env()?;
    let discord_client = WebhookClient::from_env()?;
    let checker = AvailabilityChecker::new(&shopify_client);

    let product_ids = env::var("PRODUCT_IDS")?
        .split(",")
        .map(str::trim)
        .map(str::parse::<i64>)
        .collect::<Result<Vec<i64>, ParseIntError>>()?;

    let scheduler = get_scheduler_from_env();

    loop {
        // Get all products that have become available since the last check and collect them into a (ID -> Product) map
        let new_products: HashMap<_, _> = checker
            .check_newly_available()
            .await?
            .into_iter()
            .map(|product| (product.id, product))
            .collect();

        // Retrieve all products from the map that we care about
        let notify_products: Vec<_> = product_ids
            .iter()
            .filter_map(|id| new_products.get(id))
            .collect();

        for product in notify_products {
            let webhook = webhook_from_product(&shopify_client, product)?;
            discord_client.send(&webhook).await?;
        }

        sleep_until_next(&scheduler).await?;
    }
}

async fn wait_for_terminate() -> Result<()> {
    let mut sig = signal(SignalKind::terminate())?;

    sig.recv()
        .await
        .ok_or(anyhow!("failed to listen for SIGTERM"))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    // Wait for either an error from the checker or SIGTERM
    tokio::select! {
        Err(err) = run_checker() => println!("Error while scraping shop: {}", err),
        _ = wait_for_terminate() => println!("Received SIGTERM, quitting..."),
    }

    Ok(())
}
