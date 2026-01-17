# Bostad Scout AI - System Initialization

## Project Overview
Build a Rust-based housing search system that scrapes Booli and uses AI (Google Gemini) to match apartments based on a single shared natural language description. The system presents all matching properties daily via a web dashboard where you and your partner can swipe yes/no on suggested properties. Mutual "yes" swipes are saved to a favorites list.

## Technical Stack
- **Language**: Rust
- **Web Scraping**: reqwest crate for HTTP, prepare architecture for future anti-scraping measures
- **AI/LLM**: Google Gemini API for natural language processing and matching
- **Data Storage**: 
  - Scraped data: JSON format (serde_json)
  - Historical searches: Track what properties have been evaluated
  - Future: GraphRAG implementation for enhanced semantic search
- **Web Interface**: Web dashboard (Axum/Actix-web backend, simple frontend for swipe interface)
- **Scheduling**: Manual execution initially, architecture should support daily cron jobs later

## Core Features - Phase 1

### 1. Booli Scraper Module
```
Responsibilities:
- Fetch current listings from Booli once daily
- Parse and normalize property data
- Store in JSON format with metadata (fetch_date, source, etc)
- Maintain search history (what properties have been evaluated)
- Handle duplicates (track by Booli ID)
```

### 2. AI Matching Engine
```
Responsibilities:
- Single shared natural language query for both users
  Example: "Ljus 2:a eller 3:a i Vasastan eller Kungsholmen, max 6 miljoner, 
           nära tunnelbana, balkong önskvärt, renoverat kök, minst 55 kvm"
  
- Use Gemini API to:
  * Parse and understand all criteria from natural language (including price!)
  * Match against ALL scraped properties from today
  * Score properties based on how well they match criteria
  * Return ALL properties that match SOME criteria (not just perfect matches)
  * Generate explanation for why each property matches/doesn't match fully
  
- Hybrid approach:
  * Price and other hard constraints embedded in natural language
  * AI determines which properties are "close enough" to show
  * No separate traditional filters
```

### 3. Web Dashboard
```
Responsibilities:
- Display all daily recommended properties
- Tinder-style swipe interface (yes/no for each user)
- Show both users' individual swipes on each property
- Display AI reasoning/match score for each recommendation
- Track mutual "yes" swipes → save to "Favorites List"
- Show favorites list with ability to view/manage
- Option to review why properties matched or didn't match criteria
```

### 4. Data Models
```rust
// Core structures to implement:

Property {
  id: String,              // Booli's unique ID
  source: Source,          // Booli
  location: Location,
  address: String,
  price: i64,
  rooms: f32,
  sqm: i32,
  description: String,
  features: Vec<String>,   // balkong, hiss, etc.
  images: Vec<String>,
  url: String,
  scraped_at: DateTime,
  raw_data: serde_json::Value, // Full JSON for future reprocessing
}

SharedPreference {
  query: String,           // Natural language description
  created_at: DateTime,
  updated_at: DateTime,
  embedding: Option<Vec<f32>>, // For future GraphRAG
}

PropertyMatch {
  property_id: String,
  match_score: f32,        // 0.0 - 1.0
  reasoning: String,       // Why it matches (or partially matches)
  criteria_met: Vec<String>,
  criteria_missed: Vec<String>,
  evaluated_at: DateTime,
}

UserSwipe {
  user_id: String,         // "user1" or "user2"
  property_id: String,
  decision: SwipeDecision, // Yes, No
  swiped_at: DateTime,
}

FavoriteProperty {
  property_id: String,
  added_at: DateTime,      // When both swiped yes
  notes: Option<String>,   // Optional notes about the property
}

DailyRun {
  run_id: String,
  executed_at: DateTime,
  properties_scraped: i32,
  properties_matched: i32,
  new_properties: Vec<String>, // IDs of new properties never seen before
}

SwipeDecision: enum {
  Yes,
  No,
}
```

## Architecture Considerations

