mod models;
mod scrapers;

use scrapers::BooliBrowserScraper;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🏠 Housing Scout - Booli Browser Scraper");
    info!("==========================================");
    info!("");

    // Create browser scraper
    let scraper = BooliBrowserScraper::new()?;

    // Run scraper
    info!("Starting browser-based scrape from Booli Södermalm...");
    info!("This will visit each property page for detailed information");
    info!("");
    
    let properties = scraper.scrape_sodermalm()?;

    // Display results
    info!("\n✅ Scraped {} properties\n", properties.len());

    for (i, property) in properties.iter().enumerate() {
        println!("{}. {} ({} kr)", i + 1, property.address, property.price);
        println!("   {} rum, {} kvm", property.rooms, property.sqm);
        if let Some(area) = &property.location.area {
            println!("   Area: {}", area);
        }
        println!("   ID: {}", property.id);
        println!("   Features: {}", property.features.join(", "));
        println!("   URL: {}", property.url);
        println!();
    }

    // Save to main JSON file
    let json = serde_json::to_string_pretty(&properties)?;
    tokio::fs::write("scraped_properties.json", json).await?;
    info!("💾 Saved all properties to scraped_properties.json");

    // Save each property to separate file in raw_scrape/
    tokio::fs::create_dir_all("raw_scrape").await?;
    
    for property in &properties {
        let filename = format!("raw_scrape/{}.json", property.id);
        let prop_json = serde_json::to_string_pretty(&property)?;
        tokio::fs::write(&filename, prop_json).await?;
    }
    
    info!("💾 Saved {} individual property files to raw_scrape/", properties.len());

    Ok(())
}
