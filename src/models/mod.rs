use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Source of the property listing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Source {
    Booli,
}

/// Location information for a property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub city: String,
    pub area: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Core property data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub source: Source,
    pub location: Location,
    pub address: String,
    pub price: i64,
    pub rooms: f32,
    pub sqm: i32,
    pub description: String,
    pub features: Vec<String>,
    pub images: Vec<String>,
    pub url: String,
    pub scraped_at: DateTime<Utc>,
    pub raw_data: serde_json::Value,
}

