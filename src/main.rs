mod config;
mod performance_indicators;
fn main() {
    let config = config::Config::new().unwrap();
    println!("Config: {:?}", config);

    // let provider = yahoo::YahooConnector::new();
    // println!("Quotes: {:?}", provider.get_latest_quotes("AAPL", "1m"));
}


