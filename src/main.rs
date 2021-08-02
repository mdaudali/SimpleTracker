use yahoo_finance_api as yahoo;
use clap::{App, Arg};
use chrono::{Utc, DateTime};
mod config;

fn main() {
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
    let config = ConfigMatches {
        ticker: Ticker::from_str(arg_matcher.value_of("ticker").unwrap()).unwrap(),
        from: DateTime::parse_from_rfc2822(arg_matcher.value_of("from").unwrap()).unwrap().with_timezone(&Utc)
    };

    println!("Config: {:?}", config);

    let provider = yahoo::YahooConnector::new();
    println!("Quotes: {:?}", provider.get_latest_quotes("AAPL", "1m"));
}


