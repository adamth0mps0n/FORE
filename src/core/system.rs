// core/system.rs
use super::{
    GFp2, PHI_A, PHI_B, P,
    exp_phi, exp_phi_inverse,
    binary_haar_transform, apply_phi_transform,
    to_gfp2, mul_gfp2, sub_mod, add_mod
};

/// Core FORE implementation for frame alignment and operations
#[derive(Debug, Clone)]
pub struct ForeSystem {
    phi_k: GFp2,      // Key-dependent phi power
    phi_neg_k: GFp2,  // Inverse for alignment
}

impl ForeSystem {
    pub fn new(key: u32) -> Self {
        let phi_k = exp_phi(GFp2 { a: PHI_A, b: PHI_B }, key);
        let phi_neg_k = exp_phi_inverse(GFp2 { a: PHI_A, b: PHI_B }, key);

        Self {
            phi_k,
            phi_neg_k,
        }
    }

    /// Transform data into frequency domain
    pub fn to_frequency_domain(&self, data: &mut [GFp2]) {
        // Apply wavelet transform
        binary_haar_transform(data);
        // Apply phi transformation
        apply_phi_transform(data, &self.phi_k);
    }

    /// Edit directly in frequency domain
    pub fn edit_frequency(&self, data: &mut [GFp2], level: usize, pos: usize, new_value: GFp2) {
        let span = 1 << level;
        let start = pos * span;

        if start < data.len() {
            // Transform new value to match frequency domain
            let transformed = mul_gfp2(&new_value, &self.phi_k);
            data[start] = transformed;

            if start + span/2 < data.len() {
                // Maintain wavelet relationship in frequency domain
                data[start + span/2] = GFp2 {
                    a: sub_mod(0, transformed.a),
                    b: sub_mod(0, transformed.b)
                };
            }
        }
    }

    /// Reconstruct data by aligning frame of reference
    pub fn reconstruct(&self, data: &[GFp2]) -> String {
        let mut result = String::with_capacity(data.len());

        // Create aligned view
        let mut aligned = data.to_vec();
        apply_phi_transform(&mut aligned, &self.phi_neg_k);

        // Reconstruct from aligned view
        for (i, v) in aligned.iter().enumerate() {
            if v.b == 0 {
                let val = if i & 1 == 1 {
                    sub_mod(0, v.a)
                } else {
                    v.a
                };

                if val < 128 {
                    result.push(val as u8 as char);
                }
            }
        }

        result
    }

    /// Process raw bytes into GFp2 elements
    pub fn process_data(&self, data: &[u8]) -> Vec<GFp2> {
        // Convert to field elements
        let mut result: Vec<GFp2> = data.iter()
            .map(|&b| to_gfp2(b as u32))
            .collect();

        self.to_frequency_domain(&mut result);
        result
    }

    /// Verify wavelet relationships are maintained
    pub fn verify_relationships(&self, data: &[GFp2]) -> bool {
        let mut level = 0;
        while (1 << level) <= data.len() {
            let span = 1 << level;
            for pos in 0..(data.len() / span) {
                let start = pos * span;
                if start + span/2 < data.len() {
                    let first = &data[start];
                    let second = &data[start + span/2];

                    let sum = add_mod(first.a, second.a);
                    let sum_b = add_mod(first.b, second.b);

                    if sum != 0 || sum_b != 0 {
                        return false;
                    }
                }
            }
            level += 1;
        }
        true
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    // Helper to measure throughput
    fn measure_throughput(size_mb: usize, iterations: usize, key: u32) -> f64 {
        let data_size = size_mb * 1024 * 1024; // Convert MB to bytes
        let mut data: Vec<GFp2> = vec![GFp2 { a: 0, b: 0 }; data_size / 8]; // GFp2 is 8 bytes

        // Initialize with some test data
        for (i, item) in data.iter_mut().enumerate() {
            item.a = (i % P as usize) as u32;
            item.b = ((i * 2) % P as usize) as u32;
        }

        let phi_k = exp_phi(GFp2 { a: PHI_A, b: PHI_B }, key);

        // Time the full encryption process
        let start = Instant::now();
        for _ in 0..iterations {
            // Apply Haar transform
            binary_haar_transform(&mut data);

            // Apply phi transform
            apply_phi_transform(&mut data, &phi_k);
        }
        let duration = start.elapsed();

        // Calculate throughput in MB/s
        (size_mb as f64 * iterations as f64) / duration.as_secs_f64()
    }

    #[test]
    fn test_encryption_throughput() {
        // Test parameters
        let size_mb = 100;  // Test with 100MB chunks
        let iterations = 10; // Run multiple iterations for better average
        let key = 0xDEADBEEF;

        println!("\nPerformance Test Results:");
        println!("------------------------");
        println!("Data size per iteration: {}MB", size_mb);
        println!("Number of iterations: {}", iterations);

        // Measure throughput
        let throughput = measure_throughput(size_mb, iterations, key);

        println!("Throughput: {:.2} MB/s", throughput);
        println!("Total data processed: {}MB", size_mb * iterations);
        println!("------------------------");

        // Assert minimum performance
        assert!(throughput > 1000.0, "Throughput below 1000 MB/s: {:.2} MB/s", throughput);
    }

    #[test]
    fn test_exp_phi_timing() {
        let iterations = 1_000_000;
        let base = GFp2 { a: PHI_A, b: PHI_B };
        let key = 0xDEADBEEF;

        // Time exp_phi operations
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = exp_phi(base, key);
        }
        let duration = start.elapsed();

        println!("\nexp_phi Performance:");
        println!("------------------------");
        println!("Iterations: {}", iterations);
        println!("Total time: {:.2?}", duration);
        println!("Time per operation: {:.2?}", duration / iterations as u32);
        println!("Operations per second: {:.2}", iterations as f64 / duration.as_secs_f64());
        println!("------------------------");
    }
}
