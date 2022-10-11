pub mod query;
pub mod mutation;
pub mod common;

use reqwest::{ Client };
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::{ json, value::RawValue };
use anyhow::Context;

const ROOT_URL: &str = "https://api.github.com/graphql";

/// A quick Github GraphQL API client.
#[derive(Debug)]
pub struct Api {
    client: Client,
    token: String
}

impl Api {
    pub fn new(token: String) -> Api {
        let client = Client::new();
        Api { token, client }
    }

    /// Send a GraphQL query with variables.
    pub async fn query<Res: DeserializeOwned>(&self, query: &str, variables: Variables) -> Result<Res, anyhow::Error> {
        let err_context = |msg: &str| {
            // pull out the name given to the query if possible:
            let first_line = query
                .trim_start()
                .lines()
                .next()
                .unwrap_or("<empty query>")
                .trim_end()
                .trim_end_matches('{')
                .trim_end();
            format!("{first_line}: {msg}")
        };

        let res = self.client
            .post(ROOT_URL)
            .bearer_auth(&self.token)
            .header("User-Agent", "jsdw-parity-project-sync")
            .json(&json!({
                "query": query,
                "variables": variables.build()
            }))
            .send()
            .await
            .with_context(|| err_context("Failed to send request"))?;

        // If there are errors, return them. Else if we get data back, we're all good.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum QueryResult<Res> {
            Err { errors: Vec<QueryError> },
            Ok { data: Res },
        }
        let status = res.status();
        if status.is_success() {
            // Broken down the steps to allow better debugging in case of issue:
            let text = res.text().await
                .with_context(|| {
                    err_context("Failed to obtain string response")
                })?;
            let body: QueryResult<Res> = serde_json::from_str(&text)
                .with_context(|| {
                    println!("{text}");
                    err_context("Failed to decode response")
                })?;
            match body {
                QueryResult::Ok { data } => Ok(data),
                QueryResult::Err { errors } => {
                    Err(ApiError::QueryErrors(errors))
                        .with_context(|| err_context("GraphQL errors encountered with request"))
                }
            }
        } else {
            let body = res.text().await?;
            Err(ApiError::BadResponse(status.as_u16(), body))
                .with_context(|| err_context("Bad response making request"))
        }
    }
}

/// This represents variables you can pass to a GraphQL query.
pub struct Variables {
    json: Vec<u8>
}

impl Variables {
    pub fn new() -> Self {
        Variables { json: Vec::new() }
    }
    pub fn push<T: Serialize>(&mut self, name: &str, value: T) {
        if self.json.is_empty() {
            self.json.push(b'{');
        } else {
            self.json.push(b',');
        }

        serde_json::to_writer(&mut self.json, name).unwrap();
        self.json.push(b':');
        serde_json::to_writer(&mut self.json, &value).unwrap();
    }
    fn build(mut self) -> Option<Box<RawValue>> {
        if self.json.is_empty() {
            return None;
        }

        self.json.push(b'}');

        // Everything we push is valid UFT8 so this is fine.
        let json = unsafe { String::from_utf8_unchecked(self.json) };

        Some(RawValue::from_string(json).unwrap())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    RequestError(#[from] reqwest::Error),
    #[error("{0} response: {1}")]
    BadResponse(u16, String),
    #[error("Errors with query: {0:?}")]
    QueryErrors(Vec<QueryError>),
    #[error("{0}")]
    DecodeError(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize)]
pub struct QueryError {
    pub path: Option<Vec<String>>,
    pub message: String
}

#[macro_export]
macro_rules! variables {
    ($($key:literal : $val:expr), *) => {{
        // May be unused if empty; no params.
        #[allow(unused_mut)]
        let mut params = $crate::api::Variables::new();
        $(
            params.push($key, $val);
        )*
        params
    }}
}