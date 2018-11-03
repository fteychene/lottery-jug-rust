use actix::prelude::Addr;
use tokio::timer::Interval;
use std::time::Instant;
use tokio::prelude::future::Future;
use tokio::prelude::Stream;
use core::time::Duration;

use lotterycache::{LotteryCache, UpdateAttendees, UpdateAttendeesResponse};


pub fn cache_update_interval(duration: u64, addr: Addr<LotteryCache>, token: String, organizer: String) -> impl Future<Item=(), Error=()> + 'static {
    Interval::new(Instant::now(), Duration::from_secs(duration))
        .then(move |_instant| addr.send(UpdateAttendees { token: token.clone(), organizer: organizer.clone() })
            .map_err(|err| error!("Error on sending update message : {:?}", err)))
        .for_each(move |res| {
            match res {
                UpdateAttendeesResponse::Updated => info!("Attendees cache updated"),
                UpdateAttendeesResponse::NoEventAvailable => info!("No event available on eventbrite"),
                UpdateAttendeesResponse::EventbriteError { error: ref e } => info!("Error on eventbrite : {}", e),
                UpdateAttendeesResponse::UnexpectedError { error: ref e } => error!("Unexpected error on update attendees \n{:?}", e)
            };
            Ok(())
        })
}