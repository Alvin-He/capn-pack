#![no_main]

extern crate capnpack;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let packed = capnpack::pack(data);
    let unpacked = capnpack::unpack(&packed, packed.len()).unwrap();
    assert_eq!(data, unpacked);
});
