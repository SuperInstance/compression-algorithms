//! Burrows-Wheeler Transform (BWT) and inverse.

use serde::{Deserialize, Serialize};

/// BWT transform for data rearrangement (used in compression pipelines).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BwtTransform;

impl BwtTransform {
    /// Apply the Burrows-Wheeler Transform.
    /// Returns (transformed_data, original_index).
    pub fn forward(data: &[u8]) -> (Vec<u8>, usize) {
        if data.is_empty() {
            return (Vec::new(), 0);
        }

        let n = data.len();
        let mut indices: Vec<usize> = (0..n).collect();

        // Sort rotations using suffix comparison
        indices.sort_by(|&a, &b| {
            for i in 0..n {
                let ca = data[(a + i) % n];
                let cb = data[(b + i) % n];
                if ca != cb {
                    return ca.cmp(&cb);
                }
            }
            Ordering::Equal
        });

        let mut transformed = Vec::with_capacity(n);
        let mut original_index = 0;

        for (rank, &start) in indices.iter().enumerate() {
            transformed.push(data[(start + n - 1) % n]);
            if start == 0 {
                original_index = rank;
            }
        }

        (transformed, original_index)
    }

    /// Apply the inverse BWT.
    pub fn inverse(data: &[u8], original_index: usize) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let n = data.len();

        // Build the T (transform vector) using the standard algorithm
        let mut counts = vec![0usize; 256];
        for &b in data {
            counts[b as usize] += 1;
        }

        // Cumulative counts (first occurrence of each byte in sorted column)
        let mut first_occurrence = vec![0usize; 256];
        let mut sum = 0;
        for i in 0..256 {
            first_occurrence[i] = sum;
            sum += counts[i];
        }

        // Build LF mapping
        let mut lf = vec![0usize; n];
        let mut occ = vec![0usize; 256];
        for i in 0..n {
            let b = data[i] as usize;
            lf[i] = first_occurrence[b] + occ[b];
            occ[b] += 1;
        }

        // Reconstruct original
        let mut result = vec![0u8; n];
        let mut idx = original_index;
        for i in (0..n).rev() {
            result[i] = data[idx];
            idx = lf[idx];
        }

        result
    }
}

use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwt_banana() {
        let data = b"banana".to_vec();
        let (transformed, idx) = BwtTransform::forward(&data);
        let recovered = BwtTransform::inverse(&transformed, idx);
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_bwt_empty() {
        let (t, idx) = BwtTransform::forward(&[]);
        assert!(t.is_empty());
        let recovered = BwtTransform::inverse(&t, idx);
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_bwt_single() {
        let data = b"A".to_vec();
        let (t, idx) = BwtTransform::forward(&data);
        let recovered = BwtTransform::inverse(&t, idx);
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_bwt_repeated() {
        let data = b"AAAAAA".to_vec();
        let (t, idx) = BwtTransform::forward(&data);
        let recovered = BwtTransform::inverse(&t, idx);
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_bwt_abracadabra() {
        let data = b"abracadabra".to_vec();
        let (t, idx) = BwtTransform::forward(&data);
        let recovered = BwtTransform::inverse(&t, idx);
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_bwt_binary() {
        let data: Vec<u8> = (0..=50).map(|i| i % 7).collect();
        let (t, idx) = BwtTransform::forward(&data);
        let recovered = BwtTransform::inverse(&t, idx);
        assert_eq!(recovered, data);
    }
}
