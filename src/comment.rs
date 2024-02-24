use crate::{
    storage::{Storable, Storage},
    Filterable, FromJsonString,
};
use anyhow::{Context, Result};
use serde::{de, Deserialize, Deserializer};

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Comment {
    pub author: String,
    pub body: String,
    pub subreddit: String,
    #[serde(default)]
    pub author_flair_text: Option<String>,
    #[serde(default)]
    author_flair_css_class: Option<String>,
    pub score: Option<i32>,
    pub ups: Option<i32>,
    pub downs: Option<i32>,
    pub created_utc: i64,
    #[serde(default)]
    pub retrieved_on: Option<i64>,
    pub link_id: String,
    pub id: String,
    pub permalink: Option<String>,
    pub parent_id: ParentId,
    #[serde(default)]
    pub parent_is_post: bool,
    #[serde(default)]
    pub stickied: bool,
    #[serde(default)]
    is_submitter: bool,
    #[serde(default)]
    pub distinguished: Option<String>,
    //    edited: Option<Edited>,
    #[serde(default)]
    archived: bool,
    controversiality: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ParentId {
    pub parent_type: u8,
    pub parent_id: String,
}

impl<'de> Deserialize<'de> for ParentId {
    fn deserialize<D>(deserializer: D) -> Result<ParentId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parent: Option<String> = Option::deserialize(deserializer)?;
        match parent {
            Some(parent) => {
                let make_err =
                    || de::Error::invalid_value(de::Unexpected::Str(&parent), &"a valid parent id");
                let (ty, parent_id) = parent.split_once('_').ok_or_else(make_err)?;
                let type_char = ty.chars().nth(1).ok_or_else(make_err)?;
                let parent_type: u8 = type_char
                    .to_digit(10)
                    .ok_or_else(make_err)?
                    .try_into()
                    .map_err(|_| make_err())?;
                Ok(ParentId {
                    parent_type,
                    parent_id: parent_id.into(),
                })
            },
            None => Err(de::Error::custom("parent_id is null")),
        }
    }
}

impl FromJsonString for Comment {
    fn from_json_str(line: &str) -> Result<Self> {
        let mut json: serde_json::Value = serde_json::from_str(line)
            .with_context(|| format!("Failed to read json for line: {line}"))?;
        
        // Check and convert `created_utc` from floating-point to integer if necessary
        if let Some(serde_json::Value::Number(num)) = json.get("created_utc") {
            if num.is_f64() {
                let utc = num.as_f64().unwrap() as i64; // Convert floating-point to i64
                json["created_utc"] = serde_json::Value::Number(serde_json::Number::from(utc));
            }
        }

        // Similar conversion logic can be applied to other fields if necessary

        let comment = Comment::deserialize(&json)
            .with_context(|| format!("Failed to deserialize line: {line}"))?;

        Ok(comment)
    }
}

impl Filterable for Comment {
    fn score(&self) -> Option<i32> {
        self.score
    }
    fn author(&self) -> Option<&str> {
        Some(self.author.as_str())
    }
    fn subreddit(&self) -> Option<&str> {
        Some(self.subreddit.as_str())
    }
    fn created(&self) -> i64 {
        self.created_utc
    }
}

impl Storable for Comment {
    fn store<T: Storage>(&self, storage: &mut T) -> Result<usize> {
        storage.insert_comment(self)
    }
}
