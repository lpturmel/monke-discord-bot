use crate::{Error, Handle, Result};
use std::str::FromStr;

pub struct IdsRequestBuilder {
    request: reqwest::Request,
    handle: std::sync::Arc<Handle>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    count: Option<usize>,
}

impl IdsRequestBuilder {
    pub fn new(handle: std::sync::Arc<Handle>, url: String) -> Self {
        Self {
            handle,
            request: reqwest::Request::new(
                reqwest::Method::GET,
                reqwest::Url::from_str(&url).unwrap(),
            ),
            start_time: None,
            end_time: None,
            count: None,
        }
    }
    pub fn start_time(mut self, start_time: i64) -> Self {
        self.start_time = Some(start_time);
        self
    }
    pub fn end_time(mut self, end_time: i64) -> Self {
        self.end_time = Some(end_time);
        self
    }
    /// Set the number of games to be returned.
    ///
    /// Defaults to 20
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
    pub async fn send(mut self) -> Result<Vec<String>> {
        let count = self.count.unwrap_or(20);

        let url = self.request.url_mut();

        url.query_pairs_mut()
            .append_pair("count", &count.to_string());

        if let Some(start_time) = self.start_time {
            url.query_pairs_mut()
                .append_pair("startTime", &start_time.to_string());
        }
        if let Some(end_time) = self.end_time {
            url.query_pairs_mut()
                .append_pair("endTime", &end_time.to_string());
        }

        println!("{:?}", self.request.url());
        let res = self.handle.web.execute(self.request).await?;
        match res.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::NOT_FOUND => return Err(Error::SummonerNotFound)?,
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(Error::TooManyRequests)?,
            reqwest::StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized)?,
            reqwest::StatusCode::FORBIDDEN => return Err(Error::Forbidden)?,
            reqwest::StatusCode::BAD_REQUEST => return Err(Error::BadRequest)?,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => return Err(Error::RiotError)?,
            _ => {}
        }
        let match_ids: Vec<String> = res.json().await?;
        Ok(match_ids)
    }
}
