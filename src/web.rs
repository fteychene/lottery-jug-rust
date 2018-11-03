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
//    pub cache: Addr<LotteryCache>,
//    pub db: Addr<DbExecutor>,
    pub organizer: String,
    pub token: String,
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
    use eventbrite;
    use lottery;
    match query.nb {
        a if a < 0 => Box::new(future::err(LotteryError::InvalidParameter)),
        nb => {
            let result = eventbrite::get_current_event(&state.organizer, &state.token)
                .and_then(|event| eventbrite::load_attendees(&event.id, &state.token))
                .map_err(|err| LotteryError::UnexpectedError { cause: err })
                .and_then(|attendees| lottery::draw(nb, &attendees)
                    .map(|profiles| profiles.into_iter().map(|r| r.clone()).collect::<Vec<eventbrite::Profile>>())
                    .map_err(|error| LotteryError::DrawError { cause: error }))
                .map(|res| HttpResponse::Ok().json(res));
            Box::new(future::result(result))
        }
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