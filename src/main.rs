use std::fs::File;
use std::io::{prelude::*};
use std::str::FromStr;
use num_bigint::BigUint;

fn file_read_bytes(file: &mut File, n: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; n];
    let n_read = file.read(&mut buffer).unwrap();
    assert!(n_read == n, "Trying to read more bytes than file size");
    return buffer;
}

fn file_read_u64(file: &mut File, n: usize) -> u64 {
    let mut buffer = file_read_bytes(file, n);
    buffer.resize(8, 0);
    let arr: [u8; 8] = buffer
        .as_slice()
        .try_into()
        .unwrap();
    return u64::from_le_bytes(arr);
}

fn file_read_big_uint(file: &mut File, n: usize) -> BigUint {
    let buffer = file_read_bytes(file, n);
    return BigUint::from_bytes_le(buffer.as_slice());
}

fn file_seek_section(file: &mut File, section_type: u64)
{
    file.seek(std::io::SeekFrom::Start(8)).unwrap();
    let n_sections = file_read_u64(file, 4);
    for _i in 0..n_sections {
        let cur_section_type = file_read_u64(file, 4);
        let cur_section_size = file_read_u64(file, 8);

        if cur_section_type == section_type {
            break;
        }

        file.seek(std::io::SeekFrom::Current(cur_section_size as i64)).unwrap();
    }
}

#[derive(Debug)]
struct Header {
    field_size: usize,
    n_wires: u64,
    n_pub_out: u64,
    n_pub_in: u64,
    n_prv_in: u64,
    n_labels: u64,
    n_contraints: u64,
}

fn file_read_section_header(file: &mut File) -> Header {
    file_seek_section(file, 1);

    let field_size = file_read_u64(file, 4) as usize;
    let field = file_read_big_uint(file, field_size);
    let supported = BigUint::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap();
    assert!(field == supported, "unsupported curve");
    
    let n_wires = file_read_u64(file, 4);
    let n_pub_out = file_read_u64(file, 4);
    let n_pub_in = file_read_u64(file, 4);
    let n_prv_in = file_read_u64(file, 4);
    let n_labels = file_read_u64(file, 8);
    let n_contraints = file_read_u64(file, 4);

    assert!(n_wires >= n_pub_out + n_pub_in + n_prv_in);

    return Header {
        field_size,
        n_wires,
        n_pub_out,
        n_pub_in,
        n_prv_in,
        n_labels,
        n_contraints,
    }
}


#[derive(Debug)]
struct Constraint_element {
    n: u64,
    id: Vec<u64>,
    m: Vec<BigUint>
}

#[derive(Debug)]
struct Constraint {
    a: Constraint_element,
    b: Constraint_element,
    c: Constraint_element
}

fn file_read_constraint_element(file: &mut File, header: &Header) -> Constraint_element {
    let n = file_read_u64(file, 4);
    let mut id_vec: Vec<u64> = Vec::new();
    let mut m_vec: Vec<BigUint> = Vec::new();
    for _i in 0..n {
        let id = file_read_u64(file, 4);
        let m = file_read_big_uint(file, header.field_size);

        assert!(id < header.n_wires);

        id_vec.push(id);
        m_vec.push(m);
    }

    return Constraint_element {
        n,
        id: id_vec,
        m: m_vec
    };
}

#[derive(Debug)]
struct Circuit {
    header: Header,
    constraints: Vec<Constraint>
}

fn file_read_section_constraints(file: &mut File, header: &Header) -> Vec<Constraint> {
    file_seek_section(file, 2);

    let mut constraints: Vec<Constraint> = Vec::new();
    for _i in 0..header.n_contraints {
        let a = file_read_constraint_element(file, header);
        let b = file_read_constraint_element(file, header);
        let c = file_read_constraint_element(file, header);

        constraints.push(Constraint { a, b, c });
    }
    return constraints;
}

fn read_r1cs(file_path: &str) -> Circuit {
    let mut file = File::open(file_path).unwrap();
    
    let magic = file_read_bytes(&mut file, 4);
    assert!(magic.as_slice() == &[0x72, 0x31, 0x63, 0x73], "wrong magic number at R1CS");

    let version = file_read_bytes(&mut file, 4);
    assert!(version.as_slice() == &[1, 0, 0, 0], "version not supported");

    let header = file_read_section_header(&mut file);
    let constraints = file_read_section_constraints(&mut file, &header);
    return Circuit { header, constraints };
}

fn main() -> std::io::Result<()> {
    let file_name = "cases/code.r1cs";
    let circuit = read_r1cs(file_name);
    println!("circuit: {:?}", circuit);

    println!();
    Ok(())
}
