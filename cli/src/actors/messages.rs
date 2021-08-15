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
        Fetch { until }
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

#[cfg(test)]
mod test {
    use super::PerformanceIndicators;
    use chrono::Utc;
    use lib::formatter::{Percentage, Price};
    use lib::ticker::Ticker;
    #[test]
    fn performance_indicators_constructor_has_correct_metrics() {
        let series = [15f64, 13f64, 2f64, 7.5f64];
        let time = Utc::now();
        let expected = PerformanceIndicators {
            ticker: Ticker::from("TEST"),
            current_price: Some(Price(7.5f64)),
            time,
            min: Some(Price(2f64)),
            max: Some(Price(15f64)),
            n_window_sma: Some(Price(4.75f64)),
            percentage_change: Some(Percentage(50f64)),
            abs_change: Some(Price(-7.5f64)),
        };
        assert_eq!(
            PerformanceIndicators::new(2, &series, Ticker::from("TEST"), time),
            expected
        );
    }
}
