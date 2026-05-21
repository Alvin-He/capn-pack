
use std::{fs::File, io::{self, Read}};

use capnpack;

mod tests;

fn capnpack_full(data: Vec<u8>) {
    println!("{:?}", data);

    let packed = capnpack::pack(&data);

    println!("{:?}", packed);

    let unpacked = capnpack::unpack(&packed, data.len()).unwrap();
    assert_eq!(data, unpacked);
}

fn unpack_possibly_invalid(data: Vec<u8>) {
    let r = capnpack::unpack(&data, 1024);
    println!("{}", r.is_ok());
}

fn main() {
    // let mut f = File::open("
    //         fuzz/artifacts/capnpack_full/crash-81738122842d48ecf56cc6813a5141b0a3b4d77d
    // ".trim()).unwrap();

    // let mut data = Vec::new();
    // f.read_to_end(&mut data).unwrap();
    // drop(f);

    let data = vec![0xff, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xC3];

    unpack_possibly_invalid(data);
}