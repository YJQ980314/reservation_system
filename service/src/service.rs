use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse, ListenRequest,
    QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};

use reservation::Rsvp;
use tonic::{async_trait, Request, Response, Status};

use crate::{ReservationStream, RsvpService};

#[async_trait]
impl ReservationService for RsvpService {
    /// make a reservation
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> std::result::Result<Response<ReserveResponse>, Status> {
        let request = request.into_inner();
        if request.reservation.is_none() {
            return Err(Status::invalid_argument("missing reservation"));
        }

        let reservation = self.manager.reserve(request.reservation.unwrap()).await?;

        Ok(Response::new(ReserveResponse {
            reservation: Some(reservation),
        }))
    }
    /// confirm a pending reservation, if reservation is not pending, do nothing
    async fn confirm(
        &self,
        _request: Request<ConfirmRequest>,
    ) -> std::result::Result<Response<ConfirmResponse>, Status> {
        todo!()
    }
    /// update the reservation note
    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        todo!()
    }
    /// cancel a reservation
    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> std::result::Result<Response<CancelResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    /// get a reservation by id
    async fn get(
        &self,
        _request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        todo!()
    }
    /// query reservations by resource id, user id, status, start and end time
    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> std::result::Result<Response<Self::queryStream>, Status> {
        todo!()
    }
    /// filter reservations order by reservation id
    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> std::result::Result<Response<FilterResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the listen method.
    type listenStream = ReservationStream;
    /// another system could monitor newly added/confirmed/cancelled reservations
    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> std::result::Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}

#[cfg(test)]

mod tests {
    // use std::path::Path;

    use std::ops::Deref;
    use super::*;
    use abi::{Config, Reservation};
    // use sqlx::migrate::Migrator;
    use sqlx_db_test::TestDb;

    struct TestConfig {
        pub config: Config,
        #[allow(dead_code)]
        tdb: TestDb,
    }

    impl Deref for TestConfig {
        type Target = Config;

        fn deref(&self) -> &Self::Target {
            &self.config
        }
    }

    impl TestConfig {
        pub fn new() -> Self {
            let mut config = Config::load("fixtures/config.yml").unwrap();
            
            let tdb = TestDb::new(
                &config.db.host,
                config.db.port,
                &config.db.user,
                &config.db.password,
                "../migrations",
            );

            config.db.dbname = tdb.dbname.clone();
            Self { config, tdb }
        }
    }

    #[tokio::test]
    async fn rpc_reserve_should_work() {
        let config = TestConfig::new();

        let service = RsvpService::from_config(&config).await.unwrap();
        let reservation = Reservation::new_pending(
            "tyr",
            "ixia-3230",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "test device reservation",
        );
        let request = tonic::Request::new(ReserveRequest {
            reservation: Some(reservation.clone()),
        });

        let response = service.reserve(request).await.unwrap();
        let reservation1 = response.into_inner().reservation;
        assert!(reservation1.is_some());
        let reservation1 = reservation1.unwrap();
        assert_eq!(reservation1.user_id, reservation.user_id);
        assert_eq!(reservation1.resource_id, reservation.resource_id);
        assert_eq!(reservation1.start, reservation.start);
        assert_eq!(reservation1.end, reservation.end);
        assert_eq!(reservation1.note, reservation.note);
        assert_eq!(reservation1.status, reservation.status);

        // TestConfig Dropped here -- db dropped
    }
}
