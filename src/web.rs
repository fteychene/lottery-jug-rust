use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply};
use warp::reply::{json, with_status};

use crate::eventbrite::{Profile, EventbriteError};
use crate::lottery::{DrawError, draw};

#[derive(Deserialize)]
struct WinnerQuery {
    nb: i8
}

#[derive(Serialize)]
struct ApiError {
    message: String
}

#[derive(Debug, Clone)]
struct Env {
    organizer: String,
    token: String,
}

impl From<&DrawError> for ApiError {
    fn from(e: &DrawError) -> Self {
        match e {
            DrawError::InvalidDrawRequest { asked } =>
                ApiError { message: format!("Invalid nb in request : {}", asked) },
            DrawError::NotEnoughtParticipant { asked, existant } =>
                ApiError { message: format!("Not enough participant, requested {} for {} attendees", asked, existant) }
        }
    }
}

impl From<&EventbriteError> for ApiError {
    fn from(e: &EventbriteError) -> Self {
        match e {
            EventbriteError::AttendeesLoadError { .. } =>
                ApiError { message: "Error loading attendees".to_string() },
            EventbriteError::NoEventAvailable =>
                ApiError { message: "No event available, Please come back later".to_string() },
        }
    }
}

impl From<&Error> for ApiError {
    fn from(e: &Error) -> Self {
        ApiError { message: format!("Server error : {}", e) }
    }
}

fn error_handling(error: Error) -> Box<dyn Reply> {
    error.downcast_ref::<DrawError>()
        .map(|draw_error|
            Box::new(with_status(json(&ApiError::from(draw_error)), warp::http::StatusCode::BAD_REQUEST))
        ).or(
        error.downcast_ref::<EventbriteError>()
            .map(|draw_error|
                Box::new(with_status(json(&ApiError::from(draw_error)), warp::http::StatusCode::BAD_GATEWAY))
            ))
        .unwrap_or_else(||
            Box::new(with_status(json(&ApiError::from(&error)), warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        )
}

async fn winners_handler(query: WinnerQuery, cache: Arc<RwLock<Option<Vec<Profile>>>>) -> Result<Box<dyn Reply>, Error> {
    let cache = cache.read().map_err(|_| anyhow!("Error acquiring cache read lock"))?;
    let attendees = cache.as_ref().ok_or_else(|| anyhow!(EventbriteError::NoEventAvailable))?; // TODO error should be typed
    let result = draw(query.nb, attendees)?;
    Ok(Box::new(with_status(json(&result), warp::http::StatusCode::OK)))
}

async fn winners_route(query: WinnerQuery, cache: Arc<RwLock<Option<Vec<Profile>>>>) -> Result<Box<dyn Reply>, Infallible> {
    Ok(winners_handler(query, cache).await
        .unwrap_or_else(|error| error_handling(error)))
}

// Extract=(X,) => This is some magic with Tuple to simulate HList, see generics.rs in warp to more info
fn with_cache(cache: Arc<RwLock<Option<Vec<Profile>>>>) -> impl Filter<Extract=(Arc<RwLock<Option<Vec<Profile>>>>, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || cache.clone())
}

pub async fn start_web(server_addr: SocketAddr, cache: Arc<RwLock<Option<Vec<Profile>>>>) {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec![http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        .allow_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE]);

    // GET /winners?nb=3 => 200 OK with Json body
    let winners = warp::get()
        .and(warp::path("winners"))
        .and(warp::query::<WinnerQuery>())
        .and(with_cache(cache))
        .and_then(winners_route)
        .with(cors);

    warp::serve(winners)
        .run(server_addr).await;
}