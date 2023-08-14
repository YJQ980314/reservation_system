use std::pin::Pin;

use abi::{Reservation, Config};
use reservation::ReservationManager;
use tonic::Status;
use futures::Stream;

mod service;

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub struct RsvpService{ 
    manager: ReservationManager,
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self{manager: ReservationManager::from_config(&config.db).await?})
    }
}