### Modular Design
```
src/
├── scrapers/
│   ├── mod.rs
│   ├── booli.rs
│   ├── types.rs         // Common types for scraped data
│   └── traits.rs        // ScraperTrait for future sources
├── ai/
│   ├── mod.rs
│   ├── gemini.rs        // Gemini API client
│   ├── matcher.rs       // Matching logic
│   └── prompt.rs        // Prompt engineering for matching
├── storage/
│   ├── mod.rs
│   ├── json.rs          // JSON file operations
│   ├── history.rs       // Search history tracking
│   └── graphrag.rs      // Stub for future GraphRAG
├── web/
│   ├── mod.rs
│   ├── server.rs        // Web server setup
│   ├── routes.rs        // Route definitions
│   ├── handlers.rs      // Request handlers
│   └── static/          // Frontend assets
├── models/
│   └── mod.rs           // Shared data models
└── main.rs
```

### Daily Workflow
```
1. Run scraper → fetch all new Booli listings
2. Load shared preference (natural language query)
3. Send all properties + query to Gemini
4. Gemini returns matches with scores + reasoning
5. Store results in JSON
6. Web dashboard displays properties for swiping
7. Track swipes per user
8. When both swipe yes → add to favorites
```

### Future-Proofing
- Design scraper trait to easily add new sources (Hemnet, Blocket, etc)
- Store raw JSON to enable reprocessing with improved AI later
- Keep AI matching logic separate from scraping logic
- Prepare data structures for GraphRAG embeddings
- Architecture supports adding cron scheduling without major refactor
- Consider embedding generation for properties (future semantic search)

## Key Implementation Details

### 1. Natural Language Query Processing
```rust
// Example prompt to Gemini:
"Given this apartment search criteria:
'{user_query}'

And this property:
{property_json}

Evaluate how well this property matches the criteria.
Return:
- match_score (0.0-1.0): How well it matches overall
- criteria_met: List of criteria this property satisfies
- criteria_missed: List of criteria this property doesn't satisfy
- reasoning: Brief explanation of the match

Be lenient - include properties that match SOME criteria, not just perfect matches."
```

### 2. Swipe Logic
```rust
// When processing swipes:
- Store each user's swipe independently
- Check if both users have swiped on a property
- If both swiped "Yes" → move to Favorites
- Allow viewing previous swipes (read-only)
```

### 3. Data Refresh Strategy
```rust
// Daily execution:
1. Scrape Booli (all relevant listings)
2. Compare with previous day's data (using Booli ID)
3. Identify:
   - New properties (never seen before)
   - Updated properties (price changes, etc)
   - Removed properties (no longer available)
4. Only run AI matching on new/updated properties
5. Archive old runs (keep history)
```

### 4. Favorites Management
```rust
// Favorites list features:
- View all mutual "yes" swipes
- Sort by date added, price, match score
- Add notes to favorites
- Mark as "contacted" or "viewed"
- Remove from favorites if no longer interested
```

## File Storage Structure
```
data/
├── properties/
│   ├── 2025-01-17.json          // Daily scraped properties
│   ├── 2025-01-18.json
│   └── ...
├── matches/
│   ├── 2025-01-17-matches.json  // Daily AI match results
│   └── ...
├── swipes/
│   ├── user1.json               // User 1's swipes
│   └── user2.json               // User 2's swipes
├── favorites.json               // Mutual yes swipes
├── preferences.json             // Shared query + metadata
└── history.json                 // Run history and stats
```

## Development Phases

**Phase 1**: Basic Booli scraper + JSON storage
- Implement scraper
- Store properties in JSON
- Manual CLI to view scraped data

**Phase 2**: Gemini integration + matching logic
- Integrate Gemini API
- Implement matching algorithm
- CLI to test matches with sample queries

**Phase 3**: Simple web dashboard (read-only)
- Display properties
- Show match scores and reasoning
- Basic UI/UX

**Phase 4**: Swipe functionality
- Implement swipe interface for both users
- Track swipes per user
- Implement favorites logic

**Phase 5**: Polish + enhancements
- Improve UI/UX
- Add filters/sorting on dashboard
- Notes and management for favorites
- Statistics and insights

**Phase 6**: GraphRAG + scheduling
- Generate embeddings for properties
- Implement GraphRAG for semantic search
- Add cron scheduling for daily runs
- Email/push notifications for new matches

## Open Questions for Implementation

1. **Gemini API Setup**: Which Gemini model? (gemini-pro recommended for balance of cost/performance)
2. **Rate limiting**: Batch properties or send individually to Gemini?
3. **Frontend framework**: Plain HTML/JS, React, Svelte? (Keep it simple initially)
4. **Authentication**: Password protect dashboard or keep it local-only?
5. **Property images**: Download and store locally or link to Booli?
