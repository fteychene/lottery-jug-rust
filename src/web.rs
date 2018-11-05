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
    pub db: Addr<DbExecutor>,
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
    state.cache.send(GetEvent {})
        .and_then(move |event| {
            let mut winner = winner.into_inner();
            winner.event_id = event.map(|event| event.id).ok();
            state.db.send(winner)
        })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn http_server(state: WebState, http_bind: String, http_port: String) {
    use actix_web::middleware::cors::Cors;
    HttpServer::new(move ||
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .configure(|app| Cors::for_app(app) // <- Construct CORS middleware builder
                .allowed_origin("https://jug-montpellier.github.io/pre-lottery/")
                .allowed_origin("http://localhost/")
                .allowed_origin("http://localhost:3000/")
                .allowed_origin("http://localhost:8080/")
                .allowed_methods(vec!["GET", "POST", "OPTION"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/winners", |r| r.method(http::Method::GET).with(winner_handler))
                .resource("/record", |r| r.method(http::Method::POST).with(record_winner_handler))
                .register()))
        .bind(format!("{}:{}", http_bind, http_port))
        .unwrap()
        .start();
}