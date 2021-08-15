use crate::actors::{
    messages::{Fetch, PerformanceIndicators},
    FetchActor, InMemoryQuoteWriter, OutputActor, PerformanceActor,
};
use anyhow::Result;
use bounded_vec_deque::BoundedVecDeque;
use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};
use xactor::{Broker, Service, Supervisor};
mod actors;
mod api;
mod config;
mod read_optimised_circular_buffer;

static MAX_API_BUFFER_SIZE: usize = 1023;
#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let config = config::Config::new()?;

    let pth = config.file.clone();
    let output_actor_addr = Supervisor::start(move || {
        let writer: Box<dyn Write + Send> = match &pth {
            Some(pth) => File::create(pth).map(BufWriter::new).map(Box::new).unwrap(),
            None => Box::new(std::io::stdout()),
        };
        let output_actor: OutputActor<_, PerformanceIndicators> = OutputActor::new(writer);
        output_actor
    })
    .await?;

    let read_optimised_in_memory_store =
        Arc::new(RwLock::new(BoundedVecDeque::new(MAX_API_BUFFER_SIZE)));

    let route = api::get_n_indicators(read_optimised_in_memory_store.clone());

    let _deque_actor_addr =
        Supervisor::start(move || InMemoryQuoteWriter::new(read_optimised_in_memory_store.clone()))
            .await?;

    let broker = Broker::from_registry().await?;

    let performance_actor_addr =
        Supervisor::start(move || PerformanceActor::new(broker.clone())).await?;

    let fetch_actor_addr = Supervisor::start(move || {
        let provider = yahoo_finance_api::YahooConnector::new();
        FetchActor::new(
            performance_actor_addr.clone(),
            provider,
            config.tickers.clone(),
            config.from,
        )
    })
    .await?;

    fetch_actor_addr.call(Fetch::new()).await?;

    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
    output_actor_addr.wait_for_stop().await;
    Ok(())
}
