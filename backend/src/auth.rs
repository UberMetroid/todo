use shared::TodoLists;
use std::fs::File;
use std::io::Read;

// Cryptographically secure constant-time string comparison
pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

// Generate random alphanumeric 9-character ID using /dev/urandom or LCG
pub fn generate_random_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 9];

    if let Ok(mut file) = File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    } else {
        // Fallback LCG using system time as seed
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        for byte in &mut bytes {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (seed >> 32) as u8;
        }
    }

    bytes.iter_mut().for_each(|b| {
        *b = CHARSET[(*b as usize) % CHARSET.len()];
    });

    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn run_todo_migrations(data_file: &str) {
    if let Ok(content) = std::fs::read_to_string(data_file) {
        if let Ok(mut lists) = serde_json::from_str::<TodoLists>(&content) {
            let mut updated = false;
            for items in lists.values_mut() {
                for item in items.iter_mut() {
                    if item.id.is_empty() {
                        item.id = generate_random_id();
                        updated = true;
                    }
                }
            }
            if updated {
                if let Ok(serialized) = serde_json::to_string_pretty(&lists) {
                    let temp_file = format!("{}.tmp", data_file);
                    if std::fs::write(&temp_file, serialized).is_ok() {
                        let _ = std::fs::rename(temp_file, data_file);
                        println!("Migration: assigned unique IDs to tasks.");
                    }
                }
            }
        }
    }
}
