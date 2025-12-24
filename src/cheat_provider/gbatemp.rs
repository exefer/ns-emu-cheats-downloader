use super::{CheatMap, CheatProvider};

pub struct GbaTempProvider;

impl CheatProvider for GbaTempProvider {
    fn get_cheats_for_title(&self, _title_name: &str, _title_id: &str) -> Option<CheatMap> {
        None
    }
}
