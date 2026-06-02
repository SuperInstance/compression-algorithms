//! LZW compression: dictionary-based encode/decode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LZW codec for dictionary-based compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LzwCodec {
    /// Initial dictionary size (typically 256 for byte values).
    pub initial_dict_size: usize,
    /// Maximum dictionary entries before reset.
    pub max_dict_size: usize,
}

impl Default for LzwCodec {
    fn default() -> Self {
        LzwCodec {
            initial_dict_size: 256,
            max_dict_size: 4096,
        }
    }
}

impl LzwCodec {
    pub fn new(max_dict_size: usize) -> Self {
        LzwCodec {
            initial_dict_size: 256,
            max_dict_size,
        }
    }

    /// Encode data using LZW. Returns a vector of dictionary indices.
    pub fn encode(&self, data: &[u8]) -> Vec<usize> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut dict: HashMap<Vec<u8>, usize> = HashMap::new();
        for i in 0..self.initial_dict_size {
            dict.insert(vec![i as u8], i);
        }
        let mut next_code = self.initial_dict_size;

        let mut result = Vec::new();
        let mut current = vec![data[0]];

        for &b in &data[1..] {
            let mut extended = current.clone();
            extended.push(b);

            if dict.contains_key(&extended) {
                current = extended;
            } else {
                result.push(dict[&current]);
                if next_code < self.max_dict_size {
                    dict.insert(extended, next_code);
                    next_code += 1;
                }
                current = vec![b];
            }
        }
        result.push(dict[&current]);
        result
    }

    /// Decode LZW-encoded indices back to data.
    pub fn decode(&self, codes: &[usize]) -> Vec<u8> {
        if codes.is_empty() {
            return Vec::new();
        }

        let mut dict: HashMap<usize, Vec<u8>> = HashMap::new();
        for i in 0..self.initial_dict_size {
            dict.insert(i, vec![i as u8]);
        }
        let mut next_code = self.initial_dict_size;

        let mut result = Vec::new();
        let mut prev = dict.get(&codes[0]).cloned().unwrap_or_default();
        result.extend_from_slice(&prev);

        for &code in &codes[1..] {
            let entry = if let Some(e) = dict.get(&code) {
                e.clone()
            } else if code == next_code {
                // Special case: code not yet in dictionary
                let mut e = prev.clone();
                e.push(prev[0]);
                e
            } else {
                prev.clone() // fallback
            };

            result.extend_from_slice(&entry);

            if next_code < self.max_dict_size {
                let mut new_entry = prev.clone();
                new_entry.push(entry[0]);
                dict.insert(next_code, new_entry);
                next_code += 1;
            }

            prev = entry;
        }

        result
    }

    /// Encode to packed bytes (variable-width codes).
    pub fn encode_bytes(&self, data: &[u8]) -> Vec<u8> {
        let codes = self.encode(data);
        let bit_width = self.code_bit_width();
        let mut bits = Vec::new();
        for code in codes {
            for i in (0..bit_width).rev() {
                bits.push(((code >> i) & 1) as u8);
            }
        }
        let padding = (8 - bits.len() % 8) % 8;
        let mut packed = Vec::with_capacity((bits.len() + 7) / 8);
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit == 1 {
                    byte |= 1 << (7 - i);
                }
            }
            packed.push(byte);
        }
        packed
    }

    fn code_bit_width(&self) -> usize {
        let mut width = 8;
        let mut max_val = 256;
        while max_val < self.max_dict_size {
            width += 1;
            max_val *= 2;
        }
        width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzw_roundtrip() {
        let codec = LzwCodec::default();
        let data = b"ABABABABABABABAB".to_vec();
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lzw_empty() {
        let codec = LzwCodec::default();
        let encoded = codec.encode(&[]);
        assert!(encoded.is_empty());
        let decoded = codec.decode(&[]);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_lzw_repetitive() {
        let codec = LzwCodec::default();
        let data = b"AAAAAAA".to_vec();
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lzw_binary() {
        let codec = LzwCodec::default();
        let data: Vec<u8> = (0..=255).collect();
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lzw_special_case() {
        // Test the special case where code == next_code during decode
        let codec = LzwCodec::default();
        let data = b"ABABABA".to_vec();
        let encoded = codec.encode(&data);
        let decoded = codec.decode(&encoded);
        assert_eq!(decoded, data);
    }
}
