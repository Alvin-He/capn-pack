# capnpack
This is an implementation of [CapnProto](https://capnproto.org/)'s packing algorithm in pure Rust. Along with a few slight modifications to allow the algorithm to work for any binary payload of any size.

# Usage
```rust
// Packing:
let data = vec![1, 0, 0, 0, 2, 3, 4, 5];
let packed = capnpack::pack(&data);

// Un-packing:
let packed = vec![0xF1, 1, 2, 3, 4, 5];
let unpacked_data = capnpack::unpack(&packed, 8).unwrap(); // size hint can be any reasonable number

assert_eq!(unpacked_data, data);
```

*Note: Data packed by this library may not be compatible with other implementations of CapnProto's packing algorithm due to slight modifications to the algorithm to allow for any sized payloads.* 

## Modifications to the packing algorithm
If the input data to the packing algorithm does not align to a multiple of 8 bytes, *and is compressible via the typical format*. Then the remaining data is processed via the typical packing process, but with the corresponding places for the missing bytes set to `1`.

Ex: Say we have data `0, 0, 1, 2, 0`  
The tag will be: `0b 111 01100`. `01100` is the normal packing algorithm, and the `111` is the filler `1`s for the missing bytes.  
This data packed will be: `0b11101100, 1, 2`. 

This allows the decoder/unpacker to know when to stop processing without any additional overhead.

## License & Contribution
This repository is licensed under the MIT license. See [license.md](license.md).

Any and all contributions are welcome. If you encounter a problem, please open an issue.