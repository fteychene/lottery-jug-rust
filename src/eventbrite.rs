use anyhow::{Error, anyhow, Context};
use serde::{Deserialize, Serialize};


const EVENTBRITE_BASE_URL: &'static str = "https://www.eventbriteapi.com";

#[derive(Deserialize, Debug, Clone)]
pub struct Pagination {
    pub object_count: u8,
    pub page_count: u8,
    pub page_size: u8,
    pub page_number: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Profile {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Attendee {
    pub profile: Profile
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttendeesResponse {
    pub attendees: Vec<Attendee>,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    pub id: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct EventsResponse {
    pub events: Vec<Event>
}

fn events_url(organizer: &str, token: &str) -> String {
    format!("{base_url}/v3/organizations/{organizer}/events/?status=live&order_by=start_desc&token={token}", base_url = EVENTBRITE_BASE_URL, organizer = organizer, token = token)
}

pub async fn load_events(organizer: &str, token: &str) -> Result<EventsResponse, Error> {
    let events = reqwest::get(&events_url(organizer, token)).await?
        .error_for_status()?
        .json().await?;
    Ok(events)
}

pub fn first_event(events: Vec<Event>) -> Result<Event, Error> {
    events.first()
        .ok_or(anyhow!("No event available currently"))
        .map(|e| e.clone())
}

fn attendees_url(event_id: &str, token: &str, page_id: u8) -> String {
    format!("{base_url}/v3/events/{event_id}/attendees/?token={token}&page={page}", base_url = EVENTBRITE_BASE_URL, event_id = event_id, token = token, page = page_id)
}

async fn fetch_attendees_page(event_id: &str, token: &str, page_id: u8) -> Result<AttendeesResponse, Error> {
    reqwest::get(&attendees_url(event_id, token, page_id)).await?
        .error_for_status()?
        .json::<AttendeesResponse>().await
        .context(format!("Error calling eventbrite for attendees for page {}", page_id))
}

pub async fn load_attendees(event_id: &str, token: &str) -> Result<Vec<Profile>, Error> {
    let attendees = fetch_attendees_page(event_id, token, 1).await?;
    let mut paginating = futures::future::join_all(
        (attendees.pagination.page_number..attendees.pagination.page_count).map(|page| fetch_attendees_page(event_id, token, page + 1))).await // Page + 1 because Eventbrite seems to think that 0 == 1. LOL
        .into_iter().collect::<Result<Vec<AttendeesResponse>, Error>>()?; // Vec<Result<T, E>> => Result<Vec<T>, E> sequence as collect :heart_eyes:
    paginating.insert(0, attendees);
    Ok(paginating.into_iter()
        .flat_map(|response| response.attendees)
        .map(|attendee| attendee.profile)
        .collect())
}
