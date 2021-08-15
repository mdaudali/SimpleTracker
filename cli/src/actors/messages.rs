use chrono::prelude::*;
use lib::formatter::{Percentage, Price};
use lib::performance_indicators::*;
use lib::ticker::Ticker;
use log::error;
use serde::Serialize;
use xactor::{message, Actor, Addr, Broker, Context, Handler};

#[message]
#[derive(Clone, PartialEq, Debug)]
pub struct PerformanceData {
    ticker: Ticker,
    window: usize,
    performance_data: Vec<f64>,
    to: DateTime<Utc>,
}

impl PerformanceData {
    pub fn of(
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
    pub fn create(
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
            current_price: series.last().map(|x| Price(x.clone())),
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
    to: DateTime<Utc>,
}

impl Fetch {
    pub fn new() -> Self {
        Fetch::of(Utc::now())
    }

    pub fn of(to: DateTime<Utc>) -> Self {
        Fetch { to }
    }

    pub fn to(&self) -> DateTime<Utc> {
        self.to
    }
}
