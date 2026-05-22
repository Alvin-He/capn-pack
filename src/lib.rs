//! # capnpack
//! This is an implementation of [CapnProto](https://capnproto.org/)'s packing algorithm in pure Rust. Along with a few slight modifications to allow the algorithm to work for any binary payload of any size.
//!
//! # Usage
//! ```rust
//! // Packing:
//! let data = vec![1, 0, 0, 0, 2, 3, 4, 5];
//! let packed = capnpack::pack(&data);
//!
//! // Un-packing:
//! let packed = vec![0xF1, 1, 2, 3, 4, 5];
//! let unpacked_data = capnpack::unpack(&packed, 8).unwrap(); // size hint can be any reasonable number
//!
//! assert_eq!(unpacked_data, data);
//! ```
//!
//! *Note: Data packed by this library may not be compatible with other implementations of CapnProto's packing algorithm due to slight modifications to the algorithm to allow for any sized payloads.* 
//!
//! ## Modifications to the packing algorithm
//! If the input data to the packing algorithm does not align to a multiple of 8 bytes, *and is compressible via the typical format*. Then the remaining data is processed via the typical packing process, but with the corresponding places for the missing bytes set to `1`.
//!
//! Ex: Say we have data `0, 0, 1, 2, 0`  
//! The tag will be: `0b 111 01100`. `01100` is the normal packing algorithm, and the `111` is the filler `1`s for the missing bytes.  
//! This data packed will be: `0b11101100, 1, 2`. 
//!
//! This allows the decoder/unpacker to know when to stop processing without any additional overhead.

use std::{error::Error, fmt};

