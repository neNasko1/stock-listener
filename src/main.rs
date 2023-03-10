#![allow(clippy::let_unit_value)]

use std::env::args;

pub mod trader;
pub mod runner;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();

    assert!(args.len() >= 4);

    match args[3].as_str() {
        "watch" => {
            let symbols = vec![
                String::from("AAPL"),
                String::from("NVDA"),
                String::from("MSFT"),
                String::from("AMZN"),
                String::from("META"),
                String::from("GOOGL"),
            ];
            runner::run_watcher(&args[1], &args[2], symbols).await;
        }
        "backtest" => {
            let mut trader = trader::CrossoverTrader::new();
            runner::run_backtest(&args[2], runner::MONEY_SCALING_FACTOR * 1000, &mut trader);
        }
        _ => { panic!("Unknown command type {}", args[1]); }
    }

    return;
}


