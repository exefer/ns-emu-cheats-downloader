use super::{Cheat, CheatMap, CheatProvider};
use crate::curl_helper::BodyExt;
use crate::utils::normalize_title_name;
use curl::easy::Easy;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub struct CheatSlipsProvider;

impl CheatProvider for CheatSlipsProvider {
    fn get_cheats_for_title(&self, title_name: &str, _title_id: &str) -> Option<CheatMap> {
        let mut easy = Easy::new();
        let title = normalize_title_name(title_name);
        let url = format!("https://www.cheatslips.com/game/{}", title);
        easy.url(&url).ok()?;
        easy.useragent(env!("CARGO_PKG_NAME")).ok()?;
        let html = easy.without_body().text().ok()?;
        let html = Html::parse_fragment(&html);
        let a_selector = Selector::parse("tr > td > a").ok()?;
        let ts_selector = Selector::parse(".text-secondary").ok()?;
        let st_selector = Selector::parse("strong").ok()?;
        let pre_selector = Selector::parse("pre").ok()?;
        let mut result = HashMap::new();

        for a in html.select(&a_selector) {
            let build_id = a.text().next()?;
            let url = format!("https://www.cheatslips.com/game/{}/{}", title, build_id);
            easy.url(&url).ok()?;
            let html = easy.without_body().text().ok()?;
            let html = Html::parse_fragment(&html);
            let mut cheats = Vec::new();

            for a in html.select(&ts_selector) {
                let segment = a.value().attr("href")?.rsplit('/').next()?;
                let url = format!(
                    "https://www.cheatslips.com/game/{}/{}/sources",
                    title, segment
                );
                easy.url(&url).ok()?;
                let html = easy.without_body().text().ok()?;
                let html = Html::parse_fragment(&html);

                for tbody in html.select(&Selector::parse("tbody").ok()?) {
                    let mut iter = tbody.child_elements();
                    let name = {
                        let name = iter.next()?.select(&st_selector).next()?.text().next()?;
                        name.trim_matches(['[', ']', '{', '}']).trim().to_owned()
                    };
                    let Some(source) = tbody
                        .select(&pre_selector)
                        .next()
                        .and_then(|pre| pre.text().next())
                        .filter(|text| !text.is_empty())
                    else {
                        continue;
                    };
                    let code = source
                        .lines()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>();
                    if code.is_empty() {
                        continue;
                    }
                    cheats.push(Cheat {
                        name,
                        code,
                        checked: false,
                    });
                }
            }
            if !cheats.is_empty() {
                result
                    .entry(build_id.to_uppercase())
                    .or_insert_with(Vec::new)
                    .extend(cheats);
            }
        }
        Some(result)
    }
}
