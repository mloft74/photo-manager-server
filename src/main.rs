use photo_manager_server::run;
#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        println!("debug");
    } else {
        println!("release");
    }

    run().await;
}
