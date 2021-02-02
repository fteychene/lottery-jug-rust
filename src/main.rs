use anyhow::Error;
use std::env;
use crate::eventbrite::first_event;

mod eventbrite;
mod lottery;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let organizer = env::var("ORGANIZER_TOKEN").expect("ORGANIZER_TOKEN is mandatory");
    let token = env::var("EVENTBRITE_TOKEN").expect("EVENTBRITE_TOKEN is mandatory");
    let response = eventbrite::load_events(&organizer, &token).await?;
    let event = first_event(response.events)?;

    let attendees = eventbrite::load_attendees(&event.id, &token).await?;
    let result = lottery::draw(3, &attendees)?;
    println!("{} {:?}", result.len(), result);
    Ok(())
}