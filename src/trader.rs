use super::runner;

#[derive(Debug)]
pub enum StockSignal {
    Buy(String, u64),
    Sell(String, u64)
}

pub trait Trader {
    fn listens_to(&self) -> Vec<String>;
    fn init(&mut self) -> ();
    fn new_tick(&mut self, market_info: &runner::MarketInfo) -> Vec<StockSignal>;
    fn day_end() -> ();
    fn give_dollars(&mut self, amount: u64) -> ();
    fn give_stock(&mut self, symbol: String, qty: u64) -> ();
    fn money_sum(&self) -> u64;
}


pub enum CrossoverTraderAccount {
    Currency(u64),
    Stock(u64)
}

pub struct CrossoverTrader {
    pub prices: Vec<u64>,
    pub account: CrossoverTraderAccount
}

impl CrossoverTrader {
    pub fn new() -> CrossoverTrader {
        return CrossoverTrader{ prices: Vec::new(), account: CrossoverTraderAccount::Currency(0) };
    }
}

impl Trader for CrossoverTrader {
    fn listens_to(&self) -> Vec<String> {
        return vec![String::from("AAPL")];
    }

    fn init(&mut self) -> () {
        self.prices = Vec::new();
    }

    fn new_tick(&mut self, market_info: &runner::MarketInfo) -> Vec<StockSignal> {
        self.prices.push(market_info.bars["AAPL"].close);

        let slow_mean_len = 200;
        let fast_mean_len = 150;

        assert!(slow_mean_len >= fast_mean_len);

        if self.prices.len() < slow_mean_len { return vec![]; }

        let slow_mean = self.prices.iter().rev().take(slow_mean_len).sum::<u64>() / slow_mean_len as u64;
        let fast_mean = self.prices.iter().rev().take(slow_mean_len).sum::<u64>() / slow_mean_len as u64;

        match self.account {
            CrossoverTraderAccount::Currency(dol) => {
                if fast_mean > slow_mean {
                    self.account = CrossoverTraderAccount::Stock(0);
                    return vec![StockSignal::Buy(String::from("AAPL"), dol)];
                }
            }
            CrossoverTraderAccount::Stock(qty) => {
                if fast_mean < slow_mean {
                    self.account = CrossoverTraderAccount::Currency(0);
                    return vec![StockSignal::Sell(String::from("AAPL"), qty)];
                }
            }
        }

        return vec![];
    }

    fn day_end() -> () {
        return ();
    }

    fn give_dollars(&mut self, amount: u64) -> () {
        match self.account {
            CrossoverTraderAccount::Currency(dol) => {
                self.account = CrossoverTraderAccount::Currency(dol + amount);
            }
            _ => { panic!(""); }
        }
    }

    fn give_stock(&mut self, symbol: String, amount: u64) -> () {
        assert!(symbol == String::from("AAPL"));
        match self.account {
            CrossoverTraderAccount::Stock(qty) => {
                self.account = CrossoverTraderAccount::Currency(qty + amount);
            }
            _ => { panic!(""); }
        }
    }

    fn money_sum(&self) -> u64 {
        match self.account {
            CrossoverTraderAccount::Currency(dol) => { return dol; }
            CrossoverTraderAccount::Stock(qty) => { return qty * self.prices[self.prices.len() - 1]; }
        }
    }
}
