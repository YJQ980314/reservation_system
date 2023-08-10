use crate::{ReservationId, ReservationManager, Rsvp};
// use abi::convert_to_utc_time;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();
        // generate a  insert sql for the reservation
        // execute the sql
        // Postgre对类型要求严格
        let id: Uuid = sqlx::query("INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(status.to_string())
            .fetch_one(&self.pool)
            .await?.get(0);

        rsvp.id = id.to_string();
        Ok(rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        // if current status is pending, change it to confirmed , otherwise do nothing
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp: abi::Reservation = sqlx::query_as("UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(rsvp)
    }

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        //  update the note of the reservation
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp: abi::Reservation =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(rsvp)
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        // get the reservation by id
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp: abi::Reservation =
            sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error> {
        // delete the reservation by id
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::{Reservation, ReservationConflict, ReservationConflictInfo, ResrvationWindow};

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (rsvp, _manager) = make_tyr_reservation(migrated_pool.clone()).await;
        assert!(rsvp.id != "");
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_confilct_reservation_should_reject() {
        let (_rsvp1, manager) = make_tyr_reservation(migrated_pool.clone()).await;
        let rsvp2 = abi::Reservation::new_pending(
            "aliceid",
            "ocean-view-room-713",
            "2023-12-26T15::00:00-0700".parse().unwrap(),
            "2023-12-30T12::00:00-0700".parse().unwrap(),
            "hello.",
        );

        let err = manager.reserve(rsvp2).await.unwrap_err();
        // println!("{:?}", err);

        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ResrvationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2023-12-26T15::00:00-0700".parse().unwrap(),
                end: "2023-12-30T12::00:00-0700".parse().unwrap(),
            },
            old: ResrvationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2023-12-25T15::00:00-0700".parse().unwrap(),
                end: "2023-12-28T12::00:00-0700".parse().unwrap(),
            },
        });

        assert_eq!(err, abi::Error::ConflictReservation(info));

        // if let abi::Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
        //     assert_eq!(info.old.rid, "ocean-view-room-713");
        //     assert_eq!(info.old.start.to_rfc3339(), "2023-12-25T22:00:00+00:00");
        //     assert_eq!(info.old.end.to_rfc3339(), "2023-12-28T19:00:00+00:00");
        // } else {
        //     panic!("expect conflict reservation error");
        // }
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_status_not_pending_should_do_nothing() {
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        // change status again should do nothing
        let ret = manager.change_status(rsvp.id).await.unwrap_err();
        assert_eq!(ret, abi::Error::NotFound);
        // assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        let rsvp = manager
            .update_note(rsvp.id, "Hello, World.".into())
            .await
            .unwrap();
        assert_eq!(rsvp.note, "Hello, World.");
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_reservation_should_work(){
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        let rsvp1 = manager.get(rsvp.id.clone()).await.unwrap();
        assert_eq!(rsvp, rsvp1);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_reservation_should_work(){
        let (rsvp, manager) = make_alice_reservation(migrated_pool.clone()).await;
        manager.delete(rsvp.id.clone()).await.unwrap();
        let ret = manager.get(rsvp.id.clone()).await.unwrap_err();
        assert_eq!(ret, abi::Error::NotFound);
    }

    async fn make_tyr_reservation(pool: PgPool) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            "tyrid",
            "ocean-view-room-713",
            "2023-12-25T15::00:00-0700",
            "2023-12-28T12::00:00-0700",
            "hello.",
        )
        .await
    }

    async fn make_alice_reservation(pool: PgPool) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            "aliceid",
            "ixia-test-1",
            "2024-01-25T15::00:00-0700",
            "2024-02-25T12::00:00-0700",
            "I need to book this for xyz project for a month.",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, ReservationManager) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );
        (manager.reserve(rsvp).await.unwrap(), manager)
    }
}
