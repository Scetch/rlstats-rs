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
#[derive(Debug, Deserialize)]
pub struct ResponseCode {
    pub code: i32,
    pub message: String,
}

/// A platform that RocketLeague supports.
#[derive(Clone, Debug, Deserialize)]
pub struct Platform {
    pub id: i32,
    pub name: String,
}

/// A RocketLeague season.
#[derive(Debug, Deserialize)]
pub struct Season {
    #[serde(rename = "seasonId")]
    pub season_id: i64,
    #[serde(rename = "startedOn")]
    pub started_on: i64,
    #[serde(rename = "endedOn")]
    pub ended_on: Option<i64>,
}

/// Population of a `Playlist`.
#[derive(Debug, Deserialize)]
pub struct Population {
    pub players: i32,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// A RocketLeague playlist.
#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub id: i32,
    #[serde(rename = "platformId")]
    pub platform_id: i32,
    pub name: String,
    pub population: Population,
}

/// A RocketLeague ranked tier.
#[derive(Debug, Deserialize)]
pub struct Tier {
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
#[derive(Debug, Deserialize)]
pub struct Player {
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
    #[serde(rename = "lastRequested")]
    pub last_requested: i64,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    #[serde(rename = "nextUpdateAt")]
    pub next_update_at: i64,
}

/// A search response.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub page: Option<i32>,
    pub results: i32,
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

    pub fn search_players(&self, display_name: &str, page: u32) -> Result<SearchResponse, Error> {
        self.get(&format!("/search/players?display_name={}&page={}", display_name, page))
    }

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
