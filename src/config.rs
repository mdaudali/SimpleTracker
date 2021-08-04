use chrono::{DateTime, Utc};
use clap::{App, Arg};
use anyhow::{Result, anyhow};
use thiserror::Error;
use crate::ticker::Ticker;

#[derive(Error, Debug)]
pub enum ArgumentParsingError {
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),
}

#[derive(Debug)]
pub struct Config {
    pub tickers: Vec<Ticker>,
    pub from: DateTime<Utc>
}

impl Config {
    pub fn new() -> Result<Config> {
        let arg_matcher = App::new("SimpleTracker")
        .version("0.0.1")
        .arg(
            Arg::with_name("ticker")
                .short("t")
                .long("ticker")
                .value_name("TICKER")
                .help("Loads the stock data for the provided ticker")
                .required(true)
                .multiple(true)
        )
        .arg(Arg::with_name("from")
            .short("f")
            .long("from")
            .value_name("FROM")
            .help("Start date to load data from")
            ).get_matches();

        let ticker_values = arg_matcher.values_of("ticker").ok_or(anyhow!(ArgumentParsingError::MissingParameter("Ticker")))?;
        let tickers = ticker_values.into_iter().map(String::from).map(Ticker).collect();

        let from_value = arg_matcher.value_of("from").ok_or(ArgumentParsingError::MissingParameter("From"))?;
        let from = DateTime::parse_from_rfc3339(from_value)?.with_timezone(&Utc);

        let config = Config {
            tickers,
            from
        };
        Ok(config)
    }
}