#[tokio::main]
async fn main() {
    koii::init();
    koii::core().await;
}
