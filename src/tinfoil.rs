use crate::utils::strip_html;
use serde::{Deserialize, Deserializer};

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(s) if s.is_empty() => Ok(None),
        other => Ok(other),
    }
}

fn strip_html_tags<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(strip_html(&s))
}

#[derive(Deserialize)]
pub struct TinfoilRoot {
    pub data: Vec<TinfoilTitle>,
}

#[derive(Clone, Deserialize)]
pub struct TinfoilTitle {
    pub id: String,
    #[serde(deserialize_with = "strip_html_tags")]
    pub name: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub release_date: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub publisher: Option<String>,
}
