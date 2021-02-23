use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{anyhow, Error};
use log::{info, warn};
use futures::StreamExt;
use tokio_stream::wrappers::IntervalStream;

use crate::eventbrite::{first_event, load_attendees, load_events, Profile};

async fn update_cache((cache, organizer, token): (Arc<RwLock<Option<Vec<Profile>>>>, String, String)) -> Result<(), Error> {
    let response = load_events(&organizer, &token).await?;
    let event = first_event(response.events)?;
    let attendees = load_attendees(&event.id, &token).await?;
    let mut cache = cache.write()
        .map_err(|e| {
            warn!("Lock error : {:?}", e);
            anyhow!("Error locking cache for writing")
        })?;
    *cache = Some(attendees);
    Ok(())
}

pub async fn cache_loop(duration: Duration, write_cache: Arc<RwLock<Option<Vec<Profile>>>>, organizer: String, token: String) {
    IntervalStream::new(tokio::time::interval(duration))
        // Async borrowing shenanigan
        .map(move |_| (write_cache.clone(), organizer.clone(), token.clone()))
        .for_each(|params| async {
            info!("Update cache");
            match update_cache(params).await {
                Ok(()) => info!("Cache updated"),
                Err(e) => warn!("Error updating cache : {:?}", e)
            }
        }).await;
}