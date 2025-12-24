mod imp;

use crate::utils::strip_html;
use gtk::glib;
use serde::{Deserialize, Deserializer};

glib::wrapper! {
    pub struct TinfoilTitleObject(ObjectSubclass<imp::TinfoilTitleObject>);
}

impl TinfoilTitleObject {
    pub fn from_title(title: TinfoilTitle) -> Self {
        let name_lowercase = title.name.to_lowercase();
        glib::Object::builder()
            .property("id", title.id)
            .property("name", title.name)
            .property("name-lowercase", name_lowercase)
            .property("release-date", title.release_date)
            .property("publisher", title.publisher)
            .build()
    }
}

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

#[derive(Default, Debug, Deserialize)]
pub struct TinfoilRoot {
    pub data: Vec<TinfoilTitle>,
}

#[derive(Default, Debug, Deserialize)]
pub struct TinfoilTitle {
    pub id: String,
    #[serde(deserialize_with = "strip_html_tags")]
    pub name: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub release_date: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub publisher: Option<String>,
}
