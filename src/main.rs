#![allow(clippy::let_unit_value)]

use apca::data::v2::stream::drive;
use apca::data::v2::stream::MarketData;
use apca::data::v2::stream::RealtimeData;
use apca::data::v2::stream::IEX;
use apca::data::v2::stream::Data::Bar;
use apca::ApiInfo;
use apca::Client;

use futures::FutureExt as _;
use futures::StreamExt as _;

use sqlite;

async fn run_watcher(client: Client, symbols: Vec::<String>, sqlite_connection: sqlite::Connection) {
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
      Bar(bar) => {
        let query = format!(
          "INSERT INTO bars VALUES ({}, {}, {}, {}, {}, {}, {})",
          bar.symbol,
          bar.open_price.to_f64().map(|x| x * 1000.).unwrap() as u64,
          bar.close_price.to_f64().map(|x| x * 1000.).unwrap() as u64,
          bar.low_price.to_f64().map(|x| x * 1000.).unwrap() as u64,
          bar.high_price.to_f64().map(|x| x * 1000.).unwrap() as u64,
          bar.volume,
          bar.timestamp.timestamp_millis()
        );

        sqlite_connection.execute(query).unwrap();
      }
      _ => { println!("{:?}", data); }
    }
  }
}

fn prepare_sqlite(sqlite_connection: &sqlite::Connection) {
  let query = "
    CREATE TABLE bars (symbol TEXT, open INTEGER, close INTEGER, low INTEGER, high INTEGER, timestamp INTEGER);
  ";
  sqlite_connection.execute(query).unwrap();
}

#[tokio::main]
async fn main() {
  let api_info = ApiInfo::from_env().unwrap();
  let client = Client::new(api_info);

  let sqlite_connection = sqlite::open("stocks.db").unwrap();
  // prepare_sqlite(&sqlite_connection);

  let symbols = vec![
    String::from("AAPL"),
    String::from("NVDA"),
    String::from("MSFT"),
    String::from("AMZN"),
    String::from("META"),
    String::from("GOOGL"),
  ];

  run_watcher(client, symbols, sqlite_connection).await;
}
