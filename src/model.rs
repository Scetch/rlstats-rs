use std::collections::BTreeMap;

/// A possible json error.
/// 
/// See http://documentation.rocketleaguestats.com/#response-codes
#[derive(Debug, Deserialize)]
pub struct ResponseCode {
    pub code: i32,
    pub message: String,
}

/// A platform that RocketLeague supports.
#[derive(Clone, Debug, Deserialize)]
pub struct Platform {
    /// Some known IDs:
    /// 
    /// * 1 is Steam
    /// * 2 is PS4
    /// * 3 is XboxOne
    pub id: i32,
    pub name: String,
}

/// A RocketLeague season.
#[derive(Debug, Deserialize)]
pub struct Season {
    /// 1, 2, 3, 4 and onwards.
    #[serde(rename = "seasonId")]
    pub season_id: i64,
    /// This is a unix timestamp.
    #[serde(rename = "startedOn")]
    pub started_on: i64,
    /// This is a unix timestamp.
    /// 
    /// This field will be `None` if the season has not yet ended.
    #[serde(rename = "endedOn")]
    pub ended_on: Option<i64>,
}

/// Population of a `Playlist`.
#[derive(Debug, Deserialize)]
pub struct Population {
    /// Number of players currently playing the playlist.
    pub players: i32,
    /// This is a unix timestamp.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// A RocketLeague playlist.
#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub id: i32,
    /// See the `Platform` struct.
    #[serde(rename = "platformId")]
    pub platform_id: i32,
    pub name: String,
    pub population: Population,
}

/// A RocketLeague ranked tier.
#[derive(Debug, Deserialize)]
pub struct Tier {
    /// Increments for every tier and sub-tier.
    /// 
    /// Example:
    /// 
    /// ```no-run
    /// [
    ///     Tier {
    ///         id: 0,
    ///         name: "Unranked"
    ///     },
    ///     Tier {
    ///         id: 1,
    ///         name: "Bronze I"
    ///     },
    ///     Tier {
    ///         id: 2,
    ///         name: "Bronze II"
    ///     },
    ///     Tier {
    ///         id: 3,
    ///         name: "Bronze III"
    ///     },
    ///     Tier {
    ///         id: 4,
    ///         name: "Silver I"
    ///     },
    ///     Tier {
    ///         id: 5,
    ///         name: "Silver II"
    ///     },
    ///     Tier {
    ///         id: 6,
    ///         name: "Silver III"
    ///     },
    ///     Tier {
    ///         id: 7,
    ///         name: "Gold I"
    ///     },
    ///     Tier {
    ///         id: 8,
    ///         name: "Gold II"
    ///     },
    ///     ...
    /// ]
    /// ```
    #[serde(rename = "tierId")]
    pub id: i32,
    #[serde(rename = "tierName")]
    pub name: String,
}

/// Stats about a player.
#[derive(Debug, Deserialize)]
pub struct Stats {
    pub wins: i32,
    pub goals: i32,
    pub mvps: i32,
    pub saves: i32,
    pub shots: i32,
    pub assists: i32,
}

/// Information about a season.
#[derive(Debug, Deserialize)]
pub struct RankedData {
    #[serde(rename = "rankPoints")]
    pub rank_points: Option<i32>,
    #[serde(rename = "matchesPlayed")]
    pub matches_played: Option<i32>,
    pub tier: Option<i32>,
    pub division: Option<i32>,
}

/// A RocketLeague player.
/// 
/// Players will only exist if they have scored at least one goal.
#[derive(Debug, Deserialize)]
pub struct Player {
    /// Steam 64 ID / PSN Username / Xbox XUID
    #[serde(rename = "uniqueId")]
    pub unique_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub platform: Platform,
    pub avatar: Option<String>,
    #[serde(rename = "profileUrl")]
    pub profile_url: String,
    #[serde(rename = "signatureUrl")]
    pub signature_url: String,
    pub stats: Stats,
    #[serde(rename = "rankedSeasons")]
    pub ranked_seasons: BTreeMap<String, BTreeMap<String, RankedData>>,
    /// This is a unix timestamp.
    #[serde(rename = "lastRequested")]
    pub last_requested: i64,
    /// This is a unix timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// This is a unix timestamp.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// This is a unix timestamp.
    #[serde(rename = "nextUpdateAt")]
    pub next_update_at: i64,
}

/// A search response.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub page: Option<i32>,
    pub results: i32,
    /// The total number of players that match the search.
    #[serde(rename = "totalResults")]
    pub total_results: i32,
    #[serde(rename = "maxResultsPerPage")]
    pub max_results_per_page: i32,
    pub data: Vec<Player>,
}

/// A batch player.
#[derive(Debug, Serialize)]
pub struct BatchPlayer {
    #[serde(rename = "uniqueId")]
    pub id: String,
    #[serde(rename = "platformId")]
    pub platform_id: i32,
}