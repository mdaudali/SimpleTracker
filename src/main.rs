use yahoo_finance_api::YahooConnector;
use anyhow::Result;

mod config;
mod performance_indicators;
mod output;
mod ticker;
mod price;
mod percentage;
fn main() -> Result<()> {
    let config = config::Config::new()?;

    let provider = YahooConnector::new();
    
    let output_fields: Vec<output::Fields> = config.tickers.iter().map(|ticker| {
        let series = provider.get_quote_history(&ticker.0, config.from, chrono::offset::Utc::now())?;
        let quotes = series.quotes()?;
        let series: Vec<f64> = quotes.iter().map(|q| q.adjclose).collect();
        let performance_indicators = performance_indicators::PerformanceIndicators::create(30, &series[..]);
        Ok(output::Fields {
            period_start: config.from,
            symbol: ticker.clone(),
            price: price::Price(series[series.len() - 1]),
            change: performance_indicators.percentage_change,
            min: performance_indicators.min,
            max: performance_indicators.max,
            thirty_day_average: performance_indicators.n_window_sma.map(|sma| sma[sma.len() - 1])
    })
}).collect::<Result<Vec<output::Fields>>>()?;

    output::to_csv(&output_fields, Box::new(std::io::stdout()))
    
}