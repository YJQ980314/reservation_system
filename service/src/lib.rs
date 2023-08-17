use std::pin::Pin;

use abi::{Config, Reservation};
use futures::Stream;
use reservation::ReservationManager;
use tonic::Status;

mod service;

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

#[derive(Debug)]
pub struct RsvpService {
    pub manager: ReservationManager,
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager: ReservationManager::from_config(&config.db).await?,
        })
    }
}
