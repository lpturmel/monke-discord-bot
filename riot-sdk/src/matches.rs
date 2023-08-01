use crate::ServiceUrl;
use std::fmt::Display;

pub enum Region {
    AMERICAS,
    ASIA,
    EUROPE,
    SEA,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let region = match self {
            Region::AMERICAS => "americas",
            Region::ASIA => "asia",
            Region::EUROPE => "europe",
            Region::SEA => "sea",
        };
        write!(f, "{}", region)
    }
}
impl ServiceUrl for Region {}
