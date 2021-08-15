use serde::Serialize;
use std::fmt::{Display, Formatter, Error};
use std::str::FromStr;
use std::convert::Infallible;
#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Ticker(String);

impl Ticker {
    pub fn new(ticker: String) -> Self {
        Ticker(ticker)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Ticker {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Ticker {
    fn from(t: &str) -> Self {
        Ticker::new(t.to_owned())
    }
}

impl FromStr for Ticker {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
       Ok(Ticker::from(s))
    }
}

impl AsRef<str> for Ticker {
    fn as_ref(&self) -> &str {
        &self.0
    }
}