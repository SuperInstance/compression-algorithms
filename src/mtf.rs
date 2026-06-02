//! Move-to-front transform.

use serde::{Deserialize, Serialize};

/// Move-to-front transform for improving compressibility after BWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtfTransform;

impl MtfTransform {
    /// Apply move-to-front transform.
    pub fn forward(data: &[u8]) -> Vec<u8> {
        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut result = Vec::with_capacity(data.len());

        for &b in data {
            let idx = alphabet.iter().position(|&x| x == b).unwrap();
            result.push(idx as u8);
            // Move to front
            alphabet.remove(idx);
            alphabet.insert(0, b);
        }

        result
    }

    /// Apply inverse move-to-front transform.
    pub fn inverse(data: &[u8]) -> Vec<u8> {
        let mut alphabet: Vec<u8> = (0..=255).collect();
        let mut result = Vec::with_capacity(data.len());

        for &idx in data {
            let b = alphabet[idx as usize];
            result.push(b);
            // Move to front
            alphabet.remove(idx as usize);
            alphabet.insert(0, b);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtf_roundtrip() {
        let data = b"hello world".to_vec();
        let encoded = MtfTransform::forward(&data);
        let decoded = MtfTransform::inverse(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_mtf_empty() {
        let encoded = MtfTransform::forward(&[]);
        assert!(encoded.is_empty());
        let decoded = MtfTransform::inverse(&encoded);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_mtf_repeated() {
        let data = vec![5, 5, 5, 5, 5];
        let encoded = MtfTransform::forward(&data);
        assert_eq!(encoded[0], 5); // First occurrence at index 5
        assert_eq!(encoded[1], 0); // Then moved to front, so index 0
        let decoded = MtfTransform::inverse(&encoded);
        assert_eq!(decoded, data);
    }
}
