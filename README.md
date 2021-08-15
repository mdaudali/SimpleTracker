# Simple Tracker from Manning's "building a stock tracking cli with async streams in rust" LiveProject
The LiveProject specifies 4 milestones, and the measures of success for each one. Alongside each milestone is a number of useful reading links to help achieve the milestone.

To view the code at a given milestone, find the `milestone-X` tag within this repository.

All the code is my own - as my first Rust project, it's likely there is significant improvement to be had, particularly around Rust specific idioms and styles.
If you happen to stumble upon this repository and notice any potential code improvements, please don't hesitate to file an issue / PR (won't merge)! I'd love to learn from those more experienced.

Thanks!

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
    -o, --output <FILE>                File to output CSV data to
    -f, --from <FROM>                  Start date to load data from
    -t, --ticker <TICKER>...           Loads the stock data for the provided ticker
    -i, --ticker-file <TICKER FILE>    Loads a comma delimited file of tickers
```

From `cargo run`:
`cargo run -- --ticker MSFT GOOG AAPL UBER IBM --from 2020-01-01T00:00:00Z -o milestone-4.csv` or
`cargo run -- --ticker-file sp500.txt --from 2020-01-01T00:00:00Z -o milestone-4.csv`


As binary:
`SimpleTracker.exe --ticker MSFT GOOG AAPL UBER IBM --from 2020-01-01T00:00:00Z -o milestone-4.csv` or
`SimpleTracker.exe --ticker-file sp500.txt --from 2020-01-01T00:00:00Z -o milestone-4.csv`


Example output:
```
period_start,symbol,price,change,min,max,thirty_day_average
2020-01-01T00:00:00Z,MSFT,$286.51,181.10%,$133.75,$289.67,$279.27
2020-01-01T00:00:00Z,GOOG,$2720.57,198.96%,$1056.62,$2792.89,$2630.88
2020-01-01T00:00:00Z,AAPL,$146.95,198.03%,$55.57,$149.15,$143.20
2020-01-01T00:00:00Z,UBER,$41.81,134.91%,$14.82,$63.18,$47.64
2020-01-01T00:00:00Z,IBM,$142.76,113.69%,$88.80,$151.28,$142.07
```

### Cross compiling:
Full bin cross compilation coming soon!
Targets: x86_64-unknown-linux-musl
Run:
```bash
rustup target add x86_64-unknown-linux-musl;
cd lib;
cargo build --release --target=x86_64-unknown-linux-musl;
```

### Powershell Invoke-WebRequest:
```powershell
curl -uri http://127.0.0.1:3030/tail/5 | Select-Object -Expand Content | ConvertFrom-Json | ConvertTo-Json
```

Sample output:
```json
{
    "value":  [
                  {
                      "ticker":  "WAT",
                      "time":  "2021-08-15T16:23:16.474494600Z",
                      "current_price":  "$405.27",
                      "min":  "$162.36",
                      "max":  "$405.27",
                      "n_window_sma":  "$377.68",
                      "percentage_change":  "172.41%",
                      "abs_change":  "$170.21"
                  },
                  {
                      "ticker":  "WAB",
                      "time":  "2021-08-15T16:23:16.474494600Z",
                      "current_price":  "$88.91",
                      "min":  "$40.34",
                      "max":  "$89.37",
                      "n_window_sma":  "$83.26",
                      "percentage_change":  "111.10%",
                      "abs_change":  "$8.89"
                  },
                  {
                      "ticker":  "UHS",
                      "time":  "2021-08-15T16:23:16.474494600Z",
                      "current_price":  "$149.39",
                      "min":  "$67.50",
                      "max":  "$160.62",
                      "n_window_sma":  "$152.75",
                      "percentage_change":  "105.43%",
                      "abs_change":  "$7.69"
                  },
                  {
                      "ticker":  "WMT",
                      "time":  "2021-08-15T16:23:16.474494600Z",
                      "current_price":  "$149.53",
                      "min":  "$101.15",
                      "max":  "$150.45",
                      "n_window_sma":  "$142.09",
                      "percentage_change":  "129.33%",
                      "abs_change":  "$33.91"
                  },
                  {
                      "ticker":  "ZBH",
                      "time":  "2021-08-15T16:23:16.474494600Z",
                      "current_price":  "$146.09",
                      "min":  "$79.75",
                      "max":  "$178.35",
                      "n_window_sma":  "$156.34",
                      "percentage_change":  "98.99%",
                      "abs_change":  "$-1.49"
                  }
              ],
    "Count":  5
}
```

### Licence
MIT License

Copyright (c) 2021 Mohammed Daudali

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
