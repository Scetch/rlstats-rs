use reqwest::{
    blocking::{Client, ClientBuilder},
    header::{self, HeaderMap, HeaderValue},
    Method,
};
use serde::{de::DeserializeOwned, Serialize};

mod model;

pub use model::*;

const API_URL: &str = "https://api.rocketleaguestats.com/v1";

#[derive(Debug)]
pub enum Error {
    Invalid,
    ResponseCode(ResponseCode),
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}

/// A client for the RocketLeagueStats api.
pub struct RlStats(Client);

impl RlStats {
    pub fn new<K>(api_key: K) -> Result<Self, Error>
    where
        K: AsRef<str>,
    {
        let user_agent = format!(
            "{} (v {})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let headers = [
            (header::AUTHORIZATION, api_key.as_ref()),
            (header::ACCEPT, "application/json"),
            (header::USER_AGENT, &user_agent),
        ]
        .iter()
        .cloned()
        .map(|(key, value)| (key, HeaderValue::from_str(value).unwrap()))
        .collect::<HeaderMap>();

        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(RlStats(client))
    }

    pub fn get_platforms(&self) -> Result<Vec<Platform>, Error> {
        self.request("/data/platforms", Method::GET, ())
    }

    pub fn get_seasons(&self) -> Result<Vec<Season>, Error> {
        self.request("/data/seasons", Method::GET, ())
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, Error> {
        self.request("/data/playlists", Method::GET, ())
    }

    pub fn get_tiers(&self) -> Result<Vec<Tier>, Error> {
        self.request("/data/tiers", Method::GET, ())
    }

    pub fn get_player(&self, unique_id: &str, platform_id: i32) -> Result<Player, Error> {
        self.request(
            format!(
                "/player?unique_id={}&platform_id={}",
                unique_id, platform_id
            ),
            Method::GET,
            (),
        )
    }

    /// Searches rocketleaguestats' player database, not Rocket League's.
    pub fn search_players(&self, display_name: &str, page: u32) -> Result<SearchResponse, Error> {
        self.request(
            format!(
                "/search/players?display_name={}&page={}",
                display_name, page
            ),
            Method::GET,
            (),
        )
    }

    /// Retrieve more player data faster than you would otherwise be able to.
    ///
    /// The max batch size is 10. Players that are not found will simply be
    /// excluded from the result.
    pub fn batch_players(&self, players: Vec<BatchPlayer>) -> Result<Vec<Player>, Error> {
        self.request("/player/batch", Method::POST, &players)
    }

    pub fn get_ranked_leaderboard(&self, playlist_id: i32) -> Result<Vec<Player>, Error> {
        self.request(
            format!("/leaderboard/ranked?playlist_id={}", playlist_id),
            Method::GET,
            (),
        )
    }

    pub fn get_stat_leaderboard(&self, ty: &str) -> Result<Vec<Player>, Error> {
        self.request(format!("/leaderboard/stat?type={}", ty), Method::GET, ())
    }

    fn request<E, T, J>(&self, endpoint: E, method: Method, j: J) -> Result<T, Error>
    where
        E: AsRef<str>,
        T: DeserializeOwned,
        J: Serialize,
    {
        let url = format!("{}{}", API_URL, endpoint.as_ref());
        let body = self.0.request(method, &url).json(&j).send()?.text()?;

        match serde_json::from_str::<T>(&body) {
            Ok(r) => Ok(r),
            _ => Err(Error::ResponseCode(serde_json::from_str(&body)?)),
        }
    }
}
