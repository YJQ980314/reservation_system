use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{ReservationQuery, Validator};

use super::{get_timespan, validate_range};

impl ReservationQuery {
    // #[allow(clippy::too_many_arguments)]
    // pub fn new(
    //     uid: impl Into<String>,
    //     rid: impl Into<String>,
    //     start: DateTime<Utc>,
    //     end: DateTime<Utc>,
    //     status: ReservationStatus,
    //     page: i32,
    //     page_size: i32,
    //     desc: bool,
    // ) -> Self {
    //     Self {
    //         user_id: uid.into(),
    //         resource_id: rid.into(),
    //         start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
    //         end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
    //         status: status as i32,
    //         page,
    //         desc,
    //         page_size,
    //     }
    // }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), crate::Error> {
        validate_range(self.start.as_ref(), self.end.as_ref())?;
        Ok(())
    }
}