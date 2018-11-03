use failure::Error;
use reqwest;
use std::ops::Range;
use frunk::monoid::combine_all;

const EVENTBRITE_BASE_URL: &'static str = "https://www.eventbriteapi.com";

#[derive(Debug, Fail)]
pub enum EventbriteError {
    #[fail(display = "error while loading attendees for event {}", event_id)]
    AttendeesLoadError {
        event_id: String,
        #[cause] cause: Error,
    },
    #[fail(display = "No event available on eventbrite")]
    NoEventAvailable,
}

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
pub struct Attende {
    pub profile: Profile
}

#[derive(Deserialize, Debug, Clone)]
pub struct AttendeesResponse {
    pub attendees: Vec<Attende>,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    pub id: String
}

#[derive(Deserialize, Debug)]
pub struct EventsResponse {
    pub events: Vec<Event>
}


fn events_url(organizer: &str, token: &str) -> String {
    format!("{base_url}/v3/organizations/{organizer}/events/?status=live&order_by=start_desc&token={token}", base_url = EVENTBRITE_BASE_URL, organizer = organizer, token = token)
}

fn load_events(organizer: &str, token: &str) -> Result<EventsResponse, Error> {
    unimplemented!()
}

fn first_event(events: EventsResponse) -> Result<Event, Error> {
    unimplemented!()
}

fn fetch_first_event<F: Fn(&str, &str) -> Result<EventsResponse, Error>>(fetch: F, organizer: &str, token: &str) -> Result<Event, Error> {
    fetch(organizer, token).and_then(first_event)
}

pub fn get_current_event(organizer: &str, token: &str) -> Result<Event, Error> {
    fetch_first_event(load_events, organizer, token)
}

fn attendees_url(event_id: &str, token: &str, page_id: u8) -> String {
    format!("{base_url}/v3/events/{event_id}/attendees/?token={token}&page={page}", base_url = EVENTBRITE_BASE_URL, event_id = event_id, token = token, page = page_id)
}

fn fetch_attendees_page(event_id: &str, token: &str, page: u8) -> Result<AttendeesResponse, Error> {
    unimplemented!()
}

fn fetch_all_attendees<F: Fn(&str, &str, u8) -> Result<AttendeesResponse, Error>>(fetch: F, event_id: &str, token: &str) -> Result<Vec<Profile>, Error> {
    unimplemented!()
}

pub fn load_attendees(event_id: &str, token: &str) -> Result<Vec<Profile>, Error> {
    fetch_all_attendees(fetch_attendees_page, event_id, token)
}

