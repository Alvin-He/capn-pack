#![no_main]

extern crate capnpack;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = capnpack::unpack(&data, 1024);
});
