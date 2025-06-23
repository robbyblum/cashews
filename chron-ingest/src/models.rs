#![allow(unused)]

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MmolbState {
    #[serde(rename = "GreaterLeagues")]
    pub greater_leagues: Vec<String>,

    #[serde(rename = "LesserLeagues")]
    pub lesser_leagues: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MmolbLeague {
    #[serde(rename = "Teams")]
    pub teams: Vec<String>,

    #[serde(rename = "SuperstarTeam")]
    pub superstar_team: Option<String>,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "LeagueType")]
    pub league_type: String,

    #[serde(rename = "Color")]
    pub color: String,

    #[serde(rename = "Emoji")]
    pub emoji: String,
}

#[derive(Deserialize, Debug)]
pub struct MmolbTeam {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "League")]
    pub league: String,

    #[serde(rename = "Location")]
    pub location: String,

    #[serde(rename = "FullLocation")]
    pub full_location: String,

    #[serde(rename = "Color")]
    pub color: String,

    #[serde(rename = "Emoji")]
    pub emoji: String,

    #[serde(rename = "Abbreviation")]
    pub abbreviation: String,

    #[serde(rename = "Players")]
    pub players: Vec<MmolbTeamPlayer>,
}

#[derive(Deserialize, Debug)]
pub struct MmolbTeamPlayer {
    #[serde(rename = "PlayerID")]
    pub player_id: String,

    #[serde(rename = "FirstName")]
    pub first_name: String,

    #[serde(rename = "LastName")]
    pub last_name: String,

    #[serde(rename = "PositionType")]
    // "Batter" or "Pitcher" - todo: enum?
    pub position_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MmolbPlayer {
    #[serde(rename = "FirstName")]
    pub first_name: String,

    #[serde(rename = "LastName")]
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MmolbGame {
    #[serde(rename = "Season")]
    pub season: i32,
    #[serde(rename = "Day")]
    pub day: i32,

    #[serde(rename = "AwayTeamID")]
    pub away_team_id: String,
    #[serde(rename = "HomeTeamID")]
    pub home_team_id: String,

    #[serde(rename = "State")]
    pub state: String,

    #[serde(rename = "Stats")]
    pub stats: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    #[serde(rename = "EventLog")]
    pub event_log: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MmolbGameEvent {
    pub event: String, // event type
    pub pitcher: Option<String>,
    pub batter: Option<String>,
    pub inning: i32,
    pub inning_side: i32,
}

#[derive(Debug, Deserialize)]
pub struct MmolbTime {
    pub season_day: i32,
    pub season_number: i32,
    pub season_status: String,
}

#[derive(Debug, Deserialize)]
pub struct MmolbGameByTeam {
    pub game_id: String,
    pub status: String,

    #[serde(rename = "AwayTeamID")]
    pub away_team_id: String,

    #[serde(rename = "HomeTeamID")]
    pub home_team_id: String,
}
