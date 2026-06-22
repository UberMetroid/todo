use sha2::{Digest, Sha256};
use shared::TodoLists;

pub const MAX_ATTEMPTS: usize = 5;

// Cryptographically secure constant-time string comparison using SHA-256 hashes
pub fn secure_compare(a: &str, b: &str) -> bool {
    let mut hasher_a = Sha256::new();
    hasher_a.update(a.as_bytes());
    let a_hash = hasher_a.finalize();

    let mut hasher_b = Sha256::new();
    hasher_b.update(b.as_bytes());
    let b_hash = hasher_b.finalize();

    let mut result = 0;
    for (x, y) in a_hash.iter().zip(b_hash.iter()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn generate_random_id() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(9)
        .map(char::from)
        .collect()
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
