use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Returns a random alphanumeric string of the given `length`.
pub fn random_string(length: usize) -> String {
    // Seed from wall-clock nanoseconds for basic uniqueness.
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 + d.as_secs().wrapping_mul(1_000_000_007))
        .unwrap_or(42);

    let mut result = String::with_capacity(length);
    let mut state = seed;

    for i in 0..length {
        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        i.hash(&mut hasher);
        state = hasher.finish();

        let idx = (state % CHARSET.len() as u64) as usize;
        result.push(CHARSET[idx] as char);
    }

    result
}
