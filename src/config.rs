use strum_macros::EnumString;
use chrono::{DateTime, Utc};
use clap::{App, Arg};
use std::str::FromStr;

#[derive(EnumString, Debug)]
enum Ticker {
    AAPL,
    GOOG,
    PLTR
}

#[derive(Debug)]
pub struct Config {
    ticker: Ticker,
    from: DateTime<Utc>
}

impl Config {
    pub fn new() -> Config {
        let arg_matcher = App::new("SimpleTracker")
        .version("0.0.1")
        .arg(
            Arg::with_name("ticker")
                .short("t")
                .long("ticker")
                .value_name("TICKER")
                .help("Loads the stock data for the provided ticker")
                .required(true)
        )
        .arg(Arg::with_name("from")
            .short("f")
            .long("from")
            .value_name("FROM")
            .help("Start date to load data from")
            ).get_matches();

        let config = Config {
            ticker: Ticker::from_str(arg_matcher.value_of("ticker").unwrap()).unwrap(),
            from: DateTime::parse_from_rfc2822(arg_matcher.value_of("from").unwrap()).unwrap().with_timezone(&Utc)
        };
        return config;
    }
}