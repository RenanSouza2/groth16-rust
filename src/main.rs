use std::fs::File;
use std::io::{prelude::*};
use std::str::FromStr;
use num_bigint::BigUint;
use num_traits::{One, Zero};

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

struct GF;
impl GF {
    fn q() -> BigUint {
        return BigUint::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495617", 10)
            .unwrap();
    }

    fn w(size: usize) -> BigUint {
        let e = (GF:: q() - BigUint::one()) / (BigUint::from(size));
        return GF::pow(&BigUint::from(5u64), &BigUint::from(e));
    }

    fn add(a: &BigUint, b: &BigUint) -> BigUint {
        return (a + b) % GF::q();
    }

    fn sub(a: &BigUint, b: &BigUint) -> BigUint {
        if a < b {
            return a + GF::q() - b;
        }

        return a - b;
    }

    fn mul(a: &BigUint, b: &BigUint) -> BigUint {
        return (a * b) % GF::q();
    }

    fn inv(a: &BigUint) -> BigUint {
        return a.modinv(&GF::q()).unwrap();
    }

    fn div(a: &BigUint, b: &BigUint) -> BigUint {
        return GF::mul(a, &GF::inv(b));
    }

    fn pow(a: &BigUint, b: &BigUint) -> BigUint {
        return a.modpow(b, &GF::q())
    }
}

#[derive(Debug)]
struct VecSparse {
    n: usize,
    max: usize,
    index: Vec<usize>,
    value: Vec<BigUint>,
}

impl From<Vec<BigUint>> for VecSparse {
    fn from(v: Vec<BigUint>) -> VecSparse {
        let mut res = VecSparse::new(v.len());
        let mut i: usize = 0;
        for value in v {
            res.push(i, value);
            i += 1;
        }
        return  res;
    }
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

    fn push(&mut self, index: usize, value: BigUint) {
        assert!(self.n == 0 || self.index[self.n - 1] < index);
        assert!(index < self.max);
        self.n += 1;
        self.index.push(index);
        self.value.push(value);
    }

    fn split(&self) -> (VecSparse, VecSparse) {
        let half = self.max / 2;
        let mut v0 = VecSparse::new(half);

        let mut i = 0;
        while i < self.n && self.index[i] < half {
            let value = self.value[i].clone();
            v0.push(self.index[i], value);
            i += 1;
        }

        let mut v1 = VecSparse::new(half);
        while i < self.n {
            let value = self.value[i].clone();
            v1.push(self.index[i] - half, value);
            i += 1;
        }

        return (v0, v1);
    }

    fn fft_rec(&self, r: &BigUint) -> Vec<BigUint> {
        if self.n == 0 {
            return vec![BigUint::zero(); self.max];
        }

        if self.max == 1 {
            return vec![self.value[0].clone()];
        }

        let (v0_a, v1_a) = self.split();
        let w_next = GF::mul(&r, &r);
        let v0_b = v0_a.fft_rec(&w_next);
        let v1_b = v1_a.fft_rec(&w_next);

        let mut v1_c: Vec<BigUint> = Vec::new();
        let mut w = BigUint::from(1u64);
        for i in 0..v1_b.len() {
            let mut value = v1_b[i].clone();
            value = GF::mul(&value, &w);
            v1_c.push(value);

            w = GF::mul(&w, &r);
        }

        let mut v0_d: Vec<BigUint> = v0_b
            .iter()
            .zip(v1_c.iter())
            .map(|(a, b)| GF::add(a, b))
            .collect();

        let v1_d: Vec<BigUint> = v0_b
            .iter()
            .zip(v1_c.iter())
            .map(|(a, b)| GF::sub(a, b))
            .collect();

        v0_d.extend(v1_d);
        return v0_d;
    }

    fn fft(&self) -> Vec<BigUint> {
        let w = GF::w(self.max);
        return self.fft_rec(&w);
    }

    fn ifft(&self) -> Vec<BigUint> {
        let w = GF::w(self.max);
        let res = self.fft_rec(&w);
        let i = GF::inv(&BigUint::from(self.max));
        return res
            .iter()
            .map(|value| GF::mul(value, &i))
            .collect();
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
            mt[index].push(i, value);
        }
    }

    return mt;
}

fn file_read_section_constraints(file: &mut File, header: &Header) -> Vec<Vec<VecSparse>> {
    file_seek_section(file, 2);

    let mut m_vec: Vec<Vec<VecSparse>> = (0..3)
        .map(|_| Vec::new())
        .collect();

    for _i in 0..header.n_contraints {
        for j in 0..3 {
            let v = file_read_vec_sparse(file, header);
            m_vec[j].push(v);
        }
    }

    let mut mt_vec: Vec<Vec<VecSparse>> = Vec::new();
    for m in m_vec {
        let mt = matrix_sparse_transpose(m);
        mt_vec.push(mt);
    }

    return mt_vec;
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
    // let file_name = "cases/test.r1cs";
    // let circuit = read_r1cs(file_name);
    // println!("circuit: {:?}", circuit);

    let mut v1 = VecSparse::new(4);
    v1.push(0, BigUint::one());
    let v2 = v1.ifft();
    println!("{:?}", v2);
    let v3 = VecSparse::from(v2).fft();
    println!("{:?}", v3);


    // let mut w = GF::w(32);
    // let mut i = 0u64;
    // println!("{} : {}", i, w);
    // while w != BigUint::one() {
    //     w = GF::mul(&w, &w);
    //     i += 1;
    //     println!("{} : {}", i, w);
    // }

    // let mut q = GF::q() - BigUint::one();
    // let mut s = 0u64;
    // while (q.clone() % BigUint::from(2u64)).is_zero() {
    //     q /= BigUint::from(2u64);
    //     s += 1;
    // }

    // println!("{}", q);
    // println!("{}", s);

    println!();
    Ok(())
}
