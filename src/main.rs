#![allow(proc_macro_derive_resolution_fallback)] // Diesel compilation warning, should be fixed in diesel 1.4 TODO update diesel to 1.4 when released and remove this configuration

// Http client
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate reqwest;
extern crate frunk;
extern crate rand;
extern crate core;

// Logger
#[macro_use]
extern crate log;
extern crate env_logger;

// Actors
extern crate actix;
extern crate actix_web;
extern crate tokio;

// Database
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate uuid;
extern crate r2d2;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod eventbrite;
mod lottery;
mod web;
mod lotterycache;
mod cache_loop;
mod schema;
mod database;
mod basics;

use failure::Error;
use std::env;
use actix::{System, Arbiter};
use web::WebState;


#[derive(Fail, Debug)]
pub enum LotteryError {
    #[fail(display = "Invalid parameter")]
    InvalidParameter,
    #[fail(display = "No event available")]
    NoEventAvailable,
    #[fail(display = "Error during attendees draw")]
    DrawError { cause: Error },
    #[fail(display = "Unexpected error")]
    UnexpectedError { cause: Error },
}

fn main() {
    let organizer = env::var("ORGANIZER_TOKEN").expect("ORGANIZER_TOKEN is mandatory");
    let token = env::var("EVENTBRITE_TOKEN").expect("EVENTBRITE_TOKEN is mandatory");
    let http_bind = env::var("HTTP_BIND").unwrap_or("0.0.0.0".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or("8088".to_string());

    let system = System::new("lottery");

    web::http_server(WebState{organizer: organizer, token: token}, http_bind, http_port);

    system.run();

}
