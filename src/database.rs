//! Db executor actor
use actix::prelude::*;
use actix::SyncArbiter;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid;
use schema::winners;
use LotteryError;
use failure::Error;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<SqliteConnection>>);

/// This is only message that this actor can handle, but it is easy to extend
/// number of messages.
#[derive(Serialize, Deserialize)]
pub struct CreateWinner {
    pub first_name: String,
    pub last_name: String,
    pub event_id: Option<String>
}

impl Message for CreateWinner {
    type Result = Result<Winner, Error>;
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<CreateWinner> for DbExecutor {
    type Result = Result<Winner, Error>;

    fn handle(&mut self, msg: CreateWinner, _: &mut Self::Context) -> Self::Result {
        use schema::winners::dsl::*;
        let uuid = format!("{}", uuid::Uuid::new_v4());
        let new_user = NewWinner {
            id: &uuid,
            first_name: &msg.first_name,
            last_name: &msg.last_name,
            event_id: &msg.event_id.unwrap_or("Unknown".to_owned()),
        };

        let conn: &SqliteConnection = &self.0.get().unwrap();

        diesel::insert_into(winners)
            .values(&new_user)
            .execute(conn)
            .map_err(|err| { LotteryError::UnexpectedError { cause: err.into() } })?;

        let mut items = winners
            .filter(id.eq(&uuid))
            .load::<Winner>(conn)
            .map_err(|err| { LotteryError::UnexpectedError { cause: err.into() } })?;

        Ok(items.pop().unwrap())
    }
}


#[derive(Serialize, Queryable, Debug)]
pub struct Winner {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub event_id: String
}

#[derive(Insertable)]
#[table_name = "winners"]
pub struct NewWinner<'a> {
    pub id: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub event_id: &'a str
}

embed_migrations!("migrations");

pub fn start_database(database_url: String) -> Addr<DbExecutor> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    SyncArbiter::start(3, move || DbExecutor(pool.clone()))
}