#[tokio::main]
async fn main() {
    println!("=== APUS Corp B2B Service ===");
    apus_core::corp::start_dashboard_bind_all(true, false).await;
}
