extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use reqwest::{Client, Method, StatusCode};
use reqwest::header::{Accept, Authorization, UserAgent};

use std::collections::BTreeMap;
use std::io::Read;

const API_URL: &str = "https://api.rocketleaguestats.com/v1";

#[derive(Debug)]
pub enum Error {
    RateLimited,
    ResponseCode(ResponseCode),
    ReqwestError(reqwest::Error),
    SerdeJson(serde_json::error::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

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

/// A client for the RocketLeagueStats api.
pub struct RlStats {
    client: Client,
    api_key: String,
}

impl RlStats {
    pub fn new<K>(api_key: K) -> Result<Self, Error>
        where K: Into<String>
    {
        Ok(Self { 
            client: Client::new()?,
            api_key: api_key.into(),
        })
    }

    pub fn get_platforms(&self) -> Result<Vec<Platform>, Error> {
        self.get("/data/platforms")
    }

    pub fn get_seasons(&self) -> Result<Vec<Season>, Error> {
        self.get("/data/seasons")
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, Error> {
        self.get("/data/playlists")
    }

    pub fn get_tiers(&self) -> Result<Vec<Tier>, Error> {
        self.get("/data/tiers")
    }

    pub fn get_player(&self, unique_id: &str, platform_id: i32) -> Result<Player, Error> {
        self.get(&format!("/player?unique_id={}&platform_id={}", unique_id, platform_id))
    }

    /// Searches rocketleaguestats' player database, not Rocket League's.
    pub fn search_players(&self, display_name: &str, page: u32) -> Result<SearchResponse, Error> {
        self.get(&format!("/search/players?display_name={}&page={}", display_name, page))
    }

    /// Retrieve more player data faster than you would otherwise be able to.
    /// 
    /// The max batch size is 10. Players that are not found will simply be
    /// excluded from the result.
    pub fn batch_players(&self, players: Vec<BatchPlayer>) -> Result<Vec<Player>, Error> {
        self.request("/player/batch", reqwest::Method::Post, &players)
    }

    pub fn get_ranked_leaderboard(&self, playlist_id: i32) -> Result<Vec<Player>, Error> {
        self.get(&format!("/leaderboard/ranked?playlist_id={}", playlist_id))
    }

    pub fn get_stat_leaderboard(&self, ty: &str) -> Result<Vec<Player>, Error> {
        self.get(&format!("/leaderboard/stat?type={}", ty))
    }

    fn get<T>(&self, endpoint: &str) -> Result<T, Error>
        where T: serde::de::DeserializeOwned
    {
        self.request(endpoint, Method::Get, ())
    }

    fn request<T, J>(&self, endpoint: &str, method: reqwest::Method, j: J) -> Result<T, Error> 
        where T: serde::de::DeserializeOwned,
              J: serde::Serialize
    {
        let user_agent = format!("{} (v {})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        self.client.request(method, &format!("{}{}", API_URL, endpoint))?
            .header(Authorization(self.api_key.clone()))
            .header(Accept::json())
            .header(UserAgent::new(user_agent))
            .json(&j)?
            .send()
            .map_err(Error::ReqwestError)
            .and_then(|mut resp| match resp.status() {
                StatusCode::TooManyRequests => Err(Error::RateLimited),
                _ => {
                    let mut content = String::new();
                    resp.read_to_string(&mut content).unwrap();

                    serde_json::from_str::<T>(&content)
                        .or_else(|_| serde_json::from_str::<ResponseCode>(&content)
                            .map_err(Error::SerdeJson)
                            .and_then(|r| Err(Error::ResponseCode(r))))
                }
            })
    }
}
