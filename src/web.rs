use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Error};
use serde::Deserialize;
use warp::Filter;
use warp::reply::{Json, WithStatus};

use crate::eventbrite::Profile;
use crate::lottery;

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

pub async fn start_web(server_addr: SocketAddr, cache: Arc<RwLock<Option<Vec<Profile>>>>) {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::get()
        .and(warp::path!("hello" / String))
        .map(|name| format!("Hello, {}!", name));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec![http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        .allow_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE]);

    // GET /winners?nb=3 => 200 OK with Json body
    let winners = warp::get()
        .and(warp::path("winners"))
        .and(warp::query::<WinnerQuery>())
        .and(with_cache(cache))
        .and_then(winners)
        .with(cors);

    warp::serve(winners.or(hello))
        .run(server_addr).await;
}