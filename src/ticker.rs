use serde::Serialize;
#[derive(Debug, Serialize, Clone)]
pub struct Ticker(pub String);
