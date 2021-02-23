use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Error;
use simple_logger::SimpleLogger;

mod eventbrite;
mod lottery;
mod cache_loop;
mod web;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init().expect("Fail to initialize logger");
    let organizer = env::var("ORGANIZER_TOKEN").expect("ORGANIZER_TOKEN is mandatory");
    let token = env::var("EVENTBRITE_TOKEN").expect("EVENTBRITE_TOKEN is mandatory");
    let http_bind = env::var("HTTP_BIND").unwrap_or("0.0.0.0".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or("8088".to_string());
    let server_addr: SocketAddr = format!("{}:{}", http_bind, http_port).parse().expect("HTTP_BIND and HTTP_PORT doesn't define a valid socket address");

    let cache = Arc::new(RwLock::new(None));

    tokio::spawn(cache_loop::cache_loop(Duration::from_secs(30), cache.clone(), organizer, token));

    web::start_web(server_addr, cache).await;
    Ok(())
}
