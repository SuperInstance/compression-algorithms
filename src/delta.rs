//! Delta encoding for compressing correlated numerical sequences.

use serde::{Deserialize, Serialize};

/// Delta codec for encoding differences between consecutive values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaCodec;

impl DeltaCodec {
    /// Encode by storing differences between consecutive bytes.
    pub fn encode(data: &[u8]) -> Vec<i16> {
        if data.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(data.len());
        result.push(data[0] as i16);
        for i in 1..data.len() {
            let delta = data[i] as i16 - data[i - 1] as i16;
            result.push(delta);
        }
        result
    }

    /// Decode delta-encoded sequence.
    pub fn decode(deltas: &[i16]) -> Vec<u8> {
        if deltas.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(deltas.len());
        let mut current = deltas[0];
        result.push(current as u8);
        for i in 1..deltas.len() {
            current += deltas[i];
            result.push(current as u8);
        }
        result
    }

    /// Encode ZigZag + VarInt style: delta then pack into bytes.
    /// ZigZag: 0->0, -1->1, 1->2, -2->3, ...
    pub fn encode_zigzag(data: &[u8]) -> Vec<u8> {
        let deltas = Self::encode(data);
        let mut result = Vec::new();
        for &d in &deltas {
            let zigzag = Self::zigzag_encode(d);
            result.extend_from_slice(&Self::varint_encode(zigzag));
        }
        result
    }

    /// Decode ZigZag + VarInt encoded data. Need original length.
    pub fn decode_zigzag(data: &[u8], original_len: usize) -> Vec<u8> {
        let mut deltas = Vec::with_capacity(original_len);
        let mut pos = 0;
        while pos < data.len() && deltas.len() < original_len {
            let (value, bytes_read) = Self::varint_decode(data, pos);
            deltas.push(Self::zigzag_decode(value));
            pos += bytes_read;
        }
        Self::decode(&deltas)
    }

    fn zigzag_encode(n: i16) -> u16 {
        ((n << 1) ^ (n >> 15)) as u16
    }

    fn zigzag_decode(n: u16) -> i16 {
        ((n >> 1) as i16) ^ -((n & 1) as i16)
    }

    fn varint_encode(mut n: u16) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let mut byte = (n & 0x7F) as u8;
            n >>= 7;
            if n != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if n == 0 {
                break;
            }
        }
        result
    }

    fn varint_decode(data: &[u8], mut pos: usize) -> (u16, usize) {
        let mut result: u16 = 0;
        let mut shift = 0;
        let start = pos;
        loop {
            let byte = data[pos];
            result |= ((byte & 0x7F) as u16) << shift;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        (result, pos - start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_roundtrip() {
        let data = vec![10, 12, 15, 20, 25];
        let encoded = DeltaCodec::encode(&data);
        let decoded = DeltaCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_delta_constant() {
        let data = vec![42; 10];
        let encoded = DeltaCodec::encode(&data);
        assert_eq!(encoded[0], 42);
        for i in 1..encoded.len() {
            assert_eq!(encoded[i], 0);
        }
        let decoded = DeltaCodec::decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_delta_empty() {
        let encoded = DeltaCodec::encode(&[]);
        assert!(encoded.is_empty());
        let decoded = DeltaCodec::decode(&encoded);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_delta_zigzag_roundtrip() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = DeltaCodec::encode_zigzag(&data);
        let decoded = DeltaCodec::decode_zigzag(&encoded, data.len());
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_delta_zigzag_monotonic() {
        let data: Vec<u8> = (0..50).map(|i| (i * 2) as u8).collect();
        let encoded = DeltaCodec::encode_zigzag(&data);
        let decoded = DeltaCodec::decode_zigzag(&encoded, data.len());
        assert_eq!(decoded, data);
    }
}
