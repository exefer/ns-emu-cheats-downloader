use super::{Cheat, CheatMap, CheatProvider};
use crate::curl_helper::BodyExt;
use curl::easy::Easy;
use serde::Deserialize;
use std::collections::HashMap;

pub struct HamletProvider;

impl CheatProvider for HamletProvider {
    fn get_cheats_for_title(&self, _title_name: &str, title_id: &str) -> Option<CheatMap> {
        let mut easy = Easy::new();
        let url = format!(
            "https://raw.githubusercontent.com/HamletDuFromage/switch-cheats-db/master/cheats/{}.json",
            title_id
        );
        easy.url(&url).ok()?;
        easy.useragent(env!("CARGO_PKG_NAME")).ok()?;
        let db = easy
            .without_body()
            .send_with_response::<HamletDatabase>()
            .ok()?;
        let mut result = HashMap::new();

        for build_id in db.build_ids() {
            if let Some(build_cheats) = db.get_cheats(&build_id) {
                let cheats = result.entry(build_id).or_insert_with(Vec::new);
                for (name, code) in build_cheats {
                    let code = code
                        .lines()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_owned())
                        .skip_while(|s| s.starts_with(['[', '{']))
                        .collect::<Vec<_>>();
                    if code.is_empty() {
                        continue;
                    }
                    cheats.push(Cheat {
                        name: name.trim_matches(['[', ']', '{', '}']).trim().to_owned(),
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
struct HamletDatabase {
    #[serde(flatten)]
    builds: HashMap<String, CheatEntry>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CheatEntry {
    Cheats(HashMap<String, String>),
    #[allow(dead_code)]
    Attribution(AttributionData),
}

#[derive(Deserialize)]
struct AttributionData {
    #[allow(dead_code)]
    #[serde(flatten)]
    files: HashMap<String, String>,
}

impl HamletDatabase {
    pub fn build_ids(&self) -> Vec<String> {
        self.builds
            .keys()
            .filter(|k| *k != "attribution")
            .cloned()
            .collect()
    }

    pub fn get_cheats(&self, build_id: &str) -> Option<&HashMap<String, String>> {
        match self.builds.get(build_id) {
            Some(CheatEntry::Cheats(cheats)) => Some(cheats),
            _ => None,
        }
    }
}
