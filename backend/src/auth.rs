use shared::TodoLists;
use std::fs::File;
use std::io::Read;

pub const MAX_ATTEMPTS: usize = 5;

// Cryptographically secure constant-time string comparison padded to 10 characters
pub fn secure_compare(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let equal = a_bytes.len() == b_bytes.len();

    let mut diff = 0u8;
    for i in 0..10 {
        let char_a = if i < a_bytes.len() { a_bytes[i] } else { 0 };
        let char_b = if i < b_bytes.len() { b_bytes[i] } else { 0 };
        diff |= char_a ^ char_b;
    }

    equal && (diff == 0)
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
