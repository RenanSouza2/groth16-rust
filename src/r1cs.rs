use std::fs::File;
use std::str::FromStr;

use num_bigint::BigUint;

use crate::{file_helpers, vec_sparse};
use crate::vec_sparse::VecSparse;

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

#[derive(Debug)]
pub struct R1CS {
    header: Header,
    constraints: Vec<Vec<VecSparse>>
}

fn file_read_section_header(file: &mut File) -> Header {
    file_helpers::seek_section(file, 1);

    let field_size = file_helpers::read_u64(file, 4) as usize;
    let field = file_helpers::read_big_uint(file, field_size);
    let supported = BigUint::from_str("21888242871839275222246405745257275088548364400416034343698204186575808495617").unwrap();
    assert!(field == supported, "unsupported curve");
    
    let n_wires = file_helpers::read_u64(file, 4) as usize;
    let n_pub_out = file_helpers::read_u64(file, 4);
    let n_pub_in = file_helpers::read_u64(file, 4);
    let n_prv_in = file_helpers::read_u64(file, 4);
    let n_labels = file_helpers::read_u64(file, 8);
    let n_contraints = file_helpers::read_u64(file, 4);

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

fn file_read_section_constraints(file: &mut File, header: &Header) -> Vec<Vec<VecSparse>> {
    file_helpers::seek_section(file, 2);

    let mut m_vec: Vec<Vec<VecSparse>> = (0..3)
        .map(|_| Vec::new())
        .collect();

    for _i in 0..header.n_contraints {
        for j in 0..3 {
            let v = file_helpers::file_read_vec_sparse(file, header.field_size, header.n_wires);
            m_vec[j].push(v);
        }
    }

    let mut mt_vec: Vec<Vec<VecSparse>> = Vec::new();
    for m in m_vec {
        let mt = vec_sparse::matrix_transpose(m);
        mt_vec.push(mt);
    }

    return mt_vec;
}

impl R1CS {
    pub fn read(file_path: &str) -> Self {
        let mut file = File::open(file_path).unwrap();
        
        file_helpers::validate_magic_value(&mut file, "r1cs");
        file_helpers::validate_uint(&mut file, 1);

        let header = file_read_section_header(&mut file);
        let constraints = file_read_section_constraints(&mut file, &header);
        return R1CS { header, constraints };
    }
}
