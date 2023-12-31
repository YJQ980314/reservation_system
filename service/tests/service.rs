#[path = "../src/test_utils.rs"]
mod test_utils;

use abi::{
    reservation_service_client::ReservationServiceClient, ConfirmRequest,
    Reservation, ReservationStatus, ReserveRequest, Config, QueryRequest, ReservationQueryBuilder, ReservationFilterBuilder, FilterResponse, FilterRequest,
};
use futures::StreamExt;
use reservation_service::start_server;
use tokio::time;
use tonic::transport::Channel;
use std::time::Duration;
use test_utils::TestConfig;

#[tokio::test]
async fn grpc_server_should_work() {
    let tconfig = TestConfig::with_server_port(50001);
    // print!("tconfig: {:?}", tconfig);
    let mut client = get_test_client(&tconfig).await;

    let mut rsvp = Reservation::new_pending(
        "tyr",
        "ixia-3230",
        "2022-12-26T15:00:00-0700".parse().unwrap(),
        "2022-12-30T12:00:00-0700".parse().unwrap(),
        "test device reservation",
    );
    let ret = client
        .reserve(ReserveRequest::new(rsvp.clone()))
        .await
        .unwrap()
        .into_inner().reservation.unwrap();

    rsvp.id = ret.id;
    assert_eq!(ret, rsvp);

    //then we try to make a conflicting reservation
    let rsvp2 = Reservation::new_pending(
        "tyr",
        "ixia-3230",
        "2022-12-26T15:00:00-0700".parse().unwrap(),
        "2022-12-30T12:00:00-0700".parse().unwrap(),
        "test device reservation",
    );
    let ret = client.reserve(ReserveRequest::new(rsvp2.clone())).await;

    // assert_eq!(
    //     ret.unwrap_err().to_string(),
    //     "rpc error: code = InvalidArgument desc = reservation conflict"
    // );
    assert!(ret.is_err());

    // then we confirm first reservation
    let ret = client
        .confirm(ConfirmRequest::new(rsvp.id))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(
        ret.reservation.unwrap().status,
        ReservationStatus::Confirmed as i32
    );
}

#[tokio::test]
async fn grpc_query_should_work() {
    let tconfig = TestConfig::with_server_port(50010);
    let mut client = get_test_client(&tconfig).await;
    make_reservation(&mut client, 100).await;

    let query = ReservationQueryBuilder::default()
        .user_id("alice")
        .build()
        .unwrap(); 
    // query for all reservations
    let mut ret = client
        .query(QueryRequest::new(query))
        .await
        .unwrap()
        .into_inner();

    while let Some(Ok(rsvp)) = ret.next().await {
        assert_eq!(rsvp.user_id, "alice");
    }
}

#[tokio::test]
async fn grpc_filter_should_work() {
    let tconfig = TestConfig::with_server_port(50000);
    let mut client = get_test_client(&tconfig).await;

    make_reservation(&mut client, 25).await;

    // we filter by user
    let filter = ReservationFilterBuilder::default()
        .user_id("alice")
        .status(abi::ReservationStatus::Pending as i32)
        .build()
        .unwrap();

    assert_eq!(1, 1);
    
    let FilterResponse {
        pager,
        reservations,
    } = client
        .filter(FilterRequest::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();

    let pager = pager.unwrap();

    println!("pager: {:?}", pager);
    println!("reservations: {:?}", reservations);

    assert_eq!(pager.next, Some(filter.page_size)); // we alraedy have an item
    assert_eq!(pager.prev, None);
    // assert_eq!(pager.total, 100); //not implemented yet

    assert_eq!(reservations.len(), filter.page_size as usize);

    let filter = filter.next_page(&pager).unwrap();
    // then we get next page
    let FilterResponse {
        pager,
        reservations,
    } = client
        .filter(FilterRequest::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();

    let pager = pager.unwrap();

    // println!("pager: {:?}", pager);
    // println!("reservations: {:?}", reservations);

    assert_eq!(pager.next, filter.cursor.map(|v| v + filter.page_size));
    assert_eq!(pager.prev, filter.cursor.map(|v| v + 1));
    // assert_eq!(pager.total, 100); //not implemented yet

    assert_eq!(reservations.len(), filter.page_size as usize);

    // then we get next page(last page)
    let filter = filter.next_page(&pager).unwrap();
    let FilterResponse {
        pager,
        reservations,
    } = client
        .filter(FilterRequest::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();
    let pager = pager.unwrap();

    assert_eq!(pager.next, None);
    assert_eq!(pager.prev, filter.cursor.map(|v| v + 1));
    // assert_eq!(pager.total, 100); //not implemented yet

    assert_eq!(reservations.len(), 5);
}

async fn get_test_client(tconfig: &TestConfig) -> ReservationServiceClient<Channel> {
    let config = &tconfig.config;
    setup_server(config);
    // ReservationServiceClient::connect(config.server.url(false))
    //     .await
    //     .unwrap()

    let fut = async move {
        // if error o conn retry until timeout
        while ReservationServiceClient::connect(config.server.url(false))
            .await
            .is_err() 
        {
            time::sleep(Duration::from_millis(10)).await;
        }
        ReservationServiceClient::connect(config.server.url(false))
            .await.unwrap()
    };

    time::timeout(Duration::from_secs(5), fut).await.unwrap()
}

fn setup_server(config: &Config) {
    let config_cloned = config.clone();
    tokio::spawn(async move {
        start_server(&config_cloned).await.unwrap();
    });
    // time::sleep(Duration::from_millis(100)).await;
}

async fn make_reservation(client: &mut ReservationServiceClient<Channel>, count: i32) {
    // then we make 100 reservations with conflict
    for i in 0..count {
        let mut rsvp = Reservation::new_pending(
            "alice",
            format!("router-{}", i),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            &format!("test device reservation {}", i),
        );
        let ret = client
            .reserve(ReserveRequest::new(rsvp.clone()))
            .await
            .unwrap()
            .into_inner().reservation.unwrap();

        rsvp.id = ret.id;
        assert_eq!(ret, rsvp);
    }
}