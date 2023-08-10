use crate::{ReservationStatus, RsvpStatus};
use std::fmt;

impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Blocked => ReservationStatus::Blocked,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Unknown => ReservationStatus::Unknown,
        }
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}
