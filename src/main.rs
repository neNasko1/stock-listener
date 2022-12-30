#![allow(clippy::let_unit_value)]

use std::env::args;

pub mod trader;
pub mod runner;

#[tokio::main]
async fn main() {
  let args: Vec<String> = args().collect();

  if args.len() < 2 {
    panic!("Not enough arguments in cli");
  }

  match args[1].as_str() {
    "watch" => {
      let symbols = vec![
        String::from("AAPL"),
        String::from("NVDA"),
        String::from("MSFT"),
        String::from("AMZN"),
        String::from("META"),
        String::from("GOOGL"),
      ];
      runner::run_watcher(symbols).await;
    }
    "backtest" => {
      let mut trader = trader::CrossoverTrader::new();
      runner::run_backtest(100000, &mut trader);
    }
    _ => {
      panic!("Unknown command type {}", args[1]);
    }
  }

  return;
}
