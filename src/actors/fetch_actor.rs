use crate::actors::performance_actor::PerformanceData;
use crate::ticker::Ticker;
use anyhow::Result;
use async_trait::async_trait;
use chrono::prelude::*;
use futures::{stream, stream::StreamExt};
use log::error;
use std::time::Duration;
use xactor::{message, Actor, Addr, Context, Handler};
use yahoo_finance_api::{YResponse, YahooConnector, YahooError};

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
}

pub struct FetchActor<T: YahooFinanceApi, H: Handler<PerformanceData>> {
    sender: Addr<H>,
    yahoo_api: T,
    tickers: Vec<Ticker>,
    from: DateTime<Utc>,
}

impl<T: YahooFinanceApi, H: Handler<PerformanceData>> FetchActor<T, H> {
    pub fn of(sender: Addr<H>, yahoo_api: T, tickers: Vec<Ticker>, from: DateTime<Utc>) -> Self {
        FetchActor {
            sender,
            yahoo_api,
            tickers,
            from,
        }
    }
}

#[async_trait]
impl<T: YahooFinanceApi + Send + Sync + 'static, H: Handler<PerformanceData>> Actor
    for FetchActor<T, H>
{
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval_with(Fetch::new, Duration::from_secs(30));
        Ok(())
    }
}

#[async_trait]
impl<T: YahooFinanceApi + Send + Sync + 'static, H: Handler<PerformanceData>> Handler<Fetch>
    for FetchActor<T, H>
{
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Fetch) -> () {
        let provider = &self.yahoo_api;
        let from = self.from;
        let to = msg.to;
        let sender = &self.sender;
        stream::iter(self.tickers.clone())
            .for_each_concurrent(None, |ticker| async move {
                let quotes = match provider
                    .get_quote_history(&ticker.0, from, to)
                    .await
                    .and_then(|x| x.quotes())
                {
                    Err(e) => {
                        error!("Failed to retrieve quotes for {:?}: {:?}", ticker, e);
                        return;
                    }
                    Ok(o) => o,
                };
                let series: Vec<f64> = quotes.iter().map(|q| q.adjclose).collect();
                let performance_data = PerformanceData::of(ticker, 30, series, to);
                if let Err(e) = sender.send(performance_data) {
                    error!("Failed to send quotes to actor: {:?}", e)
                }
            })
            .await;
    }
}

#[async_trait]
pub trait YahooFinanceApi {
    async fn get_quote_history(
        &self,
        ticker: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<YResponse, YahooError>;
}

#[async_trait]
impl YahooFinanceApi for YahooConnector {
    async fn get_quote_history(
        &self,
        ticker: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<YResponse, YahooError> {
        self.get_quote_history(ticker, start, end).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::performance_actor::PerformanceData;
    use crate::ticker::Ticker;
    use async_std;
    use async_trait::async_trait;
    use serde_json;
    use std::fs::File;
    use std::io::BufReader;
    use std::sync::{Arc, Mutex};
    use xactor::{Actor, Context, Handler};
    use yahoo_finance_api::YResponse;
    struct MockPerformanceDataActor {
        buf: Arc<Mutex<Vec<PerformanceData>>>,
    }

    impl MockPerformanceDataActor {
        fn of(buf: Arc<Mutex<Vec<PerformanceData>>>) -> Self {
            MockPerformanceDataActor { buf }
        }
    }
    impl Actor for MockPerformanceDataActor {}
    #[async_trait]
    impl Handler<PerformanceData> for MockPerformanceDataActor {
        async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceData) -> () {
            let mut data = self.buf.lock().unwrap();
            data.push(msg);
            ()
        }
    }

    struct MockYahooConnector;

    #[async_trait]
    impl YahooFinanceApi for MockYahooConnector {
        async fn get_quote_history(
            &self,
            _ticker: &str,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> std::result::Result<YResponse, YahooError> {
            let file = File::open(format!(
                "{}/src/mockYahooData.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap();
            let reader = BufReader::new(file);

            let u = serde_json::from_reader(reader).unwrap();
            YResponse::from_json(u)
        }
    }
    #[async_std::test]
    async fn fetch_actor_returns_quotes_from_initial_time_to_now() {
        let buf = Arc::new(Mutex::new(vec![]));
        let mock_performance_data_actor = MockPerformanceDataActor::of(buf.clone());
        let mut mock_performance_data_actor_addr =
            mock_performance_data_actor.start().await.unwrap();

        let mock_yahoo_api = MockYahooConnector;
        let fetch_actor = FetchActor::of(
            mock_performance_data_actor_addr.clone(),
            mock_yahoo_api,
            vec![Ticker("test".to_string())],
            Utc::now(),
        );
        let mut fetch_actor_addr = fetch_actor.start().await.unwrap();

        let to = Utc::now();
        fetch_actor_addr.call(Fetch::of(to)).await.unwrap();

        fetch_actor_addr.stop(None).unwrap();
        mock_performance_data_actor_addr.stop(None).unwrap();

        fetch_actor_addr.wait_for_stop().await;
        mock_performance_data_actor_addr.wait_for_stop().await;

        let sent_messages = buf.lock().unwrap().clone();
        let message = sent_messages.into_iter().nth(0).unwrap();

        let expected =
            PerformanceData::of(Ticker("test".to_string()), 30, vec![1f64, 2f64, 3f64], to);
        assert_eq!(message, expected);
    }
}
