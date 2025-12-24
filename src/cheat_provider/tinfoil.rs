use super::{Cheat, CheatMap, CheatProvider};
use crate::curl_helper::BodyExt;
use curl::easy::Easy;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

pub struct TinfoilProvider;

impl CheatProvider for TinfoilProvider {
    fn get_cheats_for_title(&self, _title_name: &str, title_id: &str) -> Option<CheatMap> {
        let mut easy = Easy::new();
        let url = format!("https://tinfoil.io/Title/{}", title_id);
        easy.url(&url).ok()?;
        easy.useragent(env!("CARGO_PKG_NAME")).ok()?;
        let html = easy.without_body().text().ok()?;
        let html = Html::parse_fragment(&html);
        let mut patches: HashMap<String, String> = HashMap::new();

        for tr in html.select(&Selector::parse("table.fixed > tbody > tr").ok()?) {
            let mut iter = tr.child_elements();
            let build_id = iter.next()?.text().collect();
            let patch = iter.next()?.text().collect();
            patches.insert(patch, build_id);
        }

        let h4 = html
            .select(&Selector::parse("div > h4:nth-child(1)").ok()?)
            .next_back()
            .filter(|e| e.text().any(|t| t == "Cheats"))?;
        let div = ElementRef::wrap(h4.next_siblings().nth(1)?)?;
        let ul_selector = Selector::parse("ul > li").ok()?;
        let mut result = HashMap::new();

        for tr in div.select(&Selector::parse("table > tbody > tr").ok()?) {
            let mut iter = tr.child_elements();
            let name = {
                let name = iter.next()?.text().next()?;
                name.trim_matches(['[', ']', '{', '}']).trim().to_owned()
            };
            let patch = iter.next()?.text().next()?;
            iter.next()?;
            let code = iter
                .next()?
                .select(&ul_selector)
                .map(|li| li.text().filter(|t| !t.is_empty()).collect())
                .skip_while(|s: &String| s.starts_with(['[', '{']))
                .collect::<Vec<_>>();
            if code.is_empty() {
                continue;
            }
            let build_id = patches.get(patch)?;
            result
                .entry(build_id.clone())
                .or_insert_with(Vec::new)
                .push(Cheat {
                    name,
                    code,
                    checked: false,
                });
        }
        Some(result)
    }
}
