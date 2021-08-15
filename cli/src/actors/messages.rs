use chrono::prelude::*;
use lib::formatter::{Percentage, Price};
use lib::performance_indicators::*;
use lib::ticker::Ticker;
use serde::Serialize;
use xactor::message;

#[message]
#[derive(Clone, PartialEq, Debug)]
pub struct PerformanceData {
    ticker: Ticker,
    window: usize,
    performance_data: Vec<f64>,
    to: DateTime<Utc>,
}

impl PerformanceData {
    pub fn new(
        ticker: Ticker,
        window: usize,
        performance_data: Vec<f64>,
        to: DateTime<Utc>,
    ) -> Self {
        PerformanceData {
            ticker,
            window,
            performance_data,
            to,
        }
    }

    pub fn ticker(&self) -> &Ticker {
        &self.ticker
    }

    pub fn window(&self) -> usize {
        self.window
    }

    pub fn performance_data(&self) -> &[f64] {
        &self.performance_data
    }

    pub fn to(&self) -> DateTime<Utc> {
        self.to
    }
}

#[message]
#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PerformanceIndicators {
    ticker: Ticker,
    time: DateTime<Utc>,
    current_price: Option<Price>,
    min: Option<Price>,
    max: Option<Price>,
    n_window_sma: Option<Price>,
    percentage_change: Option<Percentage>,
    abs_change: Option<Price>,
}

impl PerformanceIndicators {
    pub fn new(
        window: usize,
        series: &[f64],
        ticker: Ticker,
        time: DateTime<Utc>,
    ) -> PerformanceIndicators {
        let (percentage_change, abs_change) = match price_diff(series) {
            Some((percentage_change, abs_change)) => {
                (Some(Percentage(percentage_change)), Some(Price(abs_change)))
            }
            None => (None, None),
        };

        PerformanceIndicators {
            ticker,
            time,
            current_price: series.last().map(|x| Price(*x)),
            min: min(series).map(Price),
            max: max(series).map(Price),
            n_window_sma: n_window_sma(window, series)
                .and_then(|vec| vec.into_iter().map(Price).last()),
            percentage_change,
            abs_change,
        }
    }
}

#[message]
#[derive(Clone)]
pub struct Fetch {
    until: DateTime<Utc>,
}

impl Fetch {
    pub fn new() -> Self {
        Fetch::from_datetime(Utc::now())
    }

    pub fn from_datetime(until: DateTime<Utc>) -> Self {
        Fetch {
            until
        }
    }
    pub fn until(&self) -> DateTime<Utc> {
        self.until
    }
}

impl Default for Fetch {
    fn default() -> Self {
        Fetch::new()
    }
}
