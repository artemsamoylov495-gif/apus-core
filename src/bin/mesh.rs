#[tokio::main]
async fn main() {
    println!("=== APUS Mesh P2P Service ===");
    apus_core::corp::start_dashboard_bind_all(false, true).await;
}

