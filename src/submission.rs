use crate::{
    storage::{Storable, Storage},
    Filterable, FromJsonString,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone)]
pub struct Submission {
    pub author: Option<String>,
    pub url: Option<String>,
    pub permalink: String,
    pub score: Option<i32>,
    pub title: String,
    pub selftext: String,
    pub domain: Option<String>,
    pub author_flair_text: Option<String>,
    pub subreddit: Option<String>,
    pub subreddit_id: Option<String>,
    pub id: String,
    pub num_comments: i32,
    pub over_18: bool,
    pub is_self: bool,
    pub link_flair_text: Option<String>,
    pub spoiler: Option<bool>,
    pub pinned: Option<bool>,
    #[serde(default)]
    pub stickied: bool,
    pub num_crossposts: Option<u32>,
    pub ups: Option<i32>,
    pub downs: Option<i32>,
    #[serde(deserialize_with = "deserialize_time")]
    pub created_utc: i64,
    pub retrieved_on: Option<i64>,
}

impl FromJsonString for Submission {
    fn from_json_str(line: &str) -> Result<Self> {
        serde_json::from_str(line.trim_matches(char::from(0)))
            .with_context(|| format!("Failed to deserialize line: {line}"))
    }
}

impl Filterable for Submission {
    fn score(&self) -> Option<i32> {
        self.score
    }
    fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }
    fn subreddit(&self) -> Option<&str> {
        self.subreddit.as_deref()
    }
    fn created(&self) -> i64 {
        self.created_utc
    }
}

impl Storable for Submission {
    fn store<T: Storage>(&self, storage: &mut T) -> Result<usize> {
        storage.insert_submission(self)
    }
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let json = serde_json::Value::deserialize(deserializer)?;
    match json {
        serde_json::Value::Number(val) => {
            if let Some(int_val) = val.as_i64() {
                Ok(int_val)
            } else if let Some(float_val) = val.as_f64() {
                // Convert the floating-point value to an integer.
                // You can choose to round or truncate based on your needs.
                // Here, we're simply truncating.
                Ok(float_val.trunc() as i64)
            } else {
                Err(serde::de::Error::custom("invalid timestamp value, expected an integer or a floating-point number"))
            }
        },
        serde_json::Value::String(val) => {
            val.parse::<i64>().map_err(|_| {
                serde::de::Error::custom(format!("unable to parse timestamp string: {:?}", val))
            })
        },
        _ => Err(serde::de::Error::custom("invalid timestamp value, expected number or string")),
    }
}
