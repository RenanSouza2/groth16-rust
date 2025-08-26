use std::fs::File;
use std::io::{prelude::*};
use std::str::FromStr;
use num_bigint::BigUint;
// use num_traits::FromPrimitive;

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
    n_wires: usize,
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
    
    let n_wires = file_read_u64(file, 4) as usize;
    let n_pub_out = file_read_u64(file, 4);
    let n_pub_in = file_read_u64(file, 4);
    let n_prv_in = file_read_u64(file, 4);
    let n_labels = file_read_u64(file, 8);
    let n_contraints = file_read_u64(file, 4);

    assert!(n_wires as u64 >= n_pub_out + n_pub_in + n_prv_in);

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
struct VecSparse {
    n: usize,
    max: usize,
    index: Vec<usize>,
    value: Vec<BigUint>,
}

impl VecSparse {
    fn new(max: usize) -> Self {
        Self {
            n: 0,
            max,
            index: Vec::new(),
            value: Vec::new()
        }
    }
}

fn vec_sparse_push(v: &mut VecSparse, index: usize, value: BigUint) {
    assert!(v.n == 0 || v.index[v.n - 1] < index);
    assert!(index < v.max);
    v.n += 1;
    v.index.push(index);
    v.value.push(value);
}

fn matrix_sparse_push(M: &mut Vec<VecSparse>, index: usize, v: VecSparse) {
    for i in 0..v.n as usize {
        let value = v.value[i].clone();
        vec_sparse_push(&mut M[v.index[i]], index, value);
    }
}

fn file_read_vec_sparse(file: &mut File, header: &Header) -> VecSparse {
    let n = file_read_u64(file, 4) as usize;
    let mut index: Vec<usize> = Vec::new();
    let mut value: Vec<BigUint> = Vec::new();
    for _i in 0..n {
        let idx = file_read_u64(file, 4) as usize;
        let val = file_read_big_uint(file, header.field_size);

        assert!(idx < header.n_wires);

        index.push(idx);
        value.push(val);
    }

    return VecSparse {
        n,
        max: header.n_wires,
        index: index,
        value: value
    };
}

fn matrix_sparse_transpose(m: Vec<VecSparse>) -> Vec<VecSparse> {
    let mut mt: Vec<VecSparse> = (0..m[0].max)
        .map(|_| VecSparse::new(m.len()))
        .collect();

    for i  in 0..m.len() {
        let v = &m[i];
        for j in 0..v.n {
            let index = v.index[j];
            let value = v.value[j].clone();
            vec_sparse_push(&mut mt[index], i, value);
        }
    }

    return mt;
}

fn file_read_section_constraints(file: &mut File, header: &Header) -> Vec<Vec<VecSparse>> {
    file_seek_section(file, 2);

    let mut M: Vec<Vec<VecSparse>> = (0..3)
        .map(|_| Vec::new())
        .collect();

    for _i in 0..header.n_contraints {
        for j in 0..3 {
            let v = file_read_vec_sparse(file, header);
            M[j].push(v);
        }
    }

    let mut Mt: Vec<Vec<VecSparse>> = Vec::new();
    for m in M {
        let mt = matrix_sparse_transpose(m);
        Mt.push(mt);
    }

    return Mt;
}

#[derive(Debug)]
struct Circuit {
    header: Header,
    constraints: Vec<Vec<VecSparse>>
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
    let file_name = "cases/test.r1cs";
    let circuit = read_r1cs(file_name);
    println!("circuit: {:?}", circuit);

    println!();
    Ok(())
}
