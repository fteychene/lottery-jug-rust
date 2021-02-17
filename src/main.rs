use anyhow::{Error, anyhow};
use std::env;
use crate::eventbrite::{first_event, Profile};
use warp::Filter;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use serde::{Deserialize};
use warp::reply::{Json, WithStatus};
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use std::net::SocketAddr;

mod eventbrite;
mod lottery;

#[derive(Deserialize)]
struct WinnerQuery {
    nb: i8
}

#[derive(Debug, Clone)]
struct Env {
    organizer: String,
    token: String,
}

// TODO impl warp::Reply instead of forcing specific type
async fn winners_handler(query: WinnerQuery, cache: Arc<RwLock<Option<Vec<Profile>>>>) -> Result<WithStatus<Json>, Error> {
    let cache = cache.read().map_err(|_| anyhow!("Error acquiring cache read lock"))?;
    let cache = cache.as_ref();
    match cache {
        Some(attendees) => {
            let result = lottery::draw(query.nb, attendees)?;
            Ok(warp::reply::with_status(warp::reply::json(&result), warp::http::StatusCode::OK))
        },
        None => Ok(warp::reply::with_status(warp::reply::json::<Vec<Profile>>(&vec![]), warp::http::StatusCode::BAD_REQUEST))
    }

}

fn as_error_reply(error: Error) -> WithStatus<Json> {
    let reply = warp::reply::json(&format!("ERROR : {:?}", error));
    warp::reply::with_status(reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn winners(query: WinnerQuery, cache: Arc<RwLock<Option<Vec<Profile>>>>) -> Result<WithStatus<Json>, Infallible> {
    Ok(winners_handler(query, cache).await
        .unwrap_or_else(|error| as_error_reply(error)))
}

// TODO How ? WTF It's amazing but how ... Extract=(X, ) Oo Oo Oo
fn with_cache(cache: Arc<RwLock<Option<Vec<Profile>>>>) -> impl Filter<Extract=(Arc<RwLock<Option<Vec<Profile>>>>, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || cache.clone())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let organizer = env::var("ORGANIZER_TOKEN").expect("ORGANIZER_TOKEN is mandatory");
    let token = env::var("EVENTBRITE_TOKEN").expect("EVENTBRITE_TOKEN is mandatory");
    let http_bind = env::var("HTTP_BIND").unwrap_or("0.0.0.0".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or("8088".to_string());
    let server_addr: SocketAddr = format!("{}:{}", http_bind, http_port).parse().expect("HTTP_BIND and HTTP_PORT doesn't define a valid socket address");

    let cache = Arc::new(RwLock::new(None));

    let write_cache = cache.clone();

    // TODO manage unwrap
    tokio::spawn(IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
        .map(move |_| (write_cache.clone(), organizer.clone(), token.clone()))
        .for_each(|(cache, organizer, token) | async move {
            println!("Update cache");
            let response = eventbrite::load_events(&organizer, &token).await.unwrap();
            let event = first_event(response.events).unwrap();
            let attendees = eventbrite::load_attendees(&event.id, &token).await.unwrap();
            let mut cache = cache.write().unwrap();
            *cache = Some(attendees)
        }));

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::get()
        .and(warp::path!("hello" / String))
        .map(|name| format!("Hello, {}!", name));

    // GET /winners?nb=3 => 200 OK with Json body
    let winners = warp::get()
        .and(warp::path("winners"))
        .and(warp::query::<WinnerQuery>())
        .and(with_cache(cache))
        .and_then(winners);

    warp::serve(winners.or(hello))
        .run(server_addr)
        .await;
    Ok(())
}
