use std::{env, time::Duration};

use anyhow::Result;
use chrono::Local;
use croner::Cron;
use reqwest::IntoUrl;

use crate::{
    discord::{EmbedDataBuilder, EmbedFieldBuilder, EmbedThumbnail, Webhook, WebhookBuilder},
    shopify::{Product, ShopifyClient},
};

pub fn get_scheduler_from_env() -> Cron {
    // Try to parse the schedule given by the environment variable
    let result: Result<Cron> = (|| {
        let schedule = env::var("SCHEDULE")?;
        Ok(Cron::new(&schedule).parse()?)
    })();

    // Otherwise just run hourly
    result.unwrap_or_else(|_| Cron::new("@hourly").parse().unwrap())
}

pub async fn sleep_until_next(cron: &Cron) -> Result<()> {
    // Determine when the next check should happen and sleep until then
    let now: chrono::DateTime<Local> = Local::now();
    let next_run = cron.find_next_occurrence(&now, false)?;

    let ms_to_wait = (next_run - now).num_milliseconds();

    tokio::time::sleep(Duration::from_millis(ms_to_wait.try_into()?)).await;
    Ok(())
}

pub fn webhook_from_product<U: IntoUrl + Clone>(
    client: &ShopifyClient<U>,
    product: &Product,
) -> Result<Webhook> {
    let mut hook_builder = WebhookBuilder::default();

    hook_builder.add_embed({
        let mut embed_builder = EmbedDataBuilder::default();
        embed_builder.title(product.title.clone());
        embed_builder.url(client.get_product_link(product)?);

        if let Some(image) = product.images.first() {
            embed_builder.thumbnail(EmbedThumbnail::new(image.src.clone()));
        }

        for variant in &product.variants {
            let mut field_builder = EmbedFieldBuilder::default();
            field_builder.name(variant.title.clone());
            field_builder.inline(true);

            field_builder.value(if variant.available {
                "In stock"
            } else {
                "Not in stock"
            });

            embed_builder.add_field(field_builder.build()?);
        }

        embed_builder.build()
    }?);

    Ok(hook_builder.build()?)
}
