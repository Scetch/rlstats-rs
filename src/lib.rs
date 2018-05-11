extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;

use reqwest::{ ClientBuilder, Method };
use reqwest::header::{ Headers, Accept, Authorization, UserAgent };
use serde::{ Serialize, de::DeserializeOwned };

pub use model::*;

mod model;

const API_URL: &str = "https://api.rocketleaguestats.com/v1";

#[derive(Debug)]
pub enum Error {
    ResponseCode(ResponseCode),
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self { Error::ReqwestError(err) }
}

/// A client for the RocketLeagueStats api.
pub struct RlStats(reqwest::Client);

impl RlStats {
    pub fn new<K>(api_key: K) -> Result<Self, Error>
        where K: Into<String>
    {
        let mut h = Headers::with_capacity(3);
        h.set(Authorization(api_key.into()));
        h.set(Accept::json());
        h.set(UserAgent::new(format!("{} (v {})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));
        Ok(RlStats(ClientBuilder::new().default_headers(h).build()?))
    }

    pub fn get_platforms(&self) -> Result<Vec<Platform>, Error> {
        self.request("/data/platforms", Method::Get, ())
    }

    pub fn get_seasons(&self) -> Result<Vec<Season>, Error> {
        self.request("/data/seasons", Method::Get, ())
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, Error> {
        self.request("/data/playlists", Method::Get, ())
    }

    pub fn get_tiers(&self) -> Result<Vec<Tier>, Error> {
        self.request("/data/tiers", Method::Get, ())
    }

    pub fn get_player(&self, unique_id: &str, platform_id: i32) -> Result<Player, Error> {
        self.request(format!("/player?unique_id={}&platform_id={}", unique_id, platform_id), Method::Get, ())
    }

    /// Searches rocketleaguestats' player database, not Rocket League's.
    pub fn search_players(&self, display_name: &str, page: u32) -> Result<SearchResponse, Error> {
        self.request(format!("/search/players?display_name={}&page={}", display_name, page), Method::Get, ())
    }

    /// Retrieve more player data faster than you would otherwise be able to.
    /// 
    /// The max batch size is 10. Players that are not found will simply be
    /// excluded from the result.
    pub fn batch_players(&self, players: Vec<BatchPlayer>) -> Result<Vec<Player>, Error> {
        self.request("/player/batch", Method::Post, &players)
    }

    pub fn get_ranked_leaderboard(&self, playlist_id: i32) -> Result<Vec<Player>, Error> {
        self.request(format!("/leaderboard/ranked?playlist_id={}", playlist_id), Method::Get, ())
    }

    pub fn get_stat_leaderboard(&self, ty: &str) -> Result<Vec<Player>, Error> {
        self.request(format!("/leaderboard/stat?type={}", ty), Method::Get, ())
    }

    fn request<E, T, J>(&self, endpoint: E, method: Method, j: J) -> Result<T, Error> 
        where E: AsRef<str>,
              T: DeserializeOwned,
              J: Serialize
    {
        let url = format!("{}{}", API_URL, endpoint.as_ref());
        let mut resp = self.0.request(method, &url).json(&j).send()?;
        match resp.json::<T>() {
            Ok(r) => Ok(r),
            _ => Err(Error::ResponseCode(resp.json()?)),
        }
    }
}