use crate::actors::output_actor::Output;
use lib::formatter::{Percentage, Price};
use lib::ticker::Ticker;
use lib::performance_indicators::*;
use async_trait::async_trait;
use chrono::prelude::*;
use log::error;
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
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceData) -> () {
        let performance_indicators =
            PerformanceIndicators::create(msg.window, &msg.performance_data, msg.ticker, msg.to);
        if let Err(e) = self.addr.send(Output::of(performance_indicators)) {
            error!("Failed to send performance indicators: {:?}", e);
        }
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
    use lib::formatter::{Price, Percentage};
    use lib::ticker::Ticker;
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
}
