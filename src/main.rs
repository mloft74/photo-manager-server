use photo_manager_server::connect;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        println!("debug");
    } else {
        println!("release");
    }

    let res = connect().await;

    println!("{:?}", res);

    // run().await;
}
