mod config;
mod error;
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

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
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
