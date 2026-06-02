//! Run-length encoding (RLE).

use serde::{Deserialize, Serialize};

/// RLE codec for encoding and decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RleCodec;

impl RleCodec {
    /// Encode data using run-length encoding.
    /// Output format: pairs of (count, value). Runs > 255 are split.
    pub fn encode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut current = data[0];
        let mut count: u8 = 1;

        for &b in &data[1..] {
            if b == current && count < 255 {
                count += 1;
            } else {
                result.push(count);
                result.push(current);
                current = b;
                count = 1;
            }
        }
        result.push(count);
        result.push(current);
        result
    }

    /// Decode run-length encoded data.
    pub fn decode(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut i = 0;
        while i + 1 < data.len() {
            let count = data[i] as usize;
            let value = data[i + 1];
            for _ in 0..count {
                result.push(value);
            }
            i += 2;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_basic() {
        let data = vec![1, 1, 1, 2, 2, 3];
        let encoded = RleCodec::encode(&data);
        let decoded = RleCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_empty() {
        let encoded = RleCodec::encode(&[]);
        assert!(encoded.is_empty());
    }

    #[test]
    fn test_rle_single() {
        let data = vec![42];
        let encoded = RleCodec::encode(&data);
        let decoded = RleCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_long_run() {
        let data = vec![7; 300];
        let encoded = RleCodec::encode(&data);
        let decoded = RleCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_rle_alternating() {
        let data = vec![1, 2, 1, 2, 1, 2];
        let encoded = RleCodec::encode(&data);
        let decoded = RleCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }
}
