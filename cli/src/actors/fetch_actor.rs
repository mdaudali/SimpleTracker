use crate::actors::messages::{Fetch, PerformanceData};
use anyhow::Result;
use async_trait::async_trait;
use chrono::prelude::*;
use futures::{stream, stream::StreamExt};
use lib::ticker::Ticker;
use log::error;
use std::time::Duration;
use xactor::{Actor, Addr, Context, Handler};
use yahoo_finance_api::{YResponse, YahooConnector, YahooError};

pub struct FetchActor<T: YahooFinanceApi, H: Handler<PerformanceData>> {
    sender: Addr<H>,
    yahoo_api: T,
    tickers: Vec<Ticker>,
    from: DateTime<Utc>,
}

impl<T: YahooFinanceApi, H: Handler<PerformanceData>> FetchActor<T, H> {
    pub fn new(sender: Addr<H>, yahoo_api: T, tickers: Vec<Ticker>, from: DateTime<Utc>) -> Self {
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
        let until = msg.until();
        let sender = &self.sender;
        stream::iter(self.tickers.clone())
            .for_each_concurrent(None, |ticker| async move {
                let quotes = match provider
                    .get_quote_history(ticker.as_str(), from, until)
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
                let performance_data = PerformanceData::new(ticker, 30, series, until);
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
    use crate::actors::messages::{Fetch, PerformanceData};
    use async_std;
    use async_trait::async_trait;
    use lib::ticker::Ticker;
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
        fn new(buf: Arc<Mutex<Vec<PerformanceData>>>) -> Self {
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

    async fn create_buf_and_actors_and_call_actor_with(
        tickers: Vec<Ticker>,
        fetch: Fetch,
    ) -> Vec<PerformanceData> {
        let buf = Arc::new(Mutex::new(vec![]));
        let mock_performance_data_actor = MockPerformanceDataActor::new(buf.clone());
        let mut mock_performance_data_actor_addr =
            mock_performance_data_actor.start().await.unwrap();

        let mock_yahoo_api = MockYahooConnector;
        let fetch_actor = FetchActor::new(
            mock_performance_data_actor_addr.clone(),
            mock_yahoo_api,
            tickers,
            Utc::now(),
        );
        let mut fetch_actor_addr = fetch_actor.start().await.unwrap();

        fetch_actor_addr.call(fetch).await.unwrap();

        fetch_actor_addr.stop(None).unwrap();
        mock_performance_data_actor_addr.stop(None).unwrap();

        fetch_actor_addr.wait_for_stop().await;
        mock_performance_data_actor_addr.wait_for_stop().await;

        let x = buf.lock().unwrap().clone();
        x
    }
    #[async_std::test]
    async fn fetch_actor_returns_quotes_from_initial_time_to_now() {
        let now = Utc::now();
        let sent_messages = create_buf_and_actors_and_call_actor_with(
            vec![Ticker::new("test".to_string())],
            Fetch::from_datetime(now),
        )
        .await;
        let message = sent_messages.into_iter().nth(0).unwrap();

        let expected = PerformanceData::new(
            Ticker::new("test".to_string()),
            30,
            vec![1f64, 2f64, 3f64],
            now,
        );
        assert_eq!(message, expected);
    }

    #[async_std::test]
    async fn fetch_actor_retrieves_multiple_tickers() {
        let sent_messages = create_buf_and_actors_and_call_actor_with(
            vec![
                Ticker::new("test".to_string()),
                Ticker::new("other_test".to_string()),
            ],
            Fetch::new(),
        )
        .await;
        assert_eq!(sent_messages.len(), 2);
    }
}
