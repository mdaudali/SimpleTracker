// use crate::actors::output_actor::Output;
use crate::actors::messages::{PerformanceData, PerformanceIndicators};
use async_trait::async_trait;
use log::error;
use xactor::{Actor, Addr, Broker, Context, Handler};

pub struct PerformanceActor {
    addr: Addr<Broker<PerformanceIndicators>>,
}

impl PerformanceActor {
    pub fn new(addr: Addr<Broker<PerformanceIndicators>>) -> Self {
        PerformanceActor { addr }
    }
}
impl Actor for PerformanceActor {}

#[async_trait]
impl Handler<PerformanceData> for PerformanceActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceData) -> () {
        let performance_indicators = PerformanceIndicators::new(
            msg.window(),
            msg.performance_data(),
            msg.ticker().clone(),
            msg.to(),
        );
        if let Err(e) = self.addr.publish(performance_indicators) {
            error!("Failed to send performance indicators: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std;
    use async_trait::async_trait;
    use chrono::Utc;
    use lib::ticker::Ticker;
    use std::sync::{Arc, Mutex};
    use xactor::{Actor, Broker, Context, Handler, Service};

    struct MockOutputActor {
        received_messages: Arc<Mutex<Vec<PerformanceIndicators>>>,
    }

    impl MockOutputActor {
        fn new(buf: Arc<Mutex<Vec<PerformanceIndicators>>>) -> Self {
            MockOutputActor {
                received_messages: buf,
            }
        }
    }

    #[async_trait]
    impl Actor for MockOutputActor {
        async fn started(&mut self, ctx: &mut Context<Self>) -> anyhow::Result<()> {
            ctx.subscribe::<PerformanceIndicators>().await?;
            Ok(())
        }
    }

    #[async_trait]
    impl Handler<PerformanceIndicators> for MockOutputActor {
        async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceIndicators) -> () {
            let mut data = self.received_messages.lock().unwrap();
            data.push(msg)
        }
    }

    #[async_std::test]
    async fn performance_actor_messages_with_performance_indicators_when_series_is_not_empty() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let mock_actor = MockOutputActor::new(buffer.clone());
        let mut mock_actor_addr = mock_actor.start().await.unwrap();

        let broker = Broker::from_registry().await.unwrap();
        let performance_actor = PerformanceActor::new(broker.clone());
        let mut addr = performance_actor.start().await.unwrap();

        let ticker = Ticker::from("test");
        let series = [15f64, 13f64, 2f64, 7.5f64];
        let time = Utc::now();
        let expected = PerformanceIndicators::new(2, &series, ticker.clone(), time);

        let performance_data = PerformanceData::new(ticker, 2, Vec::from(series), time);

        addr.call(performance_data).await.unwrap();

        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        mock_actor_addr.stop(None).unwrap();

        mock_actor_addr.wait_for_stop().await;
        let received_messages = buffer.lock().unwrap().clone();
        assert_eq!(received_messages.into_iter().nth(0).unwrap(), expected);
    }
}