/// Traverse a Vec<Result<T, Error>> and combine the values to return a Result<Vec<T>, Error>
///
/// If all values of the vector are Ok then return a Ok containing all the values cloned
/// On the first Err it stop accumulating values and return the matched error
///
/// T must be a Clone type
fn sequence<T>(seq: Vec<Result<T, Error>>) -> Result<Vec<T>, Error>
    where T: Clone {
    let result = seq.into_iter().fold(Ok(vec![]), |result, current|
        result.and_then(|mut vec|
            match current {
                Ok(value) => {
                    vec.push(value.clone());
                    Ok(vec)
                }
                Err(e) => Err(e)
            }));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attendees_url() {
        assert_eq!(attendees_url("51124390428", "5O5ICDI5I4LUFCAZRSTX", 0), EVENTBRITE_BASE_URL.to_owned() + "/v3/events/51124390428/attendees/?token=5O5ICDI5I4LUFCAZRSTX&page=0");
        assert_eq!(attendees_url("51124390428", "5O5ICDI5I4LUFCAZRSTX", 1), EVENTBRITE_BASE_URL.to_owned() + "/v3/events/51124390428/attendees/?token=5O5ICDI5I4LUFCAZRSTX&page=1");
    }

    #[test]
    fn test_events_url() {
        assert_eq!(events_url("412451CDS", "5O5ICDI5I4LUFCAZRSTX"), EVENTBRITE_BASE_URL.to_owned() + "/v3/organizations/412451CDS/events/?status=live&order_by=start_desc&token=5O5ICDI5I4LUFCAZRSTX");
    }

    #[test]
    fn test_first_event() {
        let response = EventsResponse { events: vec![Event { id: "51124390428".to_string() }] };
        let actual = first_event(response);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), Event { id: "51124390428".to_string() });

        let response = EventsResponse { events: vec![] };
        let actual = first_event(response);
        assert!(actual.is_err());
        matches!(actual.unwrap_err().downcast::<EventbriteError>(), Ok(EventbriteError::NoEventAvailable));

        let response = EventsResponse { events: vec![Event { id: "51124390432".to_string() }, Event { id: "51124390428".to_string() }] };
        let actual = first_event(response);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), Event { id: "51124390432".to_string() });
    }

    #[test]
    fn test_fetch_first_event() {
        use std::io::Error;
        use std::io::ErrorKind;

        let fetch = |_organizer: &str, _token: &str| {
            Ok(EventsResponse { events: vec![Event { id: "51124390428".to_string() }] })
        };
        let actual = fetch_first_event(fetch, "412451CDS", "5O5ICDI5I4LUFCAZRSTX");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), Event { id: "51124390428".to_string() });

        let fetch = |_organizer: &str, _token: &str| {
            Ok(EventsResponse { events: vec![] })
        };
        let actual = fetch_first_event(fetch, "412451CDS", "5O5ICDI5I4LUFCAZRSTX");
        assert!(actual.is_err());
        matches!(actual.unwrap_err().downcast::<EventbriteError>(), Ok(EventbriteError::NoEventAvailable));

        let fetch = |_organizer: &str, _token: &str| {
            Err(Error::new(ErrorKind::ConnectionRefused, "Fake error").into())
        };
        let actual = fetch_first_event(fetch, "412451CDS", "5O5ICDI5I4LUFCAZRSTX");
        assert!(actual.is_err());
        matches!(actual.unwrap_err().downcast::<Error>(), Ok(ref e) if e.kind() == ErrorKind::ConnectionRefused);

        let fetch = |_organizer: &str, _token: &str| {
            Ok(EventsResponse { events: vec![Event { id: "51124390432".to_string() }, Event { id: "51124390428".to_string() }] })
        };
        let actual = fetch_first_event(fetch, "412451CDS", "5O5ICDI5I4LUFCAZRSTX");
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), Event { id: "51124390432".to_string() });
    }

    #[test]
    fn test_sequence() {
        let actual = sequence(vec![Ok(0), Ok(1), Ok(2)]);
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap(), vec![0, 1, 2]);

        let actual = sequence(vec![Ok(0), Ok(1), Err(EventbriteTestError::TestError { page: 2 }.into())]);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<EventbriteTestError>().unwrap(), EventbriteTestError::TestError { page: 2 });

        let actual = sequence(vec![Ok(0), Err(EventbriteTestError::TestError { page: 1 }.into()), Ok(2)]);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<EventbriteTestError>().unwrap(), EventbriteTestError::TestError { page: 1 });

        let actual = sequence(vec![Err(EventbriteTestError::TestError { page: 0 }.into()), Ok(1), Ok(2)]);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<EventbriteTestError>().unwrap(), EventbriteTestError::TestError { page: 0 });

        let actual = sequence(vec![Err(EventbriteTestError::TestError { page: 0 }.into()), Err(EventbriteTestError::TestError { page: 1 }.into()), Ok(2)]);
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<EventbriteTestError>().unwrap(), EventbriteTestError::TestError { page: 0 });
    }

    #[test]
    fn test_fetch_all_attendees() {
        use std::io::Error;
        use std::io::ErrorKind;

        // Right case
        let load_function = |_event_id: &str, _token: &str, _page: u8| {
            Ok(AttendeesResponse {
                attendees: Vec::new(),
                pagination: Pagination {
                    object_count: 0,
                    page_count: 1,
                    page_size: 0,
                    page_number: 0,
                },
            })
        };

        let result = fetch_all_attendees(load_function, "51124390428", "5O5ICDI5I4LUFCAZRSTX");
        assert_eq!(result.unwrap().as_slice(), []);

        // Err on first call
        let load_function = |_event_id: &str, _token: &str, _page: u8| {
            Err(Error::new(ErrorKind::ConnectionRefused, "Fake error").into())
        };

        let result = fetch_all_attendees(load_function, "51124390428", "5O5ICDI5I4LUFCAZRSTX");
        assert!(result.is_err());
        let typed_error = result.unwrap_err().downcast::<EventbriteError>().unwrap();
        match typed_error {
            EventbriteError::AttendeesLoadError { event_id: _, cause } => assert_eq!(cause.downcast::<Error>().unwrap().kind(), ErrorKind::ConnectionRefused),
            _ => assert!(false)
        }

        // Err on pagination loading
        let load_function = |_event_id: &str, _token: &str, page: u8| {
            match page {
                0 => Ok(AttendeesResponse {
                    attendees: Vec::new(),
                    pagination: Pagination {
                        object_count: 0,
                        page_count: 2,
                        page_size: 0,
                        page_number: 0,
                    },
                }),
                _ => Err(Error::new(ErrorKind::ConnectionRefused, "Fake error").into())
            }
        };

        let result = fetch_all_attendees(load_function, "51124390428", "5O5ICDI5I4LUFCAZRSTX");
        assert!(result.is_err());
        let typed_error = result.unwrap_err().downcast::<EventbriteError>().unwrap();
        match typed_error {
            EventbriteError::AttendeesLoadError { event_id: _, cause } => assert_eq!(cause.downcast::<Error>().unwrap().kind(), ErrorKind::ConnectionRefused),
            _ => assert!(false)
        }
    }

    #[derive(Debug, Fail, PartialEq)]
    enum EventbriteTestError {
        #[fail(display = "Unexpected Error for tests")]
        TestError {
            page: u8
        }
    }
}