use eventbrite::{Event, Profile, EventbriteError, load_attendees, get_current_event};
use lottery::draw;
use actix::{Actor, Context, Message, Handler, Addr};
use actix::dev::{MessageResponse, ResponseChannel};
use LotteryError;

pub struct LotteryCache {
    attendees: Option<Vec<Profile>>,
    event: Option<Event>,
}

//Messages
pub struct UpdateAttendees {
    pub organizer: String,
    pub token: String,
}

pub enum UpdateAttendeesResponse {
    Updated,
    NoEventAvailable,
    EventbriteError {
        error: EventbriteError
    },
    UnexpectedError {
        error: failure::Error
    },
}

pub struct GetAttendees {
    pub nb: i8
}

pub struct GetEvent {}

// Actor impl
impl Actor for LotteryCache {
    type Context = Context<Self>;
}

impl Default for LotteryCache {
    fn default() -> Self {
        LotteryCache { attendees: None, event: None }
    }
}

impl Message for UpdateAttendees {
    type Result = UpdateAttendeesResponse;
}


impl<A, M> MessageResponse<A, M> for UpdateAttendeesResponse
    where
        A: Actor,
        M: Message<Result=UpdateAttendeesResponse>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

impl Handler<UpdateAttendees> for LotteryCache {
    type Result = UpdateAttendeesResponse;

    fn handle(&mut self, msg: UpdateAttendees, _ctx: &mut Context<Self>) -> Self::Result {
        let load_attendees = get_current_event(&msg.organizer, &msg.token)
            .and_then(|event| load_attendees(&event.id, &msg.token).map(|attendees| (event, attendees)));
        match load_attendees {
            Ok((event, attendees)) => {
                self.attendees = Some(attendees);
                self.event = Some(event);
                UpdateAttendeesResponse::Updated
            }
            Err(e) => {
                self.attendees = None;
                match e.downcast::<EventbriteError>() {
                    Ok(error) => match error {
                        EventbriteError::NoEventAvailable => UpdateAttendeesResponse::NoEventAvailable,
                        other_eventbrite_error => UpdateAttendeesResponse::EventbriteError { error: other_eventbrite_error }
                    },
                    Err(error) => UpdateAttendeesResponse::UnexpectedError { error: error }
                }
            }
        }
    }
}

impl Message for GetAttendees {
    type Result = Result<Vec<Profile>, LotteryError>;
}

impl Handler<GetAttendees> for LotteryCache {
    type Result = Result<Vec<Profile>, LotteryError>;

    fn handle(&mut self, msg: GetAttendees, _ctx: &mut Context<Self>) -> Self::Result {
        self.attendees.as_ref()
            .ok_or(LotteryError::NoEventAvailable)
            .and_then(|ref attendees| draw(msg.nb, attendees).map_err(|error| LotteryError::DrawError { cause: error }))
            .map(|attendees| attendees.into_iter().map(|r| r.clone()).collect())
    }
}

impl Message for GetEvent {
    type Result = Result<Event, LotteryError>;
}

impl Handler<GetEvent> for LotteryCache {
    type Result = Result<Event, LotteryError>;

    fn handle(&mut self, _msg: GetEvent, _ctx: &mut Context<Self>) -> Self::Result {
        self.event.as_ref()
            .ok_or(LotteryError::NoEventAvailable)
            .map(|event| event.clone())
    }
}

pub fn start_cache() -> Addr<LotteryCache> {
    LotteryCache::default().start()
}