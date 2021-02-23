use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{anyhow, Error};
use log::{info, warn};
use futures::StreamExt;
use tokio_stream::wrappers::IntervalStream;

use crate::eventbrite::{first_event, load_attendees, load_events, Profile};

async fn update_cache(cache: Arc<RwLock<Option<Vec<Profile>>>>, organizer: String, token: String) -> Result<(), Error> {
    let response = load_events(&organizer, &token).await?;
    let result = match first_event(response.events) {
        Some(event) => {
            let attendees = load_attendees(&event.id, &token).await?;
            Some(attendees)
        },
        None => None
    };
    let mut cache = cache.write()
        .map_err(|e| {
            warn!("Lock error : {:?}", e);
            anyhow!("Error locking cache for writing")
        })?;
    *cache = result;
    Ok(())
}

pub async fn cache_loop(duration: Duration, write_cache: Arc<RwLock<Option<Vec<Profile>>>>, organizer: String, token: String) {
    IntervalStream::new(tokio::time::interval(duration))
        // Async borrowing shenanigan
        .map(move |_| (write_cache.clone(), organizer.clone(), token.clone()))
        .for_each(|(cache, organizer, token)| async {
            info!("Update cache");
            match update_cache(cache, organizer, token).await {
                Ok(()) => info!("Cache updated"),
                Err(e) => warn!("Error updating cache : {:?}", e)
            }
        }).await;
}