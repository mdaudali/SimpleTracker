# Simple Tracker from Manning's "building a stock tracking cli with async streams in rust" LiveProject

### Getting started
To build:
`cargo build`

To run
```
USAGE:
    simple_tracker.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --from <FROM>                  Start date to load data from
    -t, --ticker <TICKER>...           Loads the stock data for the provided ticker
    -i, --ticker-file <TICKER FILE>    Loads a comma delimited file of tickers
```

From `cargo run`:
`cargo run -- --ticker MSFT GOOG AAPL UBER IBM --from 2020-01-01T00:00:00Z` or
`cargo run -- --ticker-file sp500.txt --from 2020-01-01T00:00:00Z`


As binary:
`SimpleTracker.exe --ticker MSFT GOOG AAPL UBER IBM --from 2020-01-01T00:00:00Z` or
`SimpleTracker.exe --ticker-file sp500.txt --from 2020-01-01T00:00:00Z`


Example output:
```
period_start,symbol,price,change,min,max,thirty_day_average
2020-01-01T00:00:00Z,MSFT,$286.51,181.10%,$133.75,$289.67,$279.27
2020-01-01T00:00:00Z,GOOG,$2720.57,198.96%,$1056.62,$2792.89,$2630.88
2020-01-01T00:00:00Z,AAPL,$146.95,198.03%,$55.57,$149.15,$143.20
2020-01-01T00:00:00Z,UBER,$41.81,134.91%,$14.82,$63.18,$47.64
2020-01-01T00:00:00Z,IBM,$142.76,113.69%,$88.80,$151.28,$142.07
```

