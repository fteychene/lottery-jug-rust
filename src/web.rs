use actix::prelude::Addr;
use actix_web::{App, HttpResponse, FutureResponse, State, AsyncResponder, Query, Json};
use actix_web::{http, error, middleware};
use actix_web::server::HttpServer;
use LotteryError;
use tokio::prelude::{future, Future};
use lotterycache::{GetAttendees, GetEvent, LotteryCache};
use database::{CreateWinner, DbExecutor};

#[derive(Clone)]
pub struct WebState {
    pub cache: Addr<LotteryCache>,
//    pub db: Addr<DbExecutor>,
}

impl error::ResponseError for LotteryError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            LotteryError::InvalidParameter => HttpResponse::new(http::StatusCode::BAD_REQUEST),
            LotteryError::NoEventAvailable => HttpResponse::with_body(http::StatusCode::SERVICE_UNAVAILABLE, "No event available on eventbrite"),
            LotteryError::DrawError { cause: ref e } => HttpResponse::with_body(http::StatusCode::BAD_REQUEST, format!("{}", e)),
            LotteryError::UnexpectedError { cause: ref e } => HttpResponse::with_body(http::StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))
        }
    }
}

#[derive(Deserialize)]
struct WinnerQuery {
    nb: i8
}

fn winner_handler((state, query): (State<WebState>, Query<WinnerQuery>)) -> FutureResponse<HttpResponse, LotteryError> {
    match query.nb {
        nb if nb < 0 => Box::new(future::err(LotteryError::InvalidParameter)),
        _ => state.cache.send(GetAttendees { nb: query.nb })
            .map_err(|error| LotteryError::UnexpectedError { cause: error.into() })
            .and_then(|result| result)
            .and_then(|res| Ok(HttpResponse::Ok().json(res)))
            .responder()
    }
}

/// Async request handler
fn record_winner_handler(
    (winner, state): (Json<CreateWinner>, State<WebState>),
) -> FutureResponse<HttpResponse> {
    unimplemented!()
}

pub fn http_server(state: WebState, http_bind: String, http_port: String) {
    HttpServer::new(move ||
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/winners", |r| r.method(http::Method::GET).with(winner_handler)))
        .bind(format!("{}:{}", http_bind, http_port))
        .unwrap()
        .start();
}