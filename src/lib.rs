pub mod hardware;
pub mod triggers;
pub mod p2p;
pub mod manifest;
pub mod network;

use triggers::{TriggerMonitor, WorkMode};
use p2p::BlockValidator;

#[no_mangle]
pub extern "C" fn apus_init_node() -> bool {
    let score = hardware::evaluate_node();
    println!("[APUS v0.1 \"Shunpo\"] Hardware Audit Result: {:?}", score);
    score.can_participate
}

#[no_mangle]
pub extern "C" fn apus_get_current_limit(idle_10m: bool, media_mode: bool) -> u8 {
    let mut monitor = TriggerMonitor::new();
    let mode = monitor.check_current_mode(idle_10m, media_mode);
    
    match mode {
        WorkMode::Inactive => 50,
        WorkMode::PassiveStream => 30,
        WorkMode::ActiveStop => 0,
    }
}

#[no_mangle]
pub extern "C" fn apus_verify_chunk(chunk_ptr: *const u8, len: usize, expected_hash_ptr: *const std::os::raw::c_char) -> bool {
    if chunk_ptr.is_null() || expected_hash_ptr.is_null() {
        return false;
    }

    unsafe {
        let slice = std::slice::from_raw_parts(chunk_ptr, len);
        let c_str = std::ffi::CStr::from_ptr(expected_hash_ptr);
        if let Ok(hash_str) = c_str.to_str() {
            BlockValidator::verify_block(slice, hash_str)
        } else {
            false
        }
    }
}
pub mod zero_copy;
pub mod discovery;
pub mod nat;
pub mod routing;
pub mod compress;
pub mod corp;
