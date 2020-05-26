extern crate amqp;
extern crate fallible_iterator;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use std::default::Default;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use amqp::{AMQPError, Basic, Channel, protocol, Session, Table};
use fallible_iterator::FallibleIterator;
use postgres::{Error, NoTls};
use r2d2::{ManageConnection, Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;

const SEPARATOR: char = '|';

// Used for event uris
struct Event {
    uri: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct ProxyTable {
    table_name: String,
    exchange_uri: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Type {
    Exchange,
    Queue,
}

// Used for channel ids
struct ChannelCounter {
    counter: u16
}

impl ChannelCounter {
    pub fn new() -> ChannelCounter {
        ChannelCounter { counter: 0 }
    }
    pub fn inc(&mut self) -> u16 {
        self.counter += 1;
        self.counter
    }
}

pub fn start(pool: Pool<PostgresConnectionManager<NoTls>>, amqp_uri: &str, proxy_tables: &str, delivery_mode: &u8) -> Result<(), postgres::Error> {
    let mut children = Vec::new();
    let mut pg_conn = pool.get().unwrap();

    for proxy_table in parse_proxy_tables(proxy_tables) {
        let select_exchange_ui_command = format!("SELECT t.{} FROM public.{} t", proxy_table.exchange_uri, proxy_table.table_name);

        for row in pg_conn.query(select_exchange_ui_command.as_str(), &[])? {
            let event = Event {
                uri: row.get(0)
            };
            children.push(spawn_listener_publisher(pool.get().unwrap(), amqp_uri.parse().unwrap(), event, *delivery_mode));
        }
    }

    for child in children {
        let _ = child.join();
    }

    Ok(())
}

fn parse_proxy_tables(proxy_tables: &str) -> Vec<ProxyTable> {
    let mut tables: Vec<ProxyTable> = Vec::new();

    let strs: Vec<Vec<&str>> = proxy_tables.split(",").map(|s| s.split(":").collect()).collect();
    for i in 0..strs.len() {
        tables.push(ProxyTable {
            table_name: strs[i][0].trim().to_string(),
            exchange_uri: strs[i].get(1).unwrap_or(&"").trim().to_string(),
        });
    }
    let mut cleaned_tables: Vec<ProxyTable> = tables.into_iter().filter(|x| !x.exchange_uri.is_empty() && !x.exchange_uri.is_empty())
        .collect();

    if cleaned_tables.len() == 0 {
        panic!("No proxy tables(e.g. table_name:exchange_uri) specified in \"{}\"", proxy_tables)
    }
    cleaned_tables.sort();
    cleaned_tables
}

fn spawn_listener_publisher(mut pg_conn: PooledConnection<PostgresConnectionManager<NoTls>>,
                            amqp_uri: String, event: Event, delivery_mode: u8) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut channel_counter = ChannelCounter::new();
        let mut session = wait_for_amqp_session(&amqp_uri, event.uri.as_str());

        let amqp_entity_type = match get_amq_entity_type(
            &mut session.open_channel(channel_counter.inc()).unwrap(),
            &mut session.open_channel(channel_counter.inc()).unwrap(),
            &event.uri) {
            None => {
                eprint!("The amqp entity {:?} doesn't exist", event.uri);
                std::process::exit(1);
            }
            Some(typ) => typ
        };

        let mut local_channel = session.open_channel(channel_counter.inc()).unwrap();

        let listen_command = format!("LISTEN evt_{}", event.uri);

        pg_conn.execute(listen_command.as_str(), &[]).unwrap();

        println!("Listening on {}...", event.uri);

        let mut notifications = pg_conn.notifications();
        let mut it = notifications.blocking_iter();

        while let Ok(Some(notification)) = it.next() {
            let (routing_key, message) = parse_notification(&notification.payload());
            let (exchange, key) =
                if amqp_entity_type == Type::Exchange {
                    (event.uri.as_str(), routing_key)
                } else {
                    ("", event.uri.as_str())
                };

            let mut publication = local_channel.basic_publish(
                exchange, key, true, false,
                protocol::basic::BasicProperties { content_type: Some("text".to_string()), delivery_mode: Some(delivery_mode), ..Default::default() },
                message.as_bytes().to_vec());

            // When RMQ connection is lost retry it
            if let Err(e @ AMQPError::IoError(_)) = publication {
                eprint!("{:?}", e);
                session = wait_for_amqp_session(amqp_uri.as_str(), event.uri.as_str());
                local_channel = match get_amq_entity_type(
                    &mut session.open_channel(channel_counter.inc()).unwrap(),
                    &mut session.open_channel(channel_counter.inc()).unwrap(),
                    &event.uri) {
                    None => {
                        eprint!("The amqp entity {:?} doesn't exist", event.uri.as_str());
                        std::process::exit(1);
                    }
                    Some(_) => session.open_channel(channel_counter.inc()).unwrap()
                };
                // Republish message
                publication =
                    local_channel.basic_publish(
                        exchange, key, true, false,
                        protocol::basic::BasicProperties { content_type: Some("text".to_string()), delivery_mode: Some(delivery_mode), ..Default::default() },
                        message.as_bytes().to_vec());
            }

            match publication {
                Ok(_) => {
                    println!("{:?} -> {:?} ( routing_key: {:?}, message: {:?} )",
                          event.uri, amqp_entity_type, routing_key, message);
                }
                Err(e) => eprint!("{:?}", e)
            }
        }
    })
}

pub fn wait_for_amqp_session(amqp_uri: &str, exchange_uri: &str) -> Session {
    println!("Attempting to obtain connection on AMQP server for {} channel..", exchange_uri);
    let mut s = Session::open_url(amqp_uri);
    let mut i = 1;
    while let Err(e) = s {
        println!("{:?}", e);
        let time = Duration::from_secs(i);
        println!("Retrying the AMQP connection for {} channel in {:?} seconds..", exchange_uri, time.as_secs());
        thread::sleep(time);
        s = Session::open_url(amqp_uri);
        i *= 2;
        if i > 32 { i = 1 };
    };
    println!("Connection to AMQP server for {} channel successful", exchange_uri);
    s.unwrap()
}

/*
 * Finds the amqp entity type(Queue or Exchange) using two channels because currently rust-amqp hangs up when
 * doing exchange_declare and queue_declare on the same channel.
 * It does this with amqp "passive" set to true.
*/
fn get_amq_entity_type(queue_channel: &mut Channel, exchange_channel: &mut Channel, amqp_entity: &str) -> Option<Type> {
    let opt_queue_type = queue_channel.queue_declare(amqp_entity.clone(), true, false, false, false, false, Table::new())
        .map(|_| Type::Queue).ok();
    let opt_exchange_type = exchange_channel.exchange_declare(amqp_entity, "", true, false, false, false, false, Table::new())
        .map(|_| Type::Exchange).ok();
    queue_channel.close(200, "").unwrap();
    // Somehow the exchange channel is not being closed, a solution could be to close session and reopen
    // However when doing that some error messages(they don't seem to affect the bridge) are shown and that could be confusing for the user
    exchange_channel.close(200, "").unwrap();
    opt_exchange_type.or(opt_queue_type)
}

fn parse_notification(payload: &str) -> (&str, &str){
    let v: Vec<&str> = payload.splitn(2, SEPARATOR).map(|x| x.trim()).collect();
    if v.len() > 1 {
        (v[0], v[1])
    } else {
        ("", v[0])
    }
}


