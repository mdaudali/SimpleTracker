use std::fmt;
use serde::{Serializer, Serialize};
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Percentage(pub f64);

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}%", self.0)
    }
}

impl Serialize for Percentage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:.2}%", self.0))
    }
}