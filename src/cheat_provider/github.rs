use super::{CheatMap, CheatProvider, parse_cheat_file};
use crate::curl_helper::BodyExt;
use curl::easy::Easy;
use serde_json::Value;
use std::collections::HashMap;

pub struct GithubProvider {
    url_template: &'static str,
}

impl GithubProvider {
    pub fn new(url_template: &'static str) -> Self {
        Self { url_template }
    }
}

impl CheatProvider for GithubProvider {
    fn get_cheats_for_title(&self, _title_name: &str, title_id: &str) -> Option<CheatMap> {
        let mut easy = Easy::new();
        let url = self.url_template.replace("{}", title_id);
        easy.url(&url).ok()?;
        easy.useragent(env!("CARGO_PKG_NAME")).ok()?;
        let data = easy.without_body().send_with_response::<Value>().ok()?;
        let mut result = HashMap::new();

        for item in data.as_array()? {
            let build_id = {
                let name = item["name"].as_str()?;
                name.trim_end_matches(".txt").to_uppercase()
            };
            let url = item["download_url"].as_str()?;
            easy.url(url).ok()?;
            let content = easy.without_body().text().ok()?;
            result
                .entry(build_id)
                .or_insert_with(Vec::new)
                .extend(parse_cheat_file(&content));
        }
        Some(result)
    }
}
