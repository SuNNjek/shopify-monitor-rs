use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

use reqwest::IntoUrl;
use anyhow::Result;

use crate::shopify::{Product, ShopifyClient};

#[derive(Clone)]
pub struct AvailabilityChecker<'a, U = String> {
    last_available_ids: Arc<Mutex<HashSet<i64>>>,
    client: &'a ShopifyClient<U>,
}

impl <'a, U : IntoUrl + Clone> AvailabilityChecker<'a, U> {
    pub fn new(client: &'a ShopifyClient<U>) -> Self {
        AvailabilityChecker {
            last_available_ids: Arc::new(Mutex::new(HashSet::new())),
            client,
        }
    }

    pub async fn check_newly_available(&self) -> Result<Vec<Product>> {
        let mut last_available_ids = self.last_available_ids.lock().await;

        let response = self.client.get_all_products().await?;

        let available_products: Vec<Product> = response.get_available_products()
            .into_iter()
            .cloned()
            .collect();
            
        let newly_available: Vec<Product> = available_products.iter()
            .filter(|product| !last_available_ids.contains(&product.id))
            .cloned()
            .collect();

        println!("{} available total, {} became available since last check", available_products.len(), newly_available.len());
    
        Self::update_last_available(&mut last_available_ids, &available_products);
        Ok(newly_available)
    }

    fn update_last_available(last_available_ids: &mut HashSet<i64>, available_products: &Vec<Product>) {
        last_available_ids.clear();
        
        for product in available_products {
            last_available_ids.insert(product.id);
        }
    }
}