/// Pack some data using Capn-Pack
/// 
/// # Ex:
/// ```
/// // Packing:
/// let data = vec![1, 0, 0, 0, 2, 3, 4, 5];
/// let packed = capnpack::pack(&data);
/// 
/// // Un-packing:
/// let packed = vec![0xF1, 1, 2, 3, 4, 5];
/// let unpacked_data = capnpack::unpack(&packed, 8).unwrap(); // size hint can be any reasonable number
/// 
/// assert_eq!(unpacked_data, data);
/// ```
pub fn pack(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());

    let mut dnext = 0;
    let de = data.len();

    'outer: while dnext + 8 <= de {
        let mut s_packed_buf = [0; 8];
        let mut s_packed_pos = 0;

        let mut tag: u8 = 0x00;
        // process every single byte
        for i in 0..8 {
            let byte = data[dnext];
            dnext += 1;

            let bit = byte != 0;
            s_packed_buf[s_packed_pos] = byte;
            s_packed_pos += bit as usize;

            tag |= (bit as u8) << i;
        }

        match tag {
            0x00 => { // Write tag and keep reading until non zero            
                let dn_initial = dnext;

                // repetedly take untill non zeros
                for _ in 0..31 { // if after 256 bytes (31*8) + 8 there are still zeros, we will just leave it for the next main loop cycle 
                    let dlast = dnext;

                    if dnext + 8 <= de {
                        // 8 byte is chosen as typically a single loop run will find the next non-zero byte
                        // while also not being too performance damaging with zero additions
                        for _ in 0..8 { 
                            dnext += (data[dnext] == 0) as usize;
                        }

                        // dnext increment by 8 means 8 new zeros 
                        // which means that all bytes read were zeros again, so we continue loop
                        // or wise we end because we have hit at least one non zero
                        if dlast + 8 != dnext {
                            break;
                        };
                    } else { // EOF last bytes handling
                        // consuming any more zeros 
                        for _ in 0..(de-dnext) {
                            dnext += (data[dnext] == 0) as usize;
                        }

                        result.push(0x00);
                        result.push((8 - 1) + (dnext - dn_initial) as u8);

                        if dnext == de {
                            return result;
                        } else { // use outer loop extra bytes handling
                            continue 'outer;
                        }
                    }
                } // dnext at the end of this loop shall be at the location of the next non-zero byte or end

                result.push(0x00);
                // dnext - dn_initial is guranteed to be within (0..=248)
                result.push((8 - 1) + (dnext - dn_initial) as u8);
            },
            0xff => { // Write tag and keep consuming until 0x00 byte
                // push tag and and entire data buf as normal
                result.push(0xff);
                result.extend_from_slice(&s_packed_buf);

                let dn_initial = dnext;
                
                let mut cur_chunk_zeros: u32;
                let mut last_chunk_zeros: u32 = 0;
                for _ in 0..31 { // if after 256 byte it's still uncompressable, then we need to start another symbol
                    if dnext + 8 <= de {
                        cur_chunk_zeros = 0;
                        
                        // check the next next 8 bytes for 0s
                        for _ in 0..8 {
                            cur_chunk_zeros += (data[dnext] == 0) as u32;
                            dnext += 1;
                        }

                        // exit when we have at least 3 compressable 0s in the current word and next
                        // this is done so we only compress when we know the next word is also compressable
                        // therefore avoiding penelty when next word is incompressable 
                        if last_chunk_zeros + cur_chunk_zeros >= 3 {
                            // back track then skip forward bytes untill we reach the 1st zero
                            // this is done so we possibly squeeze out one more byte when we can shift the compression window perfectly
                            // so one window could contain 3 or more zeros; If not, then we still get the full performance
                            if last_chunk_zeros > 0 {    
                                dnext -= 16;
                                for _ in 0..16 {
                                    dnext += (data[dnext] != 0) as usize;
                                } // dnext is now at the position of the first 0  
                            } else {
                                dnext -= 8;
                                for _ in 0..8 {
                                    dnext += (data[dnext] != 0) as usize;
                                } // dnext is now at the position of the first 0
                            }

                            break;
                        } else {
                            last_chunk_zeros = cur_chunk_zeros;       
                        }
                    } else { // EOF last bytes handling
                        let bytes_left = de-dnext;

                        // check the next next 8 bytes for 0s
                        cur_chunk_zeros = 0;
                        for _ in 0..bytes_left {
                            cur_chunk_zeros += (data[dnext] == 0) as u32;
                            dnext += 1;
                        }

                        if last_chunk_zeros + cur_chunk_zeros >= 3 {
                            // back track to beginning of last chunk
                            let amount = if dnext - bytes_left - 8 >= dn_initial {
                                bytes_left + 8
                            } else {
                                bytes_left
                            };
                            dnext -= amount;
                            
                            for _ in 0..amount {
                                dnext += (data[dnext] != 0) as usize;
                            } // dnext is now at the position of the first 0
                        
                            // add data
                            result.push((dnext - dn_initial) as u8);
                            result.extend_from_slice(&data[dn_initial..dnext]);
                            
                            continue 'outer;
                        } else {
                            // add all data
                            result.push((dnext - dn_initial) as u8);
                            result.extend_from_slice(&data[dn_initial..dnext]);

                            return result;
                        }
                    }

                } // at the end of this loop, the next 16 bytes have at least 3 0x00s or we are EOF

                result.push((dnext - dn_initial) as u8); // guranteed to be within u8::MIN..=u8::MAX
                result.extend_from_slice(&data[dn_initial..dnext]); // push data
            },
            t => { // write tag and data
                result.push(t);
                result.extend_from_slice(&s_packed_buf[0..s_packed_pos]);
            }
        }
    }

    // handle any left overs, delta is guranteed to be < 8
    let delta = de-dnext;
    if delta > 0 {
        let mut s_packed_buf = [0; 8];
        let mut s_packed_pos = 0;

        let mut tag: u8 = (0x00ff << delta) as u8; // make upper bits one so docoders don't insert uncessary zeros
        for i in 0..delta {
            let byte = data[dnext];
            dnext += 1;

            let bit = byte != 0;
            s_packed_buf[s_packed_pos] = byte;
            s_packed_pos += bit as usize;

            tag |= (bit as u8) << i;
        }

        result.push(tag);
        result.extend_from_slice(&s_packed_buf[0..s_packed_pos]);
    }

    result
}

/// This error indicates `capnpack::unpack` needed more data, but was unable to get it.
/// <br>
/// `self.0` contains the tag that generated this error.
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, PartialOrd, Ord)]
pub struct UnexpectedEOF(u8);
impl fmt::Display for UnexpectedEOF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag {} expected more bytes, but got EOF!", self.0)
    }
}
impl Error for UnexpectedEOF {}

