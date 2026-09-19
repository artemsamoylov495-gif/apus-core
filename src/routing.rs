use crate::discovery::LocalPeer;

pub struct RoutingEngine;

impl RoutingEngine {
    /// Сортировка пиров по задержке RTT (Ping) для максимальной скорости
    pub fn prioritize_peers(mut peers: Vec<LocalPeer>) -> Vec<LocalPeer> {
        peers.sort_by(|a, b| a.rtt_ms.partial_cmp(&b.rtt_ms).unwrap_or(std::cmp::Ordering::Equal));
        peers
    }
}
