use std::fs::File;
use std::io::{Read, Seek};
use num_bigint::BigUint;

pub fn read_bytes(file: &mut File, n: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; n];
    let n_read = file.read(&mut buffer).unwrap();
    assert!(n_read == n, "Trying to read more bytes than file size");
    return buffer;
}

pub fn read_u64(file: &mut File, n: usize) -> u64 {
    let mut buffer = read_bytes(file, n);
    buffer.resize(8, 0);
    let arr: [u8; 8] = buffer
        .as_slice()
        .try_into()
        .unwrap();
    return u64::from_le_bytes(arr);
}

pub fn read_big_uint(file: &mut File, n: usize) -> BigUint {
    let buffer = read_bytes(file, n);
    return BigUint::from_bytes_le(buffer.as_slice());
}

pub fn seek_section(file: &mut File, section_type: u64) {
    file.seek(std::io::SeekFrom::Start(8)).unwrap();
    let n_sections = read_u64(file, 4);
    for _i in 0..n_sections {
        let cur_section_type = read_u64(file, 4);
        let cur_section_size = read_u64(file, 8);

        if cur_section_type == section_type {
            break;
        }

        file.seek(std::io::SeekFrom::Current(cur_section_size as i64)).unwrap();
    }
}

pub fn validate_magic_value(file: &mut File, magic: &str) {
    let read = read_bytes(file, 4);
    assert!(read.as_slice() == magic.as_bytes(), "wrong magic number at R1CS");
}

pub fn validate_biguint(file: &mut File, value: BigUint) {
    let version = read_big_uint(file, 4);
    assert!(version == value, "version not supported");
}

pub fn validate_uint(file: &mut File, value: u64) {
    validate_biguint(file, BigUint::from(value));
}
