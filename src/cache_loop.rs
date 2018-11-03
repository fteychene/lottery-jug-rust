use actix::prelude::Addr;
use tokio::timer::Interval;
use std::time::Instant;
use tokio::prelude::future::Future;
use tokio::prelude::Stream;
use core::time::Duration;

use lotterycache::{LotteryCache, UpdateAttendees, UpdateAttendeesResponse};


