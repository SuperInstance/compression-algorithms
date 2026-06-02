//! Arithmetic coding basics.

use serde::{Deserialize, Serialize};

/// Arithmetic coder for encoding/decoding byte streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticCoder {
    /// Frequency table for symbols.
    frequencies: Vec<u64>,
    total: u64,
}

impl ArithmeticCoder {
    /// Create from raw data.
    pub fn from_data(data: &[u8]) -> Self {
        let mut frequencies = vec![0u64; 256];
        for &b in data {
            frequencies[b as usize] += 1;
        }
        let total: u64 = frequencies.iter().sum();
        ArithmeticCoder { frequencies, total: total.max(1) }
    }

    /// Create from a frequency table.
    pub fn from_frequencies(frequencies: Vec<u64>) -> Self {
        let total: u64 = frequencies.iter().sum();
        ArithmeticCoder { frequencies, total: total.max(1) }
    }

    /// Encode data into a sequence of bits (as u8 0/1).
    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() || self.total == 0 {
            return Vec::new();
        }

        // Precompute cumulative frequencies
        let cum = self.cumulative();

        let precision: u64 = 32;
        let whole = 1u64 << precision;
        let half = whole / 2;
        let quarter = whole / 4;

        let mut low: u64 = 0;
        let mut high: u64 = whole;
        let mut bits = Vec::new();
        let mut pending = 0u32;

        for &symbol in data {
            let s = symbol as usize;
            let range = high - low + 1;
            high = low + (range * cum[s + 1]) / self.total - 1;
            low = low + (range * cum[s]) / self.total;

            loop {
                if high < half {
                    bits.push(0);
                    for _ in 0..pending {
                        bits.push(1);
                    }
                    pending = 0;
                } else if low >= half {
                    bits.push(1);
                    for _ in 0..pending {
                        bits.push(0);
                    }
                    pending = 0;
                    low -= half;
                    high -= half;
                } else if low >= quarter && high < 3 * quarter {
                    pending += 1;
                    low -= quarter;
                    high -= quarter;
                } else {
                    break;
                }
                low = low * 2;
                high = high * 2 + 1;
            }
        }

        pending += 1;
        if low < quarter {
            bits.push(0);
            for _ in 0..pending {
                bits.push(1);
            }
        } else {
            bits.push(1);
            for _ in 0..pending {
                bits.push(0);
            }
        }

        bits
    }

    /// Decode arithmetic-coded bits back to data of given length.
    pub fn decode(&self, bits: &[u8], length: usize) -> Vec<u8> {
        if length == 0 || self.total == 0 {
            return Vec::new();
        }

        let cum = self.cumulative();

        let precision: u64 = 32;
        let whole = 1u64 << precision;
        let half = whole / 2;
        let quarter = whole / 4;

        let mut code: u64 = 0;
        let mut bit_pos = 0;

        for _ in 0..precision {
            code = code * 2 + if bit_pos < bits.len() { bits[bit_pos] as u64 } else { 0 };
            bit_pos += 1;
        }

        let mut low: u64 = 0;
        let mut high: u64 = whole;
        let mut result = Vec::with_capacity(length);

        for _ in 0..length {
            let range = high - low + 1;
            let scaled_value = ((code - low + 1) * self.total - 1) / range;

            // Find symbol via linear search
            let mut symbol = 0;
            for i in 0..256 {
                if cum[i + 1] > scaled_value {
                    symbol = i;
                    break;
                }
            }

            result.push(symbol as u8);

            high = low + (range * cum[symbol + 1]) / self.total - 1;
            low = low + (range * cum[symbol]) / self.total;

            loop {
                if high < half {
                    // nothing
                } else if low >= half {
                    code -= half;
                    low -= half;
                    high -= half;
                } else if low >= quarter && high < 3 * quarter {
                    code -= quarter;
                    low -= quarter;
                    high -= quarter;
                } else {
                    break;
                }
                low = low * 2;
                high = high * 2 + 1;
                code = code * 2 + if bit_pos < bits.len() { bits[bit_pos] as u64 } else { 0 };
                bit_pos += 1;
            }
        }

        result
    }

    /// Compute cumulative frequency table.
    fn cumulative(&self) -> Vec<u64> {
        let mut cum = vec![0u64; 257];
        for i in 0..256 {
            cum[i + 1] = cum[i] + self.frequencies[i];
        }
        cum
    }

    /// Get the theoretical bit length for encoding given data.
    pub fn theoretical_bit_length(&self, data: &[u8]) -> f64 {
        let mut bits = 0.0;
        for &b in data {
            let p = self.frequencies[b as usize] as f64 / self.total as f64;
            if p > 0.0 {
                bits += -p.log2();
            }
        }
        bits
    }

    /// Get frequencies reference.
    pub fn frequencies(&self) -> &[u64] {
        &self.frequencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_roundtrip() {
        let data = b"aababcabcd".to_vec();
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let decoded = coder.decode(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_arithmetic_empty() {
        let coder = ArithmeticCoder::from_data(&[]);
        let encoded = coder.encode(&[]);
        assert!(encoded.is_empty());
        let decoded = coder.decode(&encoded, 0);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_arithmetic_repeated() {
        let data = b"aaaaaa".to_vec();
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let decoded = coder.decode(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_arithmetic_binary() {
        let data: Vec<u8> = vec![0, 1, 0, 1, 0, 1, 1, 0];
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let decoded = coder.decode(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_arithmetic_longer() {
        let data = b"the quick brown fox jumps over the lazy dog".to_vec();
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let decoded = coder.decode(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_arithmetic_two_symbols() {
        let data = b"ABABABAB".to_vec();
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let decoded = coder.decode(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_arithmetic_accuracy() {
        // Check that encoded size is close to theoretical minimum
        let data = b"aaaaabbbbbccccc".to_vec(); // 3 symbols, equal frequency
        let coder = ArithmeticCoder::from_data(&data);
        let encoded = coder.encode(&data);
        let theoretical = coder.theoretical_bit_length(&data);
        // Encoded should be within 32 bits of theoretical (overhead from final flush)
        assert!((encoded.len() as f64 - theoretical).abs() < 64.0,
            "encoded {} bits, theoretical {:.1} bits", encoded.len(), theoretical);
    }
}
