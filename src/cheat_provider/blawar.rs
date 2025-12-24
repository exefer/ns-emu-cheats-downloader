use super::{Cheat, CheatMap, CheatProvider};
use crate::curl_helper::BodyExt;
use curl::easy::Easy;
use serde::Deserialize;
use std::collections::HashMap;

pub struct BlawarProvider;

impl CheatProvider for BlawarProvider {
    fn get_cheats_for_title(&self, _title_name: &str, title_id: &str) -> Option<CheatMap> {
        let mut easy = Easy::new();
        let url = "https://raw.githubusercontent.com/blawar/titledb/master/cheats.json";
        easy.url(url).ok()?;
        easy.useragent(env!("CARGO_PKG_NAME")).ok()?;
        let db = easy
            .without_body()
            .send_with_response::<BlawarDatabase>()
            .ok()?;
        let builds = db.titles.get(title_id)?;
        let mut result = HashMap::new();

        for (build_id, build_data) in builds {
            for entry in build_data.entries.values() {
                if let BuildEntry::Cheat(CheatData { title, source }) = entry {
                    let code = source
                        .lines()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .skip_while(|s| s.starts_with(['[', '{']))
                        .map(|s| s.to_owned())
                        .collect();
                    result
                        .entry(build_id.clone())
                        .or_insert_with(Vec::new)
                        .push(Cheat {
                            name: title.trim_matches(['[', ']']).trim().to_owned(),
                            code,
                            checked: false,
                        });
                }
            }
        }
        Some(result)
    }
}

#[derive(Deserialize)]
struct BlawarDatabase {
    #[serde(flatten)]
    titles: HashMap<String, HashMap<String, BuildData>>,
}

#[derive(Deserialize)]
struct BuildData {
    #[serde(flatten)]
    entries: HashMap<String, BuildEntry>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BuildEntry {
    Cheat(CheatData),
    #[allow(dead_code)]
    Version(u64),
}

#[derive(Deserialize)]
struct CheatData {
    title: String,
    source: String,
}