/// Un-pack some data packed using Capn-Pack<br>
/// <br>
/// `expected_size_hint` is a hint for how many bytes you expect to receive.
/// This can be any number, `unpack()` will always allocate an output buffer of at least this size. 
/// Performance greatly increases if `expected_size_hint` is >= what unpack actually needs.
/// <br><br>
/// `unpack()` may return `capnpack::UnexpectedEOF` if ran out of data while unpacking. 
/// However, the lack of this error **does not** guarantee any thing in terms of weather or not the data is valid.
/// 
/// ### Ex:
/// ```
/// // Packing:
/// let data = vec![1, 0, 0, 0, 2, 3, 4, 5];
/// let packed = capnpack::pack(&data);
/// 
/// // Un-packing:
/// let packed = vec![0xF1, 1, 2, 3, 4, 5];
/// let unpacked_data = capnpack::unpack(&packed, 8).unwrap(); // size hint can be any reasonable number
/// 
/// assert_eq!(unpacked_data, data);
/// ```
pub fn unpack(data: &[u8], expected_size_hint: usize) -> Result<Vec<u8>, UnexpectedEOF> {
    let mut result = Vec::with_capacity(expected_size_hint + (expected_size_hint / 16));

    let mut dnext: usize = 0;
    let de = data.len();

    while dnext < de {
        let tag = data[dnext];
        dnext += 1;

        match tag {
            0x00 => {
                if dnext >= de { return Err(UnexpectedEOF(0x00)); }

                let size = data[dnext] as usize + 1;
                dnext += 1;

                result.resize(result.len() + size, 0x00);
            },
            0xff => {
                if dnext + 9 > de { // this can happen for very small uncompresseable bytes
                    // less than 9 bytes, so we'll just give them everything we got
                    result.extend_from_slice(&data[dnext..de]);
                    return Ok(result);
                }

                result.extend_from_slice(&data[dnext..dnext+8]); // copy leading data
                dnext += 8;

                let size = data[dnext] as usize;
                dnext += 1;
                
                if dnext + size > de {
                    return Err(UnexpectedEOF(0xff));
                }

                result.extend_from_slice(&data[dnext..dnext+size]); // copy rest
                dnext += size;
            },
            tag => {
                let n_bytes = LOOKUP_NUM_1_IN_U8[tag as usize];
                dnext -= 1; // switch from forward tracking to back tracking, just so we don't go out of bounds

                // only a non 0x00 and 0xff tagged data would end valid data, so we only need to handle ending in this case
                if dnext + n_bytes < de {
                    for n in 0..8 {
                        let is_bit_non_zero= (tag & (0x01_u8 << n)) >> n; // 0 or 1 
                        dnext += is_bit_non_zero as usize;
                        let byte = data[dnext] & (is_bit_non_zero * 0xFF);
                        result.push(byte);
                    }
                    dnext += 1;
                } else {
                    // number of 0s in tag + number of bytes left would make up the valid portion of the tag
                    let n_zeros = 8 - n_bytes;
                    let bytes_left = de - dnext - 1;
                    let n_valid = n_zeros + bytes_left;

                    // ensure we actually have enough bytes for the valid number of bits
                    let clean_tag = tag & (0xFF >> (8 - n_valid)); // mask off the upper filler zeros
                    let clean_n1 = LOOKUP_NUM_1_IN_U8[clean_tag as usize];
                    if clean_n1 != bytes_left {
                        return Err(UnexpectedEOF(tag));
                    }

                    for n in 0..n_valid {
                        let is_bit_non_zero = (tag & (0x01_u8 << n)) >> n; // 0 or 1
                        dnext += is_bit_non_zero as usize;
                        let byte = data[dnext] & (is_bit_non_zero * 0xFF);
                        result.push(byte);
                    }

                    return Ok(result);
                }
            }
        }
    }

    Ok(result)
}

/// look up table of how many 1s in a byte of u8
const LOOKUP_NUM_1_IN_U8: [usize; 256] = [ // python is the most glorious code generator
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8
];