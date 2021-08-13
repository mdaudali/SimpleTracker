use lib::ticker::Ticker;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{App, Arg};
use std::fs::read_to_string;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArgumentParsingError {
    #[error("Missing parameter: {0}")]
    MissingParameter(&'static str),

    #[error("One of {0} {1} required")]
    OneRequired(&'static str, &'static str),

    #[error("Only one of {0} {1} required")]
    TooManyParameters(&'static str, &'static str),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub tickers: Vec<Ticker>,
    pub from: DateTime<Utc>,
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
                    // .required(true)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("ticker_file")
                    .short("i")
                    .long("ticker-file")
                    .value_name("TICKER FILE")
                    .help("Loads a comma delimited file of tickers"),
            )
            .arg(
                Arg::with_name("from")
                    .short("f")
                    .long("from")
                    .value_name("FROM")
                    .help("Start date to load data from"),
            )
            .get_matches();

        let tickers: Vec<Ticker> = match (
            arg_matcher.values_of("ticker"),
            arg_matcher.value_of("ticker_file"),
        ) {
            (None, None) => Err(anyhow!(ArgumentParsingError::OneRequired(
                "Ticker",
                "Ticker-file"
            ))),
            (Some(tickers), None) => {
                Ok(tickers.into_iter().map(String::from).map(Ticker).collect())
            }
            (None, Some(tickerfile)) => {
                let tickers = read_to_string(tickerfile)?;
                Ok(tickers.split(',').map(String::from).map(Ticker).collect())
            }
            (Some(_), Some(_)) => Err(anyhow!(ArgumentParsingError::TooManyParameters(
                "Ticker",
                "Ticker-File"
            ))),
        }?;
        let from_value = arg_matcher
            .value_of("from")
            .ok_or(ArgumentParsingError::MissingParameter("From"))?;
        let from = DateTime::parse_from_rfc3339(from_value)?.with_timezone(&Utc);

        let config = Config { tickers, from };
        Ok(config)
    }
}
