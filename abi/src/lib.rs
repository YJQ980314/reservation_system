mod config;
mod error;
mod pager;
mod pb;
mod types;
mod utils;

pub use config::*;
pub use error::{Error, ReservationConflict, ReservationConflictInfo, ResrvationWindow};
pub use pb::*;
pub use utils::*;
// use sqlx::error::DatabaseError;

pub type ReservationId = i64;
pub type UserId = String;
pub type ResourceId = String;

/// validate the date structure, raise error if invalid
pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

/// valdate and normalize the date structure
pub trait Normalizer: Validator {
    /// caller should call normalize to make sure the data structure is ready to use
    fn normalize(&mut self) -> Result<(), Error> {
        self.validate()?;
        self.do_normalize();
        Ok(())
    }

    /// user shall implement do_normalize() to normalize the data structure
    fn do_normalize(&mut self);
}

/// datdbase equivalent of the "reservation_status" enum
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), Error> {
        if *self <= 0 {
            Err(Error::InvalidReservationId(*self))
        } else {
            Ok(())
        }
    }
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}
