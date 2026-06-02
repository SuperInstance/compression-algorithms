//! LZ77 sliding window compression.

use serde::{Deserialize, Serialize};

/// LZ77 codec using sliding window compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lz77Codec {
    /// Window size for back-references.
    pub window_size: usize,
    /// Maximum match length.
    pub max_match_len: usize,
    /// Minimum match length (shorter matches are stored as literals).
    pub min_match_len: usize,
}

impl Default for Lz77Codec {
    fn default() -> Self {
        Lz77Codec {
            window_size: 4096,
            max_match_len: 258,
            min_match_len: 3,
        }
    }
}

/// A token in LZ77: either a literal byte or a (distance, length) back-reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Lz77Token {
    Literal(u8),
    Reference { distance: usize, length: usize },
}

impl Lz77Codec {
    pub fn new(window_size: usize) -> Self {
        Lz77Codec {
            window_size,
            max_match_len: 258,
            min_match_len: 3,
        }
    }

    /// Encode data into a sequence of LZ77 tokens.
    pub fn encode(&self, data: &[u8]) -> Vec<Lz77Token> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            let window_start = if pos > self.window_size { pos - self.window_size } else { 0 };
            let (best_dist, best_len) = self.find_match(data, pos, window_start);

            if best_len >= self.min_match_len {
                tokens.push(Lz77Token::Reference {
                    distance: best_dist,
                    length: best_len,
                });
                pos += best_len;
            } else {
                tokens.push(Lz77Token::Literal(data[pos]));
                pos += 1;
            }
        }

        tokens
    }

    fn find_match(&self, data: &[u8], pos: usize, window_start: usize) -> (usize, usize) {
        let mut best_dist = 0;
        let mut best_len = 0;
        let max_len = self.max_match_len.min(data.len() - pos);

        for i in window_start..pos {
            let mut len = 0;
            while len < max_len {
                let src = i + len;
                // For overlapping matches, src can wrap around
                let src_byte = if src < pos { data[src] } else { data[pos + ((src - pos) % len.max(1))] };
                if src_byte != data[pos + len] {
                    break;
                }
                len += 1;
            }

            // Non-overlapping match
            let mut actual_len = 0;
            while actual_len < max_len && data[i + actual_len] == data[pos + actual_len] {
                actual_len += 1;
            }

            if actual_len > best_len {
                best_len = actual_len;
                best_dist = pos - i;
            }
        }

        (best_dist, best_len)
    }

    /// Decode LZ77 tokens back to data.
    pub fn decode(&self, tokens: &[Lz77Token]) -> Vec<u8> {
        let mut result = Vec::new();

        for token in tokens {
            match token {
                Lz77Token::Literal(b) => {
                    result.push(*b);
                }
                Lz77Token::Reference { distance, length } => {
                    let start = result.len() - distance;
                    for i in 0..*length {
                        let b = result[start + i];
                        result.push(b);
                    }
                }
            }
        }

        result
    }

    /// Encode tokens to bytes: literal (0x00, byte), reference (dist_hi, dist_lo, length).
    pub fn encode_to_bytes(&self, data: &[u8]) -> Vec<u8> {
        let tokens = self.encode(data);
        let mut result = Vec::new();
        for token in &tokens {
            match token {
                Lz77Token::Literal(b) => {
                    result.push(0);
                    result.push(*b);
                }
                Lz77Token::Reference { distance, length } => {
                    result.push((distance >> 8) as u8 | 0x80);
                    result.push((distance & 0xFF) as u8);
                    result.push(*length as u8);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_roundtrip() {
        let codec = Lz77Codec::default();
        let data = b"abcabcabcabcabc".to_vec();
        let tokens = codec.encode(&data);
        let decoded = codec.decode(&tokens);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lz77_empty() {
        let codec = Lz77Codec::default();
        let tokens = codec.encode(&[]);
        assert!(tokens.is_empty());
        let decoded = codec.decode(&tokens);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_lz77_no_repetition() {
        let codec = Lz77Codec::default();
        let data = b"abcdefghij".to_vec();
        let tokens = codec.encode(&data);
        let decoded = codec.decode(&tokens);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lz77_all_same() {
        let codec = Lz77Codec::default();
        let data = vec![42; 1000];
        let tokens = codec.encode(&data);
        let decoded = codec.decode(&tokens);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lz77_small_window() {
        let codec = Lz77Codec::new(16);
        let data = b"the cat in the hat sat on the mat".to_vec();
        let tokens = codec.encode(&data);
        let decoded = codec.decode(&tokens);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_lz77_encode_bytes() {
        let codec = Lz77Codec::default();
        let data = b"aabaabaab".to_vec();
        let bytes = codec.encode_to_bytes(&data);
        assert!(!bytes.is_empty());
    }
}
