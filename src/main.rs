use dotenv::dotenv;

mod app;
mod candle;

#[tokio::main]
async fn main() {
    dotenv().ok();

    app::run().await;
}
