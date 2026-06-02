# compression-algorithms

Compression algorithms, readable in Rust. Learn how your files get smaller.

## What's Inside

| Module | Algorithm | One-line summary |
|---|---|---|
| `huffman` | Huffman coding | Variable-length prefix codes from a frequency table |
| `rle` | Run-Length Encoding (RLE) | Collapse consecutive repeated bytes |
| `lzw` | Lempel-Ziv-Welch (LZW) | Dictionary compression that builds its codebook on the fly |
| `lz77` | LZ77 sliding window | Back-reference compression with a sliding window |
| `bwt` | Burrows-Wheeler Transform | Reversible permutation that clusters similar characters |
| `mtf` | Move-to-Front Transform | Converts BWT output into a stream of small integers |
| `arithmetic` | Arithmetic coding | Encodes a message as a single fractional number in [0, 1) |
| `delta` | Delta + ZigZag + VarInt | Stores differences between consecutive values |
| `entropy` | Shannon entropy & KL divergence | Theoretical lower bounds for lossless compression |

## Quick Start

```rust
use compression_algorithms::*;

// Huffman coding
let data = b"abracadabra";
let coder = HuffmanCoder::from_data(data);
let (packed, padding) = coder.encode_packed(data);
let recovered = coder.decode_packed(&packed, padding);
assert_eq!(recovered, data);

// Entropy analysis
let entropy = Entropy::shannon_entropy(data);
println!("Shannon entropy: {entropy:.3} bits/symbol");

// BWT → MTF → Huffman pipeline
let (bwt, idx) = BwtTransform::forward(data);
let mtf = MtfTransform::forward(&bwt);
let coder = HuffmanCoder::from_data(&mtf);
let (compressed, pad) = coder.encode_packed(&mtf);
```

## Install

```bash
cargo add compression-algorithms
```

## License

MIT OR Apache-2.0
