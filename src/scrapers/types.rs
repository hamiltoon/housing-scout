use serde::{Deserialize, Serialize};

/// Search parameters for property scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    /// City or area to search in
    pub location: String,
    /// Minimum price (SEK)
    pub min_price: Option<i64>,
    /// Maximum price (SEK)
    pub max_price: Option<i64>,
    /// Minimum number of rooms
    pub min_rooms: Option<f32>,
    /// Maximum number of rooms
    pub max_rooms: Option<f32>,
    /// Minimum size in square meters
    pub min_sqm: Option<i32>,
    /// Maximum size in square meters
    pub max_sqm: Option<i32>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            location: "Stockholm".to_string(),
            min_price: None,
            max_price: None,
            min_rooms: None,
            max_rooms: None,
            min_sqm: None,
            max_sqm: None,
        }
    }
}
