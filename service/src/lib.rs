mod service;
// #[cfg(feature = "test-utils")]
// mod test_utils;
#[cfg(test)]
pub mod test_utils;

use std::{pin::Pin, task::Poll};

use abi::{Config, Reservation, reservation_service_server::ReservationServiceServer};
use futures::Stream;
use reservation::ReservationManager;
use tokio::sync::mpsc;
use tonic::{Status, transport::Server};

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

#[derive(Debug)]
pub struct RsvpService {
    pub manager: ReservationManager,
}

pub struct TonicReceiverStream<T> {
    inner: mpsc::Receiver<Result<T, abi::Error>>,
}

impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner }
    }
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            manager: ReservationManager::from_config(&config.db).await?,
        })
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(t))) => Poll::Ready(Some(Ok(t))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub async fn start_server(config: &Config) -> Result<(), anyhow::Error> {
    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let svc = RsvpService::from_config(config).await?;
    let svc = ReservationServiceServer::new(svc);
    
    println!("Listening on {}", addr);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
