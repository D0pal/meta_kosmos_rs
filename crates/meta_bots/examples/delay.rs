use std::time::Duration;

#[tokio::main]
async fn main() {
    // let when = Instant::now() + Duration::from_millis(100);
    let start = chrono::Utc::now().to_rfc3339();
    tokio::time::sleep(Duration::from_millis(2000)).await;
    let end = chrono::Utc::now().to_rfc3339();
    println!("start {:?}", start);
    println!("end {:?}", end);
}
