//! Huffman coding: frequency analysis, tree construction, encoding/decoding.

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// A node in the Huffman tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HuffmanNode {
    Leaf { symbol: u8, weight: u64 },
    Internal { left: Box<HuffmanNode>, right: Box<HuffmanNode>, weight: u64 },
}

impl HuffmanNode {
    pub fn weight(&self) -> u64 {
        match self {
            HuffmanNode::Leaf { weight, .. } => *weight,
            HuffmanNode::Internal { weight, .. } => *weight,
        }
    }

    /// Build code table by traversing the tree.
    pub fn build_codes(&self) -> HashMap<u8, Vec<u8>> {
        let mut codes = HashMap::new();
        self.build_codes_inner(&mut codes, &mut Vec::new());
        codes
    }

    fn build_codes_inner(&self, codes: &mut HashMap<u8, Vec<u8>>, prefix: &mut Vec<u8>) {
        match self {
            HuffmanNode::Leaf { symbol, .. } => {
                codes.insert(*symbol, prefix.clone());
            }
            HuffmanNode::Internal { left, right, .. } => {
                prefix.push(0);
                left.build_codes_inner(codes, prefix);
                prefix.pop();
                prefix.push(1);
                right.build_codes_inner(codes, prefix);
                prefix.pop();
            }
        }
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight() == other.weight()
    }
}
impl Eq for HuffmanNode {}
impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight().cmp(&self.weight())
    }
}

/// Huffman coder for encoding and decoding byte data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuffmanCoder {
    tree: HuffmanNode,
    codes: HashMap<u8, Vec<u8>>,
}

impl HuffmanCoder {
    /// Build a Huffman coder from raw data by frequency analysis.
    pub fn from_data(data: &[u8]) -> Self {
        let freq = Self::frequency_analysis(data);
        Self::from_frequencies(&freq)
    }

    /// Perform frequency analysis on data.
    pub fn frequency_analysis(data: &[u8]) -> HashMap<u8, u64> {
        let mut freq = HashMap::new();
        for &b in data {
            *freq.entry(b).or_insert(0) += 1;
        }
        freq
    }

    /// Build a Huffman coder from a frequency map.
    pub fn from_frequencies(freq: &HashMap<u8, u64>) -> Self {
        let mut heap: BinaryHeap<HuffmanNode> = BinaryHeap::new();

        for (&symbol, &weight) in freq {
            heap.push(HuffmanNode::Leaf { symbol, weight });
        }

        if heap.is_empty() {
            let tree = HuffmanNode::Leaf { symbol: 0, weight: 0 };
            return HuffmanCoder { tree: tree.clone(), codes: HashMap::new() };
        }
        if heap.len() == 1 {
            let leaf = heap.pop().unwrap();
            let tree = HuffmanNode::Internal {
                left: Box::new(leaf.clone()),
                right: Box::new(HuffmanNode::Leaf { symbol: 0, weight: 0 }),
                weight: leaf.weight(),
            };
            let codes = tree.build_codes();
            return HuffmanCoder { tree, codes };
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            let weight = left.weight() + right.weight();
            heap.push(HuffmanNode::Internal {
                left: Box::new(left),
                right: Box::new(right),
                weight,
            });
        }

        let tree = heap.pop().unwrap();
        let codes = tree.build_codes();
        HuffmanCoder { tree, codes }
    }

    /// Encode data to bits.
    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        let mut bits = Vec::with_capacity(data.len() * 8);
        for &b in data {
            if let Some(code) = self.codes.get(&b) {
                bits.extend_from_slice(code);
            }
        }
        bits
    }

    /// Encode data to packed bytes. Returns (packed_bytes, padding_bits).
    pub fn encode_packed(&self, data: &[u8]) -> (Vec<u8>, usize) {
        let bits = self.encode(data);
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
        (packed, padding)
    }

    /// Decode bits back to data.
    pub fn decode(&self, bits: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut node = &self.tree;

        for &bit in bits {
            node = match node {
                HuffmanNode::Internal { left, right, .. } => {
                    if bit == 0 { left } else { right }
                }
                HuffmanNode::Leaf { .. } => node,
            };

            if let HuffmanNode::Leaf { symbol, .. } = node {
                result.push(*symbol);
                node = &self.tree;
            }
        }

        result
    }

    /// Decode packed bytes (with padding) back to data.
    pub fn decode_packed(&self, packed: &[u8], padding: usize) -> Vec<u8> {
        let bits: Vec<u8> = packed.iter()
            .flat_map(|&b| (0..8).rev().map(move |i| (b >> i) & 1))
            .take(packed.len() * 8 - padding)
            .collect();
        self.decode(&bits)
    }

    /// Get the code table.
    pub fn codes(&self) -> &HashMap<u8, Vec<u8>> {
        &self.codes
    }

    /// Get reference to the tree.
    pub fn tree(&self) -> &HuffmanNode {
        &self.tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_roundtrip() {
        let data = b"hello world".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let (packed, padding) = coder.encode_packed(&data);
        let decoded = coder.decode_packed(&packed, padding);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_bits_roundtrip() {
        let data = b"abracadabra".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        let decoded = coder.decode(&bits);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_single_char() {
        let data = b"aaaaa".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        let decoded = coder.decode(&bits);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_frequency() {
        let data = b"aabbbcccc".to_vec();
        let freq = HuffmanCoder::frequency_analysis(&data);
        assert_eq!(freq[&b'a'], 2);
        assert_eq!(freq[&b'b'], 3);
        assert_eq!(freq[&b'c'], 4);
    }

    #[test]
    fn test_huffman_code_prefix_free() {
        let data = b"abcdefg".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let codes = coder.codes();
        // Verify no code is a prefix of another
        let code_list: Vec<&Vec<u8>> = codes.values().collect();
        for i in 0..code_list.len() {
            for j in 0..code_list.len() {
                if i != j {
                    let a = code_list[i];
                    let b = code_list[j];
                    assert!(a.len() > b.len() || a != &b[..a.len()],
                        "Code {:?} is a prefix of {:?}", a, b);
                }
            }
        }
    }

    #[test]
    fn test_huffman_repeated_pattern() {
        let data = b"abcabcabcabcabcabcabcabc".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        let decoded = coder.decode(&bits);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_two_chars() {
        let data = b"ab".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        assert_eq!(bits.len(), 2); // One bit each
        let decoded = coder.decode(&bits);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        let decoded = coder.decode(&bits);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_packed_roundtrip() {
        let data = b"the quick brown fox jumps over the lazy dog".to_vec();
        let coder = HuffmanCoder::from_data(&data);
        let (packed, padding) = coder.encode_packed(&data);
        let decoded = coder.decode_packed(&packed, padding);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_huffman_compression_ratio() {
        // Highly repetitive data should compress well
        let data = vec![b'a'; 1000];
        let coder = HuffmanCoder::from_data(&data);
        let bits = coder.encode(&data);
        // Should be ~1000 bits for single symbol (1 bit each)
        assert!(bits.len() < data.len() * 2);
    }
}
