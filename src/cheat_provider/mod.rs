mod blawar;
mod cheatslips;
mod github;
mod hamlet;
mod tinfoil;

use github::GithubProvider;
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum CheatSource {
    Blawar,
    Chansey,
    CheatSlips,
    GbaTemp,
    Hamlet,
    Ibnux,
    Tinfoil,
}

impl CheatSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blawar => "Blawar",
            Self::Chansey => "Chansey",
            Self::CheatSlips => "CheatSlips",
            Self::GbaTemp => "GbaTemp",
            Self::Hamlet => "Hamlet",
            Self::Ibnux => "Ibnux",
            Self::Tinfoil => "Tinfoil",
        }
    }

    pub fn provider(&self) -> Box<dyn CheatProvider> {
        match self {
            Self::Blawar => Box::new(blawar::BlawarProvider),
            Self::Chansey => Box::new(GithubProvider::new(
                "https://api.github.com/repos/ChanseyIsTheBest/NX-60FPS-RES-GFX-Cheats/contents/titles/{}/cheats",
            )),
            Self::CheatSlips => Box::new(cheatslips::CheatSlipsProvider),
            Self::GbaTemp => Box::new(GithubProvider::new(
                "https://api.github.com/repos/exefer/gbatemp-matias3ds-cheats/contents/titles/{}/cheats",
            )),
            Self::Hamlet => Box::new(hamlet::HamletProvider),
            Self::Ibnux => Box::new(GithubProvider::new(
                "https://api.github.com/repos/ibnux/switch-cheat/contents/atmosphere/titles/{}/cheats",
            )),
            Self::Tinfoil => Box::new(tinfoil::TinfoilProvider),
        }
    }
}

pub type CheatMap = HashMap<String, Vec<Cheat>>;

pub trait CheatProvider: Send {
    fn get_cheats_for_title(&self, title_name: &str, title_id: &str) -> Option<CheatMap>;
}

pub struct Cheat {
    pub name: String,
    pub code: Vec<String>,
    pub checked: bool,
}

fn parse_cheat_file(content: &str) -> Vec<Cheat> {
    let mut cheats = Vec::new();
    let mut current_name = None;
    let mut current_code = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .map(|s| s.trim())
        {
            if let Some(name) = current_name.take() {
                if !current_code.is_empty() {
                    cheats.push(Cheat {
                        name,
                        code: current_code,
                        checked: false,
                    });
                }
                current_code = Vec::new();
            }
            current_name = Some(name.to_owned());
        } else if !trimmed.is_empty() && current_name.is_some() {
            current_code.push(trimmed.to_owned());
        }
    }

    if let Some(name) = current_name
        && !current_code.is_empty()
    {
        cheats.push(Cheat {
            name,
            code: current_code,
            checked: false,
        });
    }

    cheats
}
