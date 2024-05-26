use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{bail, misc::now_utc, Result};

pub type Tags = HashMap<String, Option<String>>;
pub type TagDict = HashMap<String, TagValue>;

pub fn valid_nonspace_tag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':' || c == '·' || c == '.' || c == '：'
}

pub fn valid_tag_char(c: char) -> bool {
    valid_nonspace_tag_char(c) || c == ' '
}

pub fn scope_of(tag: &str) -> &str {
    if tag.starts_with('@') {
        tag.split_once('.').map_or(tag, |(pre, _)| pre)
    } else {
        ""
    }
}

pub fn validate_tag_name(tag: &str) -> Result<()> {
    let result: Result<(), &'static str> = (|| {
        let mut chars = tag.chars();
        let Some(first) = chars.next() else {
            return Err("empty tag");
        };
        if (first != '@' && first != '#' && !valid_tag_char(first)) || !chars.all(valid_tag_char) {
            return Err("invalid tag");
        }
        if tag.len() > 200 {
            return Err("tag too long");
        }
        Ok(())
    })();
    if let Err(err) = result {
        bail!(@InvalidTag ("tag" => tag.to_owned()) "{err}");
    }

    Ok(())
}
pub fn validate_tag_value(tag: &str, value: Option<&str>) -> Result<()> {
    if value.map_or(false, |it| it.len() > 2000) {
        bail!(@InvalidTag ("tag" => tag.to_owned()) "value too long");
    }
    Ok(())
}

pub fn to_tag_dict(tags: Tags) -> (DateTime<Utc>, TagDict) {
    let now = now_utc();
    (
        now,
        tags.into_iter()
            .map(|(k, v)| (k, TagValue::new(v, now)))
            .collect(),
    )
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TagValue {
    #[serde(rename = "v")]
    pub value: Option<String>,
    #[serde(rename = "u")]
    pub updated: DateTime<Utc>,
}

impl TagValue {
    pub fn new(value: Option<String>, time: DateTime<Utc>) -> Self {
        Self {
            value,
            updated: time,
        }
    }
}
