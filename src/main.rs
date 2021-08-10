use yahoo_finance_api::YahooConnector;
use anyhow::Result;
use futures::{
    stream::{
        FuturesUnordered,
        StreamExt
    },  
};
use chrono::prelude::*;

mod config;
mod performance_indicators;
mod output;
mod ticker;
mod price;
mod percentage;

#[async_std::main]
async fn main() -> Result<()> {
    let config = config::Config::new()?;

    let provider = YahooConnector::new();
    let output_fields = config.clone().tickers.iter().map(|ticker| {
        let provider = &provider;
        let config_clone = &config;
        async move {
            let quote = provider.get_quote_history(&ticker.0, config_clone.from, chrono::offset::Utc::now()).await?;
            let quotes = quote.quotes()?;
            let series: Vec<f64> = quotes.iter().map(|q| q.adjclose).collect();
            let performance_indicators = performance_indicators::PerformanceIndicators::create(30, &series[..]);
            Ok::<output::Fields, anyhow::Error>(output::Fields {
                period_start: config_clone.from,
                symbol: ticker.clone(),
                price: series.last().map(|e| price::Price(*e)),
                change: performance_indicators.percentage_change,
                min: performance_indicators.min,
                max: performance_indicators.max,
                thirty_day_average: performance_indicators.n_window_sma.and_then(|sma| sma.last().map(|e| *e))
        })
    }}).collect::<FuturesUnordered<_>>().collect::<Vec<Result<_>>>().await.into_iter().collect::<Result<Vec<_>>>()?;
    output::to_csv(&output_fields, Box::new(std::io::stdout()))
    
}
