mod conflict;
use sqlx::postgres::PgDatabaseError;
// use std::io;
use thiserror::Error;

pub use conflict::{ReservationConflict, ReservationConflictInfo, ResrvationWindow};

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    #[error("Database error: {0}")]
    DbError(sqlx::Error),

    #[error("Failed to read configuration file")]
    ConfigReadError,

    #[error("Failed to parse configuration file")]
    ConfigParseError,

    #[error("No reservation found by the given condition")]
    NotFound,

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Invalid reservation id: {0}")]
    InvalidReservationId(i64),

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown data store error")]
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // // TODO: this is not a good way to compare DB errors, but we don't do that in the code
            (Error::DbError(_), Error::DbError(_)) => true,
            (Error::NotFound, Error::NotFound) => true,
            (Error::InvalidTime, Error::InvalidTime) => true,
            (Error::ConflictReservation(v1), Error::ConflictReservation(v2)) => v1 == v2,
            (Error::InvalidReservationId(v1), Error::InvalidReservationId(v2)) => v1 == v2,
            (Error::InvalidUserId(v1), Error::InvalidUserId(v2)) => v1 == v2,
            (Error::InvalidResourceId(v1), Error::InvalidResourceId(v2)) => v1 == v2,
            (Error::Unknown, Error::Unknown) => true,
            _ => false,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                        // ConflictReservation获取的是String，把String Parse成一个ReservationConflictInfo
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::DbError(e),
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::DbError(_) => tonic::Status::internal(e.to_string()),
            Error::ConfigReadError => tonic::Status::internal("Failed to read configuration file"),
            Error::ConfigParseError => {
                tonic::Status::internal("Failed to parse configuration file")
            }
            Error::InvalidTime => {
                tonic::Status::invalid_argument("Invalid start or end time for the reservation")
            }
            Error::ConflictReservation(info) => {
                tonic::Status::failed_precondition(format!("Conflict reservation: {:?}", info))
            }
            Error::NotFound => {
                tonic::Status::not_found("No reservation found by the given condition")
            }
            Error::InvalidReservationId(id) => {
                tonic::Status::invalid_argument(format!("Invalid reservation {}", id))
            }
            Error::InvalidUserId(id) => {
                tonic::Status::invalid_argument(format!("Invalid user id {}", id))
            }
            Error::InvalidResourceId(id) => {
                tonic::Status::invalid_argument(format!("Invalid resource {}", id))
            }
            Error::Unknown => tonic::Status::unknown("unknown error"),
        }
    }
}
