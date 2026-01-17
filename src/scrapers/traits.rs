use crate::models::Property;
use anyhow::Result;
use async_trait::async_trait;

/// Common trait for all property scrapers
/// This allows easy addition of new sources (Hemnet, Blocket, etc) in the future
#[async_trait]
pub trait ScraperTrait: Send + Sync {
    /// Scrape properties from the source
    async fn scrape(&self) -> Result<Vec<Property>>;
    
    /// Get the name of the scraper source
    fn source_name(&self) -> &'static str;
}
