//! Shared compute-engine logic.
//!
//! Pure Rust, no dependencies, compiles to wasm32 as-is. The same functions are
//! used by the native thread implementation and the wasm worker implementation.

/// One engine step: wrapping i64 increment (wraps on overflow, no panic).
#[inline]
pub fn step(counter: i64) -> i64 {
    counter.wrapping_add(1)
}

/// Count primes in `lo..hi` (half-open).
///
/// This is the embarrassingly-parallel job for the rayon experiment: the range
/// is split into chunks, each chunk is independent.
pub fn count_primes_in_range(lo: u64, hi: u64) -> u64 {
    (lo..hi).filter(|&n| is_prime(n)).count() as u64
}

/// Number of chunks used by the parallel job.
pub const PRIME_CHUNKS: u64 = 16;

/// Split `2..n` into `PRIME_CHUNKS` chunks and return them.
pub fn prime_chunks(n: u64) -> Vec<(u64, u64)> {
    let mut chunks = Vec::new();
    let start: u64 = 2;
    let total = n.saturating_sub(start);
    let per = total / PRIME_CHUNKS;
    let mut lo = start;
    for i in 0..PRIME_CHUNKS {
        let hi = if i == PRIME_CHUNKS - 1 {
            n
        } else {
            lo + per
        };
        if hi > lo {
            chunks.push((lo, hi));
        }
        lo = hi;
    }
    chunks
}

#[inline]
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    let mut d = 3u64;
    while d * d <= n {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_wraps() {
        assert_eq!(step(0), 1);
        assert_eq!(step(i64::MAX), i64::MIN);
        assert_eq!(step(-1), 0);
    }

    #[test]
    fn primes() {
        assert_eq!(count_primes_in_range(2, 10), 4); // 2, 3, 5, 7
        assert_eq!(count_primes_in_range(2, 1000), 168);
        assert_eq!(count_primes_in_range(0, 2), 0);
    }

    #[test]
    fn chunks_cover_full_range() {
        let n = 1000;
        let chunks = prime_chunks(n);
        assert_eq!(chunks.first().unwrap().0, 2);
        assert_eq!(chunks.last().unwrap().1, n);
        assert_eq!(count_primes_in_range(2, n), 168);
    }
}
