use chrono::{DateTime, Utc};
use crate::ticker::Ticker;
use serde::Serialize;
use std::io::Write;
use csv::Writer;
use anyhow::{Result, anyhow};
use crate::percentage::Percentage;
use crate::price::Price;
#[derive(Serialize)]
pub struct Fields {
    pub period_start: DateTime<Utc>,
    pub symbol: Ticker,
    pub price: Option<Price>,
    pub change: Option<Percentage>,
    pub min: Option<Price>,
    pub max: Option<Price>,
    pub thirty_day_average: Option<Price>
}

pub fn to_csv(fields: &[Fields], output: Box<dyn Write>) -> Result<()> {
    let mut wtr = Writer::from_writer(output);
    fields.iter().try_for_each(|field| wtr.serialize(field).map_err(|err| anyhow!(err)))?;
    Ok(())
}