use anyhow::Result;
use xactor::Actor;
mod actors;
mod config;
mod formatter;
mod ticker;

// fn main() {}

#[async_std::main]
async fn main() -> Result<()> {
    let config = config::Config::new()?;

    let output_actor = actors::output_actor::OutputActor::of(std::io::stdout());
    let output_actor_addr = output_actor.start().await?;

    let performance_actor =
        actors::performance_actor::PerformanceActor::of(output_actor_addr.clone());
    let performance_actor_addr = performance_actor.start().await?;

    let provider = yahoo_finance_api::YahooConnector::new();
    let fetch_actor = actors::fetch_actor::FetchActor::of(
        performance_actor_addr,
        provider,
        config.tickers,
        config.from,
    );
    let fetch_actor_addr = fetch_actor.start().await?;

    fetch_actor_addr
        .call(actors::fetch_actor::Fetch::new())
        .await?;
    output_actor_addr.wait_for_stop().await;
    Ok(())
}
