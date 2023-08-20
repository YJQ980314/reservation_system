use sqlx_postgres::PgListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATAVASE_URL must be set");
    let mut listener = PgListener::connect(&url).await.unwrap();
    listener.listen("reservation_update").await.unwrap();
    println!("Listening for reservation_update events...");
    loop {
        let notification = listener.recv().await.unwrap();
        println!("Received notification: {:?}", notification);
    }
}