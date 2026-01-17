use crate::models::{Location, Property, Source};
use anyhow::{Context, Result};
use chrono::Utc;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};
use serde_json::json;
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Browser-based scraper for Booli using headless Chrome
pub struct BooliBrowserScraper {
    browser: Browser,
}

impl BooliBrowserScraper {
    /// Create a new browser-based scraper
    pub fn new() -> Result<Self> {
        info!("Launching headless Chrome...");
        
        let options = LaunchOptions::default_builder()
            .headless(true)
            .build()
            .context("Failed to build launch options")?;
        
        let browser = Browser::new(options)
            .context("Failed to launch Chrome browser")?;
        
        Ok(Self { browser })
    }

    /// Scrape all properties from Södermalm listing page
    pub fn scrape_sodermalm(&self) -> Result<Vec<Property>> {
        let url = "https://www.booli.se/sok/till-salu?areaIds=115341";
        
        info!("Opening Södermalm search page...");
        let tab = self.browser.new_tab()?;
        
        // Navigate to search page
        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;
        
        // Wait longer for page to fully load
        info!("Waiting for page to fully load...");
        thread::sleep(Duration::from_secs(8));
        
        // Accept cookies if present
        let _ = tab.evaluate(
            r#"
            const button = document.querySelector('button[id*="accept"], button[id*="godkann"]');
            if (button) button.click();
            "#,
            false,
        );
        
        thread::sleep(Duration::from_secs(2));
        
        // Create debug directory
        std::fs::create_dir_all("debug")?;
        
        // Capture HTML for debugging
        info!("Capturing page HTML for debugging...");
        let html_result = tab.evaluate("document.documentElement.outerHTML", false)?;
        if let Some(html_value) = html_result.value {
            if let Some(html_str) = html_value.as_str() {
                std::fs::write("debug/booli_page.html", html_str)?;
                info!("Saved page HTML to debug/booli_page.html ({} bytes)", html_str.len());
            }
        }
        
        // Capture screenshot
        info!("Capturing screenshot...");
        let screenshot_data = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;
        std::fs::write("debug/booli_screenshot.png", screenshot_data)?;
        info!("Saved screenshot to debug/booli_screenshot.png");
        
        info!("Extracting property data from listing page HTML...");
        
        // Get the HTML content we just captured
        let html_result = tab.evaluate("document.documentElement.outerHTML", false)?;
        let html_str = match html_result.value {
            Some(value) => value.as_str().unwrap_or("").to_string(),
            None => {
                warn!("Could not get HTML from page");
                String::new()
            }
        };
        
        if html_str.is_empty() {
            warn!("HTML is empty");
            return Ok(Vec::new());
        }
        
        // Parse HTML with scraper
        let document = Html::parse_document(&html_str);
        let card_selector = Selector::parse("a.object-card-link").unwrap();
        
        let cards: Vec<_> = document.select(&card_selector).collect();
        info!("Found {} property cards in HTML", cards.len());
        
        let mut properties = Vec::new();
        
        for (idx, element) in cards.iter().enumerate() {
            // Extract data from the card  
            let href = element.value().attr("href").unwrap_or("");
            let aria_label_raw = element.value().attr("aria-label").unwrap_or("");
            
            // Decode HTML entities (&nbsp; -> space)
            let aria_label = aria_label_raw.replace("&nbsp;", " ");
            
            debug!("Processing: {}", aria_label);
            
            // Extract Booli ID from URL
            let booli_id = href.split('/').last().unwrap_or("unknown").to_string();
            
            // Parse aria-label: "2 rum lägenhet på Götgatan 120 Södermalm, Stockholms kommun"
            let mut rooms = 0.0;
            let mut address = String::new();
            let mut area = String::from("Södermalm");
            
            if let Some(rum_match) = aria_label.split("rum").next() {
                if let Some(last_word) = rum_match.trim().split_whitespace().last() {
                    rooms = last_word.replace(",", ".").parse().unwrap_or(0.0);
                }
            }
            
            // Extract address - between "på " and area name
            if let Some(pa_pos) = aria_label.find("på ") {
                let after_pa = &aria_label[pa_pos + 3..];
                // Address is until we hit the area or comma
                if let Some(comma_pos) = after_pa.find(',') {
                    address = after_pa[..comma_pos].trim().to_string();
                    // Extract area from what's before the comma
                    let area_part = &after_pa[..comma_pos];
                    if let Some(last_space) = area_part.rfind(' ') {
                        let potential_area = &area_part[last_space + 1..];
                        if !potential_area.chars().next().unwrap_or('a').is_numeric() {
                            area = potential_area.to_string();
                        }
                    }
                } else {
                    address = after_pa.trim().to_string();
                }
            }
            
            // Get the element's inner HTML to extract other data
            let card_html = element.html();
            
            // Debug: Save first card HTML
            if idx == 0 {
                std::fs::write("debug/first_card.html", &card_html)?;
                info!("Saved first card HTML to debug/first_card.html");
            }
            
            let card_doc = Html::parse_fragment(&card_html);
            
            // Extract price, sqm, and other details from list items
            let li_selector = Selector::parse("li").unwrap();
            
            let mut price: i64 = 0;
            let mut sqm: i32 = 0;
            let mut features = Vec::new();
            let mut monthly_fee = String::new();
            
            for li in element.select(&li_selector) {
                if let Some(aria) = li.value().attr("aria-label") {
                    let aria_decoded = aria.replace("&nbsp;", " ");
                    
                    // Extract sqm from "35,5 kvadratmeter" or similar
                    if aria_decoded.contains("kvadratmeter") {
                        let sqm_str: String = aria_decoded.chars()
                            .filter(|c| c.is_numeric() || *c == ',' || *c == '.')
                            .collect();
                        sqm = sqm_str.replace(",", ".").parse::<f32>().unwrap_or(0.0) as i32;
                    }
                    
                    // Monthly fee
                    if aria_decoded.contains("kr/mån") {
                        monthly_fee = aria_decoded.clone();
                    }
                }
            }
            
            // Extract price from price container
            let price_selector = Selector::parse("span.object-card__price--logo").unwrap();
            if let Some(price_el) = element.select(&price_selector).next() {
                let price_text = price_el.text().collect::<String>();
                // Remove spaces and "kr", parse
                let price_clean: String = price_text.chars()
                    .filter(|c| c.is_numeric())
                    .collect();
                if !price_clean.is_empty() {
                    price = price_clean.parse().unwrap_or(0);
                }
            }
            
            // Extract features from amenities
            let tag_selector = Selector::parse("div.tag").unwrap();
            for tag in element.select(&tag_selector) {
                let feature = tag.text().collect::<String>().trim().to_string();
                if !feature.is_empty() && feature != "Snart till salu" {
                    features.push(feature);
                }
            }
            
            // Only add if we have minimum data
            if !address.is_empty() && (price > 0 || sqm > 0) {
                let property = Property {
                    id: booli_id.clone(),
                    source: Source::Booli,
                    location: Location {
                        city: "Stockholm".to_string(),
                        area: Some(area.clone()),
                        latitude: Some(59.3145),
                        longitude: Some(18.0736),
                    },
                    address: address.clone(),
                    price,
                    rooms,
                    sqm,
                    description: format!("{} rum lägenhet i {}. {} kvm.", rooms, area, sqm),
                    features: features.clone(),
                    images: vec![],
                    url: format!("https://www.booli.se{}", href),
                    scraped_at: Utc::now(),
                    raw_data: json!({
                        "area": area,
                        "scraped_from": "listing_page",
                        "booli_id": booli_id,
                        "aria_label": aria_label,
                        "monthly_fee": monthly_fee
                    }),
                };
                
                properties.push(property);
            } else {
                info!("Skipped property {}: address='{}', price={}, sqm={}", idx, address, price, sqm);
            }
        }
        
        info!("Successfully scraped {} properties from listing page", properties.len());
        
        Ok(properties)
    }
}
