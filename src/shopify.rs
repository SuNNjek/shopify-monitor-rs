use std::env;

use anyhow::Result;
use reqwest::{header::HeaderMap, IntoUrl};
use serde::Deserialize;

pub struct ShopifyClient<U> {
    req_client: reqwest::Client,
    url: U,
}

impl<U: IntoUrl + Clone> ShopifyClient<U> {
    fn get_default_headers() -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("pragma", "no-cache".parse().unwrap());
        map.insert("cache-control", "no-cache".parse().unwrap());
        map.insert("upgrade-insecure-requests", "1".parse().unwrap());
        map.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36".parse().unwrap());
        map.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".parse().unwrap());
        map.insert("sec-fetch-site", "none".parse().unwrap());
        map.insert("sec-fetch-mode", "navigate".parse().unwrap());
        map.insert("sec-fetch-user", "?1".parse().unwrap());
        map.insert("sec-fetch-dest", "document".parse().unwrap());
        map.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());

        map
    }

    pub fn new(url: U) -> Result<Self> {
        let req_client = reqwest::Client::builder()
            .default_headers(Self::get_default_headers())
            .build()?;

        Ok(Self { req_client, url })
    }

    pub async fn get_all_products(&self) -> Result<ProductResponse> {
        let mut url = self.url.clone().into_url()?;
        url.set_path("/products.json");

        let response = self.req_client.get(url).send().await?.json().await?;

        Ok(response)
    }

    pub fn get_product_link(&self, product: &Product) -> Result<String> {
        let mut url = self.url.clone().into_url()?;
        url.set_path(format!("/products/{}", product.handle).as_str());

        Ok(url.into())
    }
}

impl ShopifyClient<String> {
    pub fn from_env() -> Result<Self> {
        let store_url = env::var("STORE_URL")?;
        let client = ShopifyClient::new(store_url)?;

        Ok(client)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProductVariant {
    pub id: i64,
    pub title: String,
    pub option1: Option<String>,
    pub option2: Option<String>,
    pub option3: Option<String>,
    pub available: bool,
    pub price: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProductImage {
    pub src: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub title: String,
    pub handle: String,
    pub variants: Vec<ProductVariant>,
    pub images: Vec<ProductImage>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProductResponse {
    products: Vec<Product>,
}

impl Product {
    pub fn any_variant_available(&self) -> bool {
        self.variants.iter().any(|variant| variant.available)
    }
}

impl ProductResponse {
    pub fn get_product(&self, product_id: i64) -> Option<&Product> {
        self.products
            .iter()
            .find(|product| product.id == product_id)
    }

    pub fn is_product_available(&self, product_id: i64) -> bool {
        let product = self.get_product(product_id);

        match product {
            None => false,
            Some(product) => product.any_variant_available(),
        }
    }

    pub fn get_available_products(&self) -> Vec<&Product> {
        self.products
            .iter()
            .filter(|&product| product.any_variant_available())
            .collect()
    }
}
