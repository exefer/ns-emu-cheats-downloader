use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use curl::easy::Easy;
use serde::{Deserialize, Deserializer};

use crate::curl_helper::BodyExt;
use crate::utils::strip_html;

pub const BASE_URL: &str = "https://tinfoil.media";

pub fn fetch_titles(titles: Arc<Mutex<Vec<TinfoilTitle>>>, error: Arc<Mutex<Option<String>>>) {
    *error.lock().unwrap() = None;
    thread::spawn(move || match request_titles() {
        Ok(data) => *titles.lock().unwrap() = data,
        Err(e) => *error.lock().unwrap() = Some(format!("{e:#}")),
    });
}

fn request_titles() -> anyhow::Result<Vec<TinfoilTitle>> {
    let mut easy = Easy::new();
    easy.url(&format!("{BASE_URL}/Title/ApiJson/"))?;
    easy.useragent(env!("CARGO_PKG_NAME"))?;
    easy.fail_on_error(true)?;
    easy.connect_timeout(Duration::from_secs(10))?;
    easy.low_speed_limit(1)?;
    easy.low_speed_time(Duration::from_secs(15))?;
    Ok(easy
        .without_body()
        .send_with_response::<TinfoilRoot>()?
        .data)
}

pub fn fetch_icon(title_id: &str) -> Result<Vec<u8>, curl::Error> {
    let mut easy = Easy::new();
    let mut bytes = Vec::new();
    easy.url(&format!("{BASE_URL}/ti/{title_id}/235/235"))?;
    easy.useragent(env!("CARGO_PKG_NAME"))?;
    easy.fail_on_error(true)?;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            bytes.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    Ok(bytes)
}

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.filter(|s| !s.is_empty()))
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
