// core/transform.rs
use super::{GFp2, mul_gfp2, sub_mod};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;
use rayon::prelude::*;

/// Base chunk size - will be scaled based on hardware
pub const BASE_CHUNK_SIZE: usize = 1024;

// Static configuration for adaptive performance
pub static OPTIMAL_CHUNK_SIZE: AtomicUsize = AtomicUsize::new(0);
pub static INIT: Once = Once::new();

/// Initialize optimal chunk size based on hardware
pub fn init_chunk_size() {
    INIT.call_once(|| {
        let cpu_count = num_cpus::get();
        let optimal = BASE_CHUNK_SIZE * cpu_count;
        OPTIMAL_CHUNK_SIZE.store(optimal, Ordering::Relaxed);
    });
}

/// Get optimal chunk size based on data and system characteristics
#[inline]
pub fn get_chunk_size() -> usize {
    const TARGET_CHUNK_SIZE: usize = 256 * 1024;  // 256KB target
    const MIN_CHUNK_SIZE: usize = 4 * 1024;       // 4KB minimum

    static CHUNK_SIZE: AtomicUsize = AtomicUsize::new(0);
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let cpu_count = num_cpus::get();
        let size = TARGET_CHUNK_SIZE.min(TARGET_CHUNK_SIZE * cpu_count)
                                  .max(MIN_CHUNK_SIZE);
        CHUNK_SIZE.store(size, Ordering::Relaxed);
    });

    CHUNK_SIZE.load(Ordering::Relaxed)
}

/// Apply binary Haar transform with parallel processing
pub fn binary_haar_transform(data: &mut [GFp2]) {
    data.par_chunks_mut(get_chunk_size()).for_each(|chunk| {
        chunk.iter_mut().enumerate().for_each(|(i, v)| {
            if (i & 1) == 1 {
                v.a = sub_mod(0, v.a);
                v.b = sub_mod(0, v.b);
            }
        });
    });
}

/// Apply φ^k transform with parallel processing
pub fn apply_phi_transform(data: &mut [GFp2], phi_k: &GFp2) -> Option<()> {
    if data.is_empty() {
        return None;
    }

    data.par_chunks_mut(get_chunk_size()).for_each(|chunk| {
        chunk.iter_mut().for_each(|v| {
            *v = mul_gfp2(v, phi_k);
        });
    });

    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::to_gfp2;
    use rayon::iter::ParallelIterator;

    #[test]
    fn test_parallel_haar_transform() {
        let mut data: Vec<GFp2> = (0..1000)
            .into_par_iter()
            .map(|i| to_gfp2(i as u32))
            .collect();

        binary_haar_transform(&mut data);

        // Verify alternating signs
        for i in (1..data.len()).step_by(2) {
            assert_eq!(data[i].a, sub_mod(0, to_gfp2(i as u32).a));
        }
    }
}
