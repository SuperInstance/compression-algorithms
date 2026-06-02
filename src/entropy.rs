//! Shannon entropy bounds and theoretical compression limits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entropy calculator for analyzing data compressibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entropy;

impl Entropy {
    /// Calculate Shannon entropy in bits per symbol.
    pub fn shannon_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let freq = Self::frequency(data);
        let total = data.len() as f64;
        let mut entropy = 0.0;

        for &count in freq.values() {
            if count > 0 {
                let p = count as f64 / total;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Calculate frequency distribution.
    pub fn frequency(data: &[u8]) -> HashMap<u8, usize> {
        let mut freq = HashMap::new();
        for &b in data {
            *freq.entry(b).or_insert(0) += 1;
        }
        freq
    }

    /// Calculate theoretical minimum compressed size in bits.
    pub fn min_compressed_bits(data: &[u8]) -> f64 {
        Self::shannon_entropy(data) * data.len() as f64
    }

    /// Calculate theoretical minimum compressed size in bytes.
    pub fn min_compressed_bytes(data: &[u8]) -> f64 {
        Self::min_compressed_bits(data) / 8.0
    }

    /// Calculate compression ratio achieved (original / compressed).
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if compressed_size == 0 {
            return f64::INFINITY;
        }
        original_size as f64 / compressed_size as f64
    }

    /// Compare actual compression against entropy bound.
    /// Returns (actual_ratio, theoretical_max_ratio, efficiency).
    pub fn compression_efficiency(data: &[u8], compressed_size: usize) -> (f64, f64, f64) {
        let original_size = data.len();
        let actual_ratio = Self::compression_ratio(original_size, compressed_size);
        let min_bits = Self::min_compressed_bits(data);
        let theoretical_max = if min_bits > 0.0 {
            (original_size * 8) as f64 / min_bits
        } else {
            f64::INFINITY
        };
        let efficiency = if theoretical_max.is_finite() && theoretical_max > 0.0 {
            actual_ratio / theoretical_max
        } else {
            0.0
        };
        (actual_ratio, theoretical_max, efficiency)
    }

    /// Calculate entropy for a sequence of arbitrary symbols (not just bytes).
    pub fn symbol_entropy<T: Eq + std::hash::Hash>(symbols: &[T]) -> f64 {
        if symbols.is_empty() {
            return 0.0;
        }
        let mut freq: HashMap<&T, usize> = HashMap::new();
        for s in symbols {
            *freq.entry(s).or_insert(0) += 1;
        }
        let total = symbols.len() as f64;
        let mut entropy = 0.0;
        for &count in freq.values() {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
        entropy
    }

    /// Cross-entropy between two distributions.
    pub fn cross_entropy(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let model_total: usize = model_freq.values().sum();
        if model_total == 0 {
            return f64::INFINITY;
        }
        let mut ce = 0.0;
        let data_freq = Self::frequency(data);
        let data_total = data.len() as f64;

        for (&sym, &count) in &data_freq {
            let p = count as f64 / data_total;
            let q = if let Some(&mc) = model_freq.get(&sym) {
                mc as f64 / model_total as f64
            } else {
                1e-10
            };
            ce -= p * q.log2();
        }
        ce
    }

    /// KL divergence from data distribution to model distribution.
    pub fn kl_divergence(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64 {
        let h_data = Self::shannon_entropy(data);
        let ce = Self::cross_entropy(data, model_freq);
        ce - h_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_uniform() {
        // All bytes the same -> entropy = 0
        let data = vec![42; 100];
        let e = Entropy::shannon_entropy(&data);
        assert!(e.abs() < 1e-10);
    }

    #[test]
    fn test_entropy_fair_coin() {
        // Two equally likely symbols -> entropy = 1 bit
        let data: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        let e = Entropy::shannon_entropy(&data);
        assert!((e - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_max() {
        // 256 equally likely bytes -> entropy = 8 bits
        let data: Vec<u8> = (0..=255).cycle().take(2560).collect();
        let e = Entropy::shannon_entropy(&data);
        assert!((e - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_entropy_empty() {
        assert_eq!(Entropy::shannon_entropy(&[]), 0.0);
    }

    #[test]
    fn test_min_compressed() {
        let data = vec![0; 100];
        let bits = Entropy::min_compressed_bits(&data);
        assert!(bits.abs() < 1e-10);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = Entropy::compression_ratio(100, 50);
        assert!((ratio - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_compression_efficiency() {
        let data: Vec<u8> = (0..=255).cycle().take(2560).collect();
        let (actual, theoretical, eff) = Entropy::compression_efficiency(&data, 2560);
        assert!(actual > 0.0);
        assert!(theoretical > 0.0);
        assert!(eff > 0.0);
    }

    #[test]
    fn test_symbol_entropy() {
        let symbols = vec!["a", "a", "b", "b"];
        let e = Entropy::symbol_entropy(&symbols);
        assert!((e - 1.0).abs() < 1e-10);
    }
}
