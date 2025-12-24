mod blawar;
mod chansey;
mod cheatslips;
mod gbatemp;
mod hamlet;
mod ibnux;
mod tinfoil;

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
            Self::Chansey => Box::new(chansey::ChanseyProvider),
            Self::CheatSlips => Box::new(cheatslips::CheatSlipsProvider),
            Self::GbaTemp => Box::new(gbatemp::GbaTempProvider),
            Self::Hamlet => Box::new(hamlet::HamletProvider),
            Self::Ibnux => Box::new(ibnux::IbnuxProvider),
            Self::Tinfoil => Box::new(tinfoil::TinfoilProvider),
        }
    }
}

pub trait CheatProvider {}
