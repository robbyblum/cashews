use std::time::Duration;

use chron_base::normalize_location;
use chron_db::models::EntityKind;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::{
    models::MmolbTeam,
    workers::{IntervalWorker, WorkerContext},
};

#[derive(Serialize)]
struct PlacesAutocompleteRequest {
    input: String,
    #[serde(rename = "includedPrimaryTypes")]
    included_primary_types: String,
    #[serde(rename = "sessionToken")]
    session_token: String,
}

#[derive(Deserialize, Debug)]
struct PlacesAutocompleteResponse {
    #[serde(default)]
    suggestions: Vec<PlacesSuggestion>,
}

#[derive(Deserialize, Debug)]
struct PlacesSuggestion {
    #[serde(rename = "placePrediction")]
    place_prediction: PlacePrediction,
}

#[derive(Deserialize, Debug)]
struct PlacePrediction {
    #[serde(rename = "placeId")]
    place_id: String,
    text: PlacePredictionText,
}

#[derive(Deserialize, Debug)]
struct PlacePredictionText {
    text: String,
}

async fn places_autocomplete(
    client: &reqwest::Client,
    location: &str,
    session: Uuid,
    api_key: &str,
) -> anyhow::Result<PlacesAutocompleteResponse> {
    let body = PlacesAutocompleteRequest {
        input: location.to_string(),
        included_primary_types: "(cities)".to_string(),
        session_token: session.to_string(),
    };

    let resp = client
        .post("https://places.googleapis.com/v1/places:autocomplete")
        .json(&body)
        .header("X-Goog-Api-Key", api_key)
        .header(
            "X-Goog-FieldMask",
            "suggestions.placePrediction.placeId,suggestions.placePrediction.text.text",
        )
        .send()
        .await?;

    let resp = resp.error_for_status()?;

    let data: PlacesAutocompleteResponse = resp.json().await?;
    Ok(data)
}

async fn place_info(
    client: &reqwest::Client,
    place_id: &str,
    session: Uuid,
    api_key: &str,
) -> anyhow::Result<serde_json::Value> {
    let resp = client
        .get(format!(
            "https://places.googleapis.com/v1/places/{}?sessionToken={}",
            place_id, session
        ))
        .header("X-Goog-Api-Key", api_key)
        // should all be place details essentials sku
        .header("X-Goog-FieldMask", "id,location,formattedAddress,addressComponents,shortFormattedAddress,postalAddress,types")
        .send()
        .await?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Ok(serde_json::Value::Null);
    }

    let resp = resp.error_for_status()?;
    Ok(resp.json().await?)
}

fn none_if_null(value: serde_json::Value) -> Option<serde_json::Value> {
    if value.is_null() { None } else { Some(value) }
}

// note: not multithread-safe! run sequentially!
async fn search_place_inner(
    ctx: &WorkerContext,
    location: &str,
) -> anyhow::Result<Option<serde_json::Value>> {
    let normalized = normalize_location(location);

    let place_data = sqlx::query_scalar("select data from locations where loc = $1")
        .bind(&normalized)
        .fetch_optional(&ctx.db.pool)
        .await?;

    if let Some(place_data) = place_data {
        return Ok(none_if_null(place_data));
    }

    let Some(ref api_key) = ctx.config.maps_api_key else {
        return Err(anyhow::anyhow!("missing maps api key"));
    };

    info!("missing location data for {}, querying...", normalized);

    // ok, don't have data, query it
    let client = reqwest::ClientBuilder::new().build()?;
    let session = Uuid::new_v4();
    let suggestions = places_autocomplete(&client, location, session, api_key).await?;

    let data = if let Some(best_suggestion) = suggestions.suggestions.first() {
        let place_id = &best_suggestion.place_prediction.place_id;

        if normalize_location(&best_suggestion.place_prediction.text.text) != normalized {
            warn!(
                "queried location name didn't match: requested {}, got {}",
                normalized, best_suggestion.place_prediction.text.text
            );
        }

        let place_info = place_info(&client, place_id, session, api_key).await?;
        place_info
    } else {
        warn!("no locations found for query {}", location);
        serde_json::Value::Null
    };

    sqlx::query("insert into locations (loc, data) values ($1, $2) on conflict (loc) do nothing")
        .bind(&normalized)
        .bind(&data)
        .execute(&ctx.db.pool)
        .await?;

    Ok(none_if_null(data))
}

pub struct LookupMapLocations;
impl IntervalWorker for LookupMapLocations {
    fn interval() -> tokio::time::Interval {
        tokio::time::interval(Duration::from_secs(60))
    }

    async fn tick(&mut self, ctx: &mut super::WorkerContext) -> anyhow::Result<()> {
        let teams = ctx.db.get_all_latest(EntityKind::Team).await?;
        for team in teams {
            let team = team.parse::<MmolbTeam>()?;
            search_place_inner(ctx, &team.full_location).await?;
        }

        Ok(())
    }
}
