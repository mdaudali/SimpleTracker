use crate::actors::messages::PerformanceIndicators;
use crate::read_optimised_circular_buffer::ReadOptimisedCircularBuffer;
use async_trait::async_trait;
use xactor::{Actor, Context, Handler};

pub struct InMemoryQuoteWriter {
    store: ReadOptimisedCircularBuffer<PerformanceIndicators>,
}

// TODO: Test subscribe
#[async_trait]
impl Actor for InMemoryQuoteWriter {
    async fn started(&mut self, ctx: &mut Context<Self>) -> anyhow::Result<()> {
        ctx.subscribe::<PerformanceIndicators>().await?;
        Ok(())
    }
}

#[async_trait]
impl Handler<PerformanceIndicators> for InMemoryQuoteWriter {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PerformanceIndicators) -> () {
        let mut r = self.store.write().unwrap();
        r.push_back(msg);
    }
}

impl InMemoryQuoteWriter {
    pub fn new(store: ReadOptimisedCircularBuffer<PerformanceIndicators>) -> Self {
        InMemoryQuoteWriter { store }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::messages::PerformanceIndicators;
    use bounded_vec_deque::BoundedVecDeque;
    use chrono::prelude::*;
    use lib::ticker::Ticker;
    use std::sync::{Arc, RwLock};
    use xactor::Actor;

    #[async_std::test]
    async fn in_memory_quote_writer_writes_data() {
        let store = Arc::new(RwLock::new(BoundedVecDeque::new(10)));
        let actor = InMemoryQuoteWriter::new(store.clone());
        let mut actor_addr = actor.start().await.unwrap();
        let s = PerformanceIndicators::new(20, &vec![], Ticker::from("test"), Utc::now());
        actor_addr.call(s.clone()).await.unwrap();
        actor_addr.stop(None).unwrap();
        actor_addr.wait_for_stop().await;

        let r = store.read().unwrap();
        assert!(r.iter().any(|x| *x == s));
        // Unfortunately, since Rust runs tests in parallel, this test can be affected by tests in performance_actor since performance actor publishes a message
        // that this actor listens to. The quick solution was to check that we're storing the message at all (and really, we could just test that there exists a message)
        // To avoid the clash, we could either decouple this class from PerformanceIndicators and/or decouple the performance actor from broker.
    }
}
