use crate::actors::output_actor::Output;
use crate::formatter::{Percentage, Price};
use crate::ticker::Ticker;
use async_trait::async_trait;
use chrono::prelude::*;
use serde::Serialize;
use xactor::{message, Actor, Addr, Context, Handler};
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
}
pub struct PerformanceActor<T: Handler<Output<PerformanceIndicators>>> {
    addr: Addr<T>,
}

impl<T: Handler<Output<PerformanceIndicators>>> PerformanceActor<T> {
    pub fn of(addr: Addr<T>) -> Self {
        PerformanceActor { addr }
    }
}
impl<T: Handler<Output<PerformanceIndicators>>> Actor for PerformanceActor<T> {}

#[async_trait]
impl<T: Handler<Output<PerformanceIndicators>>> Handler<PerformanceData> for PerformanceActor<T> {
    // TODO: Make result type, since we don't want to drop performance data
    // TODO: Remove all the unwraps.
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceData) -> () {
        let performance_indicators =
            PerformanceIndicators::create(msg.window, &msg.performance_data, msg.ticker, msg.to);
        self.addr.send(Output::of(performance_indicators)).unwrap();
    }
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PerformanceIndicators {
    ticker: Ticker,
    time: DateTime<Utc>,
    min: Option<Price>,
    max: Option<Price>,
    n_window_sma: Option<Price>,
    percentage_change: Option<Percentage>,
    abs_change: Option<Price>,
}

fn min(series: &[f64]) -> Option<f64> {
    get_extreme(series, |v, mv| v < mv)
}
fn max(series: &[f64]) -> Option<f64> {
    get_extreme(series, |v, mv| v > mv)
}

fn get_extreme(series: &[f64], comparator: fn(&f64, &f64) -> bool) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        let mut min_value = series.first()?;
        for value in series.iter() {
            if comparator(value, min_value) {
                min_value = value;
            }
        }
        Some(*min_value)
    }
}

pub fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if n > series.len() || n == 0 {
        return None;
    }
    let mut window: f64 = series.iter().take(n).sum();
    let mut index = 0;
    let window_size: f64 = n as f64;
    let mut sma = vec![window / window_size];
    let mut series_iterator = series.iter();
    for _ in 0..n {
        series_iterator.next();
    }
    for next_value in series_iterator {
        window += next_value - series[index];
        index += 1;
        sma.push(window / window_size);
    }
    Some(sma)
}

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    if series.len() < 2 {
        return None;
    }

    let first = series.first()?;
    let second = series.last()?;

    Some(((second / first) * 100_f64, second - first))
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
            min: min(series).map(Price),
            max: max(series).map(Price),
            n_window_sma: n_window_sma(window, series)
                .and_then(|vec| vec.into_iter().map(Price).last()),
            percentage_change,
            abs_change,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::output_actor::Output;
    use crate::formatter::*;
    use crate::ticker::Ticker;
    use async_std;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use xactor::{Actor, Context, Handler};

    struct MockOutputActor {
        received_messages: Arc<Mutex<Vec<PerformanceIndicators>>>,
    }

    impl MockOutputActor {
        fn of(buf: Arc<Mutex<Vec<PerformanceIndicators>>>) -> Self {
            MockOutputActor {
                received_messages: buf,
            }
        }
    }
    impl Actor for MockOutputActor {}

    #[async_trait]
    impl Handler<Output<PerformanceIndicators>> for MockOutputActor {
        async fn handle(
            &mut self,
            _ctx: &mut Context<Self>,
            msg: Output<PerformanceIndicators>,
        ) -> () {
            let mut data = self.received_messages.lock().unwrap();
            data.push(msg.to_inner().to_owned())
        }
    }

    #[async_std::test]
    async fn performance_actor_messages_with_performance_indicators_when_series_is_not_empty() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let mock_actor = MockOutputActor::of(buffer.clone());
        let mut mock_actor_addr = mock_actor.start().await.unwrap();

        let performance_actor = PerformanceActor::of(mock_actor_addr.clone());
        let mut addr = performance_actor.start().await.unwrap();

        let ticker = Ticker(String::from("test"));
        let series = [15f64, 13f64, 2f64, 7.5f64];
        let time = Utc::now();
        let expected = PerformanceIndicators {
            ticker: ticker.clone(),
            time,
            min: Some(Price(2f64)),
            max: Some(Price(15f64)),
            n_window_sma: Some(Price(4.75f64)),
            percentage_change: Some(Percentage(50f64)),
            abs_change: Some(Price(-7.5f64)),
        };

        let performance_data = PerformanceData {
            ticker,
            window: 2,
            performance_data: Vec::from(series),
            to: time,
        };

        addr.call(performance_data).await.unwrap();

        addr.stop(None).unwrap();
        mock_actor_addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        mock_actor_addr.wait_for_stop().await;
        let received_messages = buffer.lock().unwrap().clone();
        assert_eq!(received_messages.into_iter().nth(0).unwrap(), expected);
    }

    #[test]
    fn min_returns_none_on_empty_list() {
        assert_eq!(min(&[]), None);
    }

    #[test]
    fn min_returns_minimum_value_on_non_empty_list() {
        assert_eq!(min(&[3f64, 2f64, 14f64]), Some(2f64));
    }

    #[test]
    fn max_returns_none_on_empty_list() {
        assert_eq!(max(&[]), None);
    }

    #[test]
    fn max_returns_max_value_on_non_empty_list() {
        assert_eq!(max(&[3f64, 1f64, 14f64, 2f64]), Some(14f64));
    }

    #[test]
    fn n_window_sma_returns_none_if_n_is_0() {
        assert_eq!(n_window_sma(0, &[1f64]), None);
    }

    #[test]
    fn n_window_sma_returns_none_if_n_is_greater_than_series() {
        assert_eq!(n_window_sma(15, &[1f64]), None)
    }

    #[test]
    fn n_window_sma_returns_correct_simple_moving_average() {
        let series = [15f64, 13f64, 2f64, 11f64];
        let expected_sma = vec![14f64, 7.5f64, 6.5f64];
        assert_eq!(n_window_sma(2, &series), Some(expected_sma));
    }

    #[test]
    fn price_diff_returns_none_if_series_is_smaller_than_2() {
        assert_eq!(price_diff(&[1f64]), None);
    }

    #[test]
    fn price_diff_returns_correct_abs_and_percentage_diff_on_positive_change() {
        let series = [16f64, 3f64, 32f64];
        let expected = (200f64, 16f64);
        assert_eq!(price_diff(&series), Some(expected));
    }

    #[test]
    fn price_diff_returns_correct_abs_and_percentage_diff_on_negative_change() {
        let series = [16f64, 3f64, 0f64];
        let expected = (0f64, -16f64);
        assert_eq!(price_diff(&series), Some(expected));
    }
}
