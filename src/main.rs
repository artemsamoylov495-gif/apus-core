use std::env;
use apus_core::{apus_init_node, apus_get_current_limit};
use apus_core::p2p::BlockValidator;
use apus_core::manifest::UpdateManifest;
use apus_core::network::P2PNode;
use apus_core::zero_copy::ZeroCopyEngine;
use apus_core::discovery::DiscoveryEngine;
use apus_core::nat::NatEngine;
use apus_core::routing::RoutingEngine;
use apus_core::compress::CompressEngine;
use apus_core::corp;
use sha2::{Sha256, Digest};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let is_corp = args.contains(&"--corp".to_string());
    let is_mesh = args.contains(&"--mesh".to_string());

    println!("=== APUS v0.75 \"Hirenkyaku\" Full Mesh Engine ===");

    if apus_init_node() {
        println!("\n--- [1] Проверка интеллектуальных триггеров ---");
        println!("[Сценарий 1] Афк 10 минут -> Лимит: {}%", apus_get_current_limit(true, false));
        println!("[Сценарий 2] Фильм в полноэкране -> Лимит: {}%", apus_get_current_limit(false, true));
        println!("[Сценарий 3] Игрульке / работаем -> Лимит: {}%", apus_get_current_limit(false, false));

        println!("\n--- [2] NAT Traversal & Hole Punching ---");
        let nat = NatEngine::resolve_nat();
        println!("[NAT Status] External IP: {} | CGNAT Detected: {}", nat.external_ip, nat.is_behind_cgnat);
        println!("[NAT Traversal] UDP Hole Punching & UPnP Port Mapping: Active (Port {})", nat.mapped_port);

        println!("\n--- [3] LAN Discovery & RTT Routing ---");
        println!("[mDNS Discovery] Scanning local subnet 192.168.1.0/24...");
        let raw_peers = DiscoveryEngine::scan_local_network("192.168.1");
        let sorted_peers = RoutingEngine::prioritize_peers(raw_peers);
        for (idx, peer) in sorted_peers.iter().enumerate() {
            println!("[RTT Routing] Priority #{}: '{}' ({}:{}) [RTT: {:.2} ms]", idx + 1, peer.node_id, peer.ip_address, peer.port, peer.rtt_ms);
        }

        println!("\n--- [4] Dynamic Zstd Compression ---");
        let chunk_size = 4 * 1024 * 1024;
        let comp = CompressEngine::compress_chunk(chunk_size, 50);
        println!("[Zstd Engine] Input: {:.2} MB -> Output: {:.2} MB (Saved {:.1}%) [{}]",
                 comp.original_size as f64 / (1024.0 * 1024.0),
                 comp.compressed_size as f64 / (1024.0 * 1024.0),
                 comp.ratio_pct, comp.algorithm);

        println!("\n--- [5] Zero-Copy Kernel Transfer (splice / sendfile) ---");
        let metrics = ZeroCopyEngine::send_chunk_kernel(comp.compressed_size);
        println!("[Zero-Copy Engine] Streaming compressed chunk NVMe -> Kernel Socket...");
        println!("[Zero-Copy Engine] Transfer completed in {:.3} ms", metrics.elapsed_ms);
        println!("[Metrics] CPU Usage: {:.2}% | RAM Overhead: {:.1} MB | Zero-Copy: Active", metrics.cpu_usage_pct, metrics.ram_used_mb);

        println!("\n--- [6] Манифест, P2P Обмен & Integrity ---");
        let fake_chunk = b"AerOS_APUS_Update_Block_#1337".to_vec();
        let mut hasher = Sha256::new();
        hasher.update(&fake_chunk);
        let hash_result = hasher.finalize();
        let real_hash: String = hash_result.iter().map(|b| format!("{:02x}", b)).collect();

        let mut manifest = UpdateManifest::new("patch-v1.4.2", "tanks_patch.bin", 2_254_857_830, 4_194_304);
        manifest.add_chunk(0, &real_hash);

        let server_chunk = fake_chunk.clone();
        tokio::spawn(async move {
            let _ = P2PNode::start_seeder(server_chunk).await;
        });

        sleep(Duration::from_millis(300)).await;

        if let Ok(downloaded_data) = P2PNode::download_chunk().await {
            let is_valid = BlockValidator::verify_block(&downloaded_data, &real_hash);
            println!("[P2P Integrity Check] Скачанный блок #0 валиден? -> {}", is_valid);
        }

        println!("\n--- [7] Тест HTTPS Fallback ---");
        let fallback_data = P2PNode::fallback_https_download("https://cdn.aeros.org/updates/patch.bin");
        let is_fallback_valid = BlockValidator::verify_block(&fallback_data, &real_hash);
        println!("[Fallback Check] Блок из HTTPS CDN валиден? -> {}", is_fallback_valid);
    }

    if is_corp || is_mesh {
        let mode_label = if is_mesh { "APUS Mesh (--mesh)" } else { "APUS Corp (--corp)" };
        println!("\n--- [{}] ---", mode_label);
        println!("[Dashboard Active] Web Interface bound to http://0.0.0.0:9090");
        println!("[Dashboard Active] Instant Search & Zero-Copy Drag-and-Drop GUI Ready");
        println!("\nPress Ctrl+C to stop APUS engine.");

        let corp_task = tokio::spawn(async move {
            corp::start_dashboard_bind_all(is_corp, is_mesh).await;
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n[APUS Engine] Shutting down gracefully...");
            }
            _ = corp_task => {}
        }
    }
}
