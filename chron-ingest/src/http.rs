use std::{sync::Arc, time::Duration};

use chron_db::models::{EntityKind, NewObject};
use reqwest::{Client, ClientBuilder, IntoUrl, StatusCode, Url};
use serde::de::DeserializeOwned;
use time::OffsetDateTime;
use tokio::sync::Semaphore;
use tracing::{info, warn};

#[derive(Clone)]
pub struct DataClient {
    client: Client,
    semaphore: Arc<Semaphore>
    // cached_responses: Arc<DashMap<String, ClientResponse>>,
}

#[derive(Debug, Clone)]
pub struct ClientResponse {
    pub _url: Url,
    pub timestamp_before: OffsetDateTime,
    pub timestamp_after: OffsetDateTime,
    // pub etag: Option<String>,
    // pub content_type: Option<String>,
    // pub last_modified: Option<String>,
    pub data: Vec<u8>,
    pub _status_code: StatusCode,
    pub _was_cached: bool,
}

impl ClientResponse {
    pub fn parse<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_slice(&self.data)?)
    }

    pub fn to_chron(&self, kind: EntityKind, entity_id: &str) -> anyhow::Result<NewObject> {
        let parsed = serde_json::from_slice(&self.data)?;

        Ok(NewObject {
            data: parsed,
            kind,
            entity_id: entity_id.to_string(),
            request_time: self.request_time(),
            timestamp: self.timestamp(),
        })
    }

    // pub fn to_asset_object(&self) -> anyhow::Result<NewObject> {
    //     let asset = Asset::new(
    //         self.url.clone(),
    //         self.last_modified.clone(),
    //         self.content_type.clone(),
    //         &self.data,
    //     );
    //     let value = serde_json::to_value(&asset)?;

    //     Ok(NewObject {
    //         kind: EntityKind::Asset,
    //         entity_id: asset.id,
    //         data: value,
    //         timestamp: self.timestamp(),
    //         request_time: self.request_time(),
    //     })
    // }

    pub fn request_time(&self) -> time::Duration {
        self.timestamp_after - self.timestamp_before
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp_before
    }
}

impl DataClient {
    pub fn new() -> anyhow::Result<DataClient> {
        let client = ClientBuilder::new()
            .deflate(true)
            .zstd(true)
            .brotli(true)
            .gzip(true)
            .build()?;

        let semaphore = Arc::new(Semaphore::new(20));

        Ok(DataClient { client, semaphore })
    }

    pub async fn try_fetch(
        &self,
        orig_url: impl IntoUrl,
    ) -> anyhow::Result<Option<ClientResponse>> {
        let res = self.fetch(orig_url).await;

        // if this is specifically a not found error, return None instead
        // todo: can we make this cleaner?
        if let Err(e) = &res {
            if let Some(e) = e.downcast_ref::<reqwest::Error>() {
                match e.status() {
                    Some(StatusCode::NOT_FOUND) => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }

        Ok(Some(res?))
    }

    pub async fn fetch(&self, orig_url: impl IntoUrl) -> anyhow::Result<ClientResponse> {
        let _permit = self.semaphore.acquire().await?;

        let request = self.client.get(orig_url);
        // if let Some(cached_etag) = self
        //     .cached_responses
        //     .get(orig_url)
        //     .and_then(|x| x.etag.clone())
        // {
        //     request = request.header(header::IF_NONE_MATCH, cached_etag);
        // }

        let timestamp_before = OffsetDateTime::now_utc();
        let response = request.send().await?;
        let timestamp_after = OffsetDateTime::now_utc();
        info!(
            "{} {} ({}s)",
            response.status(),
            response.url(),
            (timestamp_after - timestamp_before).as_seconds_f64()
        );

        let url = response.url().clone();
        // let last_modified = response
        //     .headers()
        //     .get(header::LAST_MODIFIED)
        //     .and_then(|x| x.to_str().ok())
        //     .map(|x| x.to_owned());
        // let content_type = response
        //     .headers()
        //     .get(header::CONTENT_TYPE)
        //     .and_then(|x| x.to_str().ok())
        //     .map(|x| x.to_owned());
        // let etag = response
        //     .headers()
        //     .get(header::ETAG)
        //     .and_then(|x| x.to_str().ok())
        //     .map(|x| x.to_owned());
        let status_code = response.status();
        if status_code == StatusCode::BAD_GATEWAY {
            // if we get a 502 from the server, sleep for a second
            // because we're still within the semaphore, this basically functions as a light "circuit breaker"
            // and will slow down at least one "slot" of the available permits
            warn!("received 502 response, sleeping for a bit");
            let _cb_permit = self.semaphore.acquire_many(self.semaphore.available_permits() as u32).await?;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        let response = response.error_for_status()?;

        // if response.status() == StatusCode::NOT_MODIFIED {
        //     if let Some(resp) = self.cached_responses.get(orig_url) {
        //         if resp.etag == etag {
        //             let mut cached_resp = resp.clone();
        //             cached_resp.status_code = response.status();
        //             cached_resp.was_cached = true;
        //             cached_resp.timestamp_before = timestamp_before;
        //             cached_resp.timestamp_after = timestamp_after;
        //             return Ok(cached_resp);
        //         }
        //     }
        // }

        let data = response.bytes().await?.to_vec();

        let sr = ClientResponse {
            _url: url,
            timestamp_before,
            timestamp_after,
            // etag,
            data,
            // content_type,
            // last_modified,
            _status_code: status_code,
            _was_cached: false,
        };

        // if sr.etag.is_some() {
        //     self.cached_responses
        //         .insert(orig_url.to_string(), sr.clone());
        // }

        Ok(sr)
    }
}
