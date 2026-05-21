

#[cfg(test)]
mod tests {
    use capnpack::{pack, unpack};

    #[test]
    fn test_cpnp_normal_bytes() {
        let d1 = [0x01, 0x05, 0x09, 0xff, 0xf3, 0x00, 0x04, 0x00];
        let e1 = [0b01011111, 0x01, 0x05, 0x09, 0xff, 0xf3, 0x04];
        assert_eq!(pack(&d1), e1);
        assert_eq!(unpack(&e1, 100).unwrap(), d1);

        let d2 = [0x05, 0x00, 0x03, 0xfc, 0xcc, 0x00, 0x00, 0x81];
        let e2 = [0b10011101, 0x05, 0x03, 0xfc, 0xcc, 0x81];
        assert_eq!(pack(&d2), e2);
        assert_eq!(unpack(&e2, 100).unwrap(), d2);

        let d3 = [0x00, 0x00, 0x04, 0x05, 0xef];
        let e3 = [0b11111100, 0x04, 0x05, 0xef];
        assert_eq!(pack(&d3), e3);
        assert_eq!(unpack(&e3, 100).unwrap(), d3);
    }

    #[test]
    fn test_cpnp_long() {
        let d: Vec<u8> = (1..10).collect();
        let e = vec![vec![0xff], (1..9).collect(), vec![0x01], (9..10).collect()].concat();
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 10).unwrap(), d);

        let d = vec![(1..10).collect(), vec![0x00], (1..25).collect()].concat();
        let e = vec![vec![0xFF], (1..9).collect(), vec![26, 9, 0x00], (1..25).collect()].concat();
        println!("{:?}", d);
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);
    }

    #[test]
    fn test_cpnp_zeros() {
        let d = [0; 210];
        let e = vec![0x00, 209];
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 10).unwrap(), d);
    }

    #[test]
    fn test_cpnp_combined() {
        let d = vec![vec![0x00], (1..8).collect(), (10..30).collect(), vec![0; 200], vec![0x23, 0x00, 0x00, 0x0c, 0xc3, 0xd5]].concat();
        let e = vec![vec![0b11111110], (1..8).collect(), vec![0xFF], (10..18).collect(), vec![12], (18..30).collect(), vec![0x00, 199], vec![0b11111001, 0x23, 0x0c, 0xc3, 0xd5]].concat();
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);
    }

    #[test]
    fn test_cpnp_smart_pack() {
        let d = vec![(1..10).collect(), vec![0x00], (1..8).collect()].concat();
        let e = vec![vec![0xFF], (1..9).collect(), vec![9, 9, 0x00], (1..8).collect()].concat();
        println!("{:?}", d);
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);
    
        let d = vec![(1..10).collect(), vec![0x00; 2], (1..8).collect()].concat();
        let e = vec![vec![0xFF], (1..9).collect(), vec![10, 9, 0x00, 0x00], (1..8).collect()].concat();
        println!("{:?}", d);
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);

        let d = vec![(1..9).collect(), vec![0x00; 3], (1..8).collect()].concat();
        let e = vec![vec![0xFF], (1..9).collect(), vec![0x00, 0b11111000], (1..=5).collect(), vec![0xFF], (6..8).collect()].concat();
        println!("{:?}", d);
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);
    
        let d = vec![(1..9).collect(), vec![0x00; 2], (1..=6).collect(), (0..8).collect()].concat();
        let e = vec![vec![0xFF], (1..9).collect(), vec![0x00, 0b11111100], (1..=6).collect(), vec![0b11111110], (1..8).collect()].concat();
        println!("{:?}", d);
        assert_eq!(pack(&d), e);
        assert_eq!(unpack(&e, 100).unwrap(), d);
    }

}