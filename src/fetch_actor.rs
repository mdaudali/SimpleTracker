use xactor::{message, Handler, Actor, Context, Addr};
use async_trait::async_trait;
use anyhow::Result;
use std::time::Duration;
use crate::performance_actor::PerformanceData;
use yahoo_finance_api::{YResponse, YahooError, YahooConnector};
use crate::ticker::Ticker;
use futures::{stream, stream::StreamExt};
use chrono::prelude::*;
#[message]
#[derive(Clone)]
pub struct Fetch;

pub struct FetchActor<T: YahooFinanceApi, H: Handler<PerformanceData>> {
    sender: Addr<H>,
    yahoo_api: T,
    tickers: Vec<Ticker>,
    from: DateTime<Utc>
}

impl <T: YahooFinanceApi, H: Handler<PerformanceData>> FetchActor<T, H> {
    pub fn of(sender: Addr<H>, yahoo_api: T, tickers: Vec<Ticker>, from: DateTime<Utc>) -> Self {
        FetchActor{ sender,  yahoo_api, tickers, from }
    }
}


#[async_trait]
impl <T: YahooFinanceApi + Send + Sync + 'static, H: Handler<PerformanceData>> Actor for FetchActor<T, H> {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval(Fetch, Duration::from_secs(30));
        Ok(())
    }
}

#[async_trait]
impl <T: YahooFinanceApi + Send + Sync + 'static, H: Handler<PerformanceData>> Handler<Fetch> for FetchActor<T, H> {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: Fetch) -> () {
        let provider = &self.yahoo_api;
        let from = self.from;
        let to = Utc::now();
        let sender = &self.sender;
        stream::iter(self.tickers.clone()).for_each_concurrent(None, |ticker| {
            let ticker = ticker.clone();
            async move { 
                let response = provider.get_quote_history(&ticker.0, from, to).await.unwrap();
                let quotes = response.quotes().unwrap();
                let series: Vec<f64> = quotes.iter().map(|q| q.adjclose).collect();
                let performance_data =  PerformanceData::of(ticker, 30, series, to);
                sender.send(performance_data).unwrap();
            }
        }).await;
    }

}

#[async_trait]
pub trait YahooFinanceApi {
    async fn get_quote_history(
        &self,
        ticker: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<YResponse, YahooError>;
}

#[async_trait]
impl YahooFinanceApi for YahooConnector {
    async fn get_quote_history(
        &self,
        ticker: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<YResponse, YahooError> {
        self.get_quote_history(ticker, start, end).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, Arc};
    use std::io::BufReader;
    use std::fs::File;
    use serde_json;
    use async_std;
    use xactor::{Context, Actor, Handler};
    use async_trait::async_trait;
    use crate::performance_actor::PerformanceData;
    use yahoo_finance_api::YResponse;
    use crate::ticker::Ticker;
    struct MockPerformanceDataActor {
        buf: Arc<Mutex<Vec<PerformanceData>>>
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
            _end: DateTime<Utc>
        ) -> std::result::Result<YResponse, YahooError> {
            let file = File::open(format!("{}/src/mockYahooData.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
            let reader = BufReader::new(file);

            let u = serde_json::from_reader(reader).unwrap();
            YResponse::from_json(u)
        }
    }
    #[async_std::test]
    async fn fetch_actor_returns_quotes_from_initial_time_to_now() {
        let buf = Arc::new(Mutex::new(vec![]));
        let mock_performance_data_actor = MockPerformanceDataActor::of(buf.clone());
        let mut mock_performance_data_actor_addr = mock_performance_data_actor.start().await.unwrap();

        let mock_yahoo_api = MockYahooConnector;
        let fetch_actor = FetchActor::of(mock_performance_data_actor_addr.clone(), mock_yahoo_api, vec![Ticker("test".to_string())], Utc::now());
        let mut fetch_actor_addr = fetch_actor.start().await.unwrap();
        fetch_actor_addr.call(Fetch).await.unwrap();

        fetch_actor_addr.stop(None).unwrap();
        mock_performance_data_actor_addr.stop(None).unwrap();

        fetch_actor_addr.wait_for_stop().await;
        mock_performance_data_actor_addr.wait_for_stop().await;


        let sent_messages = buf.lock().unwrap().clone();
        let message = sent_messages.into_iter().nth(0).unwrap();
        assert_eq!(message.ticker(), Ticker("test".to_string()));
        assert_eq!(message.performance_data(), vec![1f64, 2f64, 3f64]);


        // let 

    }
}