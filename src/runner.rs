use apca::data::v2::stream::drive;
use apca::data::v2::stream::MarketData;
use apca::data::v2::stream::RealtimeData;
use apca::data::v2::stream::IEX;
use apca::data::v2::stream::Data;
use apca::Client;
use apca::ApiInfo;

use futures::FutureExt as _;
use futures::StreamExt as _;

use serde_json;

use std::collections::HashMap;
use std::fs;

use sqlite;
use super::trader;

pub const MONEY_SCALING_FACTOR: u64 = 10000;

#[derive(Clone)]
pub struct Bar {
    pub open:   u64,
    pub close:  u64,
    pub high:   u64,
    pub low:    u64,
    pub volume: u64
}

pub struct MarketInfo {
    pub bars: HashMap<String, Bar>
}

impl MarketInfo {
    pub fn new() -> MarketInfo {
        return MarketInfo{bars: HashMap::new()};
    }
}

pub fn prepare_sqlite(database_dir: &str) -> sqlite::Connection {
    let sqlite_connection = sqlite::open(database_dir).unwrap();
    let _ = sqlite_connection.execute(
        "CREATE TABLE bars (
            symbol    TEXT,
            open      INTEGER,
            close     INTEGER,
            low       INTEGER,
            high      INTEGER,
            timestamp TEXT
            );
        "
        );
    return sqlite_connection;
}

pub fn prepare_client(config_json: &str, is_paper: bool) -> Client {
    let file = fs::File::open(config_json)
        .expect("There should be config.json present.");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("config.json should be a valid json.");

    let api_info =
        if is_paper {
            ApiInfo::from_parts(
                "https://paper-api.alpaca.markets",
                json.get("PAPER_APCA_API_KEY_ID").expect("config.json should have paper-alpaca key"),
                json.get("PAPER_APCA_API_SECRET_KEY").expect("config.json should have paper-alpaca secret key")
                )
                .unwrap()
        } else {
            ApiInfo::from_parts(
                "https://api.alpaca.markets",
                json.get("APCA_API_KEY_ID").expect("config.json should have alpaca key"),
                json.get("APCA_API_SECRET_KEY").expect("config.json should have alpaca secret key")
                )
                .unwrap()
        };

    return Client::new(api_info);
}

pub async fn run_watcher(config_json: &str, database_dir: &str, symbols: Vec::<String>) {
    let sqlite_connection = prepare_sqlite(database_dir);
    let client = prepare_client(&config_json, true);

    let (mut stream, mut subscription) = client.subscribe::<RealtimeData<IEX>>().await.unwrap();

    let mut data = MarketData::default();
    data.set_bars(symbols);

    let subscribe = subscription.subscribe(&data).boxed();
    let () = drive(subscribe, &mut stream)
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    while let Some(result) = stream.next().await {
        let data = result.unwrap().unwrap();
        match data {
            Data::Bar(bar) => {
                let query = format!(
                    "INSERT INTO bars VALUES ({}, {}, {}, {}, {}, {}, {})",
                    bar.symbol,
                    bar.open_price .to_f64().map(|x| x * MONEY_SCALING_FACTOR as f64).unwrap() as u64,
                    bar.close_price.to_f64().map(|x| x * MONEY_SCALING_FACTOR as f64).unwrap() as u64,
                    bar.low_price  .to_f64().map(|x| x * MONEY_SCALING_FACTOR as f64).unwrap() as u64,
                    bar.high_price .to_f64().map(|x| x * MONEY_SCALING_FACTOR as f64).unwrap() as u64,
                    bar.volume,
                    bar.timestamp.format("%d/%m/%Y %H:%M")
                    );

                sqlite_connection.execute(query).unwrap();
            }
            _ => { println!("{:?}", data); }
        }
    }
}

pub fn run_backtest(database_dir: &str, starting_funds: u64, tested_trader: &mut impl trader::Trader) -> u64 {
    let sqlite_connection = prepare_sqlite(database_dir);

    tested_trader.give_dollars(starting_funds);

    let mut last_timestamp = String::from("");
    let mut market_info = MarketInfo::new();

    let dollars_symbol: String = String::from("$");
    let mut account = HashMap::new();
    account.insert(dollars_symbol.clone(), starting_funds);
    for listen in tested_trader.listens_to() { account.insert(listen, 0); }

    for row in sqlite_connection
        .prepare("SELECT * FROM bars ORDER BY timestamp;")
            .unwrap()
            .into_iter()
            .map(|row| row.unwrap()) {
                println!("{:?}", row);

                let bar = Bar{
                    open  : row.read::<i64, _>("open")   as u64,
                    close : row.read::<i64, _>("close")  as u64,
                    high  : row.read::<i64, _>("high")   as u64,
                    low   : row.read::<i64, _>("low")    as u64,
                    volume: row.read::<i64, _>("volume") as u64
                };
                let symbol = String::from(row.read::<&str, _>("symbol"));
                let now    = String::from(row.read::<&str, _>("timestamp"));

                market_info.bars.insert(String::from(symbol), bar.clone());

                if last_timestamp == "" || last_timestamp == now { continue; }
                last_timestamp = now;

                let trades = tested_trader.new_tick(&market_info);
                for trade in trades {
                    match trade {
                        // TODO: Possible losses
                        // Urget-TODO: Possible cheating by buying more than you can purchase
                        trader::StockSignal::Buy(stock, dol) => {
                            let qty = dol / bar.close;
                            tested_trader.give_stock(stock.clone(), qty);

                            assert!(account[&dollars_symbol] >= dol);
                            account.insert(stock.clone(), account[&stock] + qty);
                            account.insert(dollars_symbol.clone(), account["$"] - dol);
                        }
                        trader::StockSignal::Sell(stock, qty) => {
                            let dol = qty * bar.close;
                            tested_trader.give_dollars(dol);

                            assert!(account[&stock] >= qty);
                            account.insert(stock.clone(), account[&stock] - qty);
                            account.insert(dollars_symbol.clone(), account["$"] + dol);
                        }
                    }
                }
            }

    return tested_trader.money_sum();
}
