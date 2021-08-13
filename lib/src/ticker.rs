use serde::Serialize;
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Ticker(pub String);
