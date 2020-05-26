extern crate env_logger;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate kura as proxy;

use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use r2d2::{Pool, ManageConnection};
use r2d2_postgres::{PostgresConnectionManager};
use postgres::{NoTls};

#[derive(Debug, Clone)]
struct Config {
    postgresql_uri: String,
    amqp_uri: String,
    proxy_tables: String,
    delivery_mode: u8,
}

impl Config {
    fn new() -> Config {
        Config {
            postgresql_uri: read_env_with_secret("POSTGRESQL_URI"),
            amqp_uri: read_env_with_secret("AMQP_URI"),
            proxy_tables: env::var("PROXY_TABLES").expect("PROXY_TABLES must be defined"),
            delivery_mode:
            match env::var("DELIVERY_MODE").ok().as_ref().map(String::as_ref) {
                None => 1,
                Some("NON-PERSISTENT") => 1,
                Some("PERSISTENT") => 2,
                Some(_) => panic!("DELIVERY_MODE environment variable can only be PERSISTENT or NON-PERSISTENT")
            },
        }
    }
}

fn read_env_with_secret(key: &str) -> String {
    return match env::var(format!("{}_FILE", key)) {
        Ok(val) => fs::read_to_string(val.clone()).expect(format!("Something went wrong reading {}", val).as_ref()),
        Err(_e) => env::var(key).expect(format!("{} environment variable must be defined", key).as_ref()),
    };
}

fn main() {
    env_logger::init();
    let config = Config::new();

    loop {
        let pool = wait_for_pg_connection(&config.postgresql_uri);
        // This functions spawns threads for each pg channel and waits for the threads to finish,
        // that only occurs when the threads die due to a pg connection error
        // and so if that happens the pg connection is retried and the bridge is started again.
        match proxy::start(pool, &config.amqp_uri, &config.proxy_tables, &config.delivery_mode) {
            Err(e) => eprint!("{:?}", e),
            _ => {}
        }
    }
}

fn wait_for_pg_connection(pg_uri: &String) -> Pool<PostgresConnectionManager<NoTls>> {
    println!("Attempting to connect to PostgreSQL...");
    let manager = PostgresConnectionManager::new(
        pg_uri.parse().unwrap(),
        NoTls,
    );

    let mut i = 1;
    while let Err(e) = manager.connect() {
        println!("{:?}", e);
        let time = Duration::from_secs(i);
        println!("Retrying the PostgreSQL connection in {:?} seconds...", time.as_secs());
        thread::sleep(time);
        i *= 2;
        if i > 32 { i = 1 };
    };

    println!("Connection to PostgreSQL successful");
    Pool::new(manager).unwrap()
}
