use std::fs::File;
use array_init::array_init;
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::fft::{fft, ifft};
use crate::witness::Witness;
use crate::{file_helpers, gf};
use crate::vec_sparse::{MatrixSparce, VecSparse};



#[derive(Debug)]
struct Header {
    field_size: usize,
    n_wires: usize,
    n_pub_out: u64,
    n_pub_in: u64,
    n_prv_in: u64,
    n_labels: u64,
    n_contraints: usize,
}

impl Header {
    fn read(file: &mut File) -> Self {
        file_helpers::seek_section(file, 1);

        let field_size = file_helpers::read_u64(file, 4) as usize;
        let field = file_helpers::read_big_uint(file, field_size);
        assert!(field == gf::q(), "unsupported curve");
        
        let n_wires = file_helpers::read_u64(file, 4) as usize;
        let n_pub_out = file_helpers::read_u64(file, 4);
        let n_pub_in = file_helpers::read_u64(file, 4);
        let n_prv_in = file_helpers::read_u64(file, 4);
        let n_labels = file_helpers::read_u64(file, 8);
        let n_contraints = file_helpers::read_u64(file, 4) as usize;

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
}

#[derive(Debug)]
struct Constraints {
    data: [MatrixSparce; 3],
}

impl Constraints {
    fn read(file: &mut File, header: &Header) -> Self {
        file_helpers::seek_section(file, 2);

        let mut data: [MatrixSparce; 3] = array_init(|_| MatrixSparce::new());

        for _i in 0..header.n_contraints {
            for j in 0..3 {
                let v = VecSparse::read(file, header.field_size, header.n_wires);
                data[j].data.push(v);
            }
        }

        return Constraints{ data: data };
    }
}

#[derive(Debug)]
pub struct R1CS {
    header: Header,
    constraints: Constraints
}

fn vec_norm(v: &Vec<BigUint>) -> Vec<BigUint> {
    let mut res = v.clone();
    while res.len() > 0 && res[res.len() - 1].is_zero() {
        res.remove(res.len() - 1);
    }

    return res;
}

#[derive(Debug, Clone)]
pub struct Poly {
    pub data: Vec<BigUint>
}

impl Poly {
    pub fn zero() -> Self {
        return Poly { data: Vec::new() };
    }

    pub fn one() -> Self {
        return Poly { data: { vec![ BigUint::one() ]}};
    }

    pub fn get(&self, index: usize) -> BigUint {
        if index >= self.data.len() {
            return  BigUint::zero();
        }

        return self.data[index].clone();
    }

    pub fn set(&mut self, index: usize, value: BigUint) {
        if index >= self.data.len() {
            self.data.resize(index + 1, BigUint::zero());
        }

        self.data[index] = value;
    }

    pub fn sub(&self, b: &Poly) -> Poly {
        let size = self.data.len().max(b.data.len());
        let mut res = self.clone();
        res.data.resize(size, BigUint::zero());
        for i in 0..b.data.len() {
            let mut value = res.data[i].clone();
            value = gf::sub(&value, &b.data[i]);
            res.data[i] = value;
        }
        return res;
    }

    pub fn mul(&self, b: &Poly) -> Poly {
        let mut res = Poly::zero();
        for i in 0..self.data.len() {
            if i%100 == 0 {
                println!("i: {} / {}", i, self.data.len());
            }

            let val_a = self.get(i);
            for j in 0..b.data.len() {
                let mut val_b = b.get(j);  
                val_b = gf::mul(&val_a, &val_b);

                let mut val_c = res.get(i + j);
                val_c = gf::add(&val_c, &val_b);
                res.set(i + j, val_c);
            }
        }
        return res;
    }

    pub fn mul_fast(mut self, mut b: Poly) -> Poly {
        let size: usize = (self.data.len() + b.data.len()).next_power_of_two();
        self.data.resize(size, BigUint::zero());
        b.data.resize(size, BigUint::zero());
        let mut v1 = fft(&self.data);
        let v2 = fft(&b.data);
        for i in 0..size {
            v1[i] = gf::mul(&v1[i], &v2[i]);   
        }
        v1 = ifft(&v1);

        return Poly { data: v1 };
    }

    fn div_t(&self, size: usize) -> Poly {
        let mut v = vec_norm(&self.data);

        if v.len() == 0 {
            return Poly::zero();
        }

        assert!(v.len() > size);
        let mut right = v.split_off(size);
        right = vec_norm(&right);
        v = vec_norm(&v);
        v = v
            .iter()
            .map(gf::opo)
            .collect();
        assert!(right == v);
        
        return Poly { data: v };
    }

    
    fn tag(&self, t: &str) -> Self {
        println!("{}: {:?}", t, self);
        return self.clone();
    }
}

impl R1CS {
    pub fn read(file_path: &str) -> Self {
        let mut file = File::open(file_path).unwrap();
        
        file_helpers::validate_magic_value(&mut file, "r1cs");
        file_helpers::validate_uint(&mut file, 1);

        let header = Header::read(&mut file);
        let constraints = Constraints::read(&mut file, &header);
        return R1CS { header, constraints };
    }

    pub fn pub_verify(&self, w: &Witness) {
        assert!((self.header.n_wires == w.header.n_values), "size mismatch");

        for i in 0..self.header.n_contraints {
            let a = self.constraints.data[0].data[i].inner(&w.values.data);
            let b = self.constraints.data[1].data[i].inner(&w.values.data);
            let c = self.constraints.data[2].data[i].inner(&w.values.data);

            assert!(gf::mul(&a, &b) == c, "constraint not fulfilled {}", i);
        }
    }

    pub fn comp_h(&self, w: &Witness) -> Poly {
        let size = self.header.n_contraints.next_power_of_two();
        let [mut u, mut v, w] = std::array::from_fn(|i| {
            let mut v = self.constraints.data[i].mul_vec(&w.values.data);
            v.resize(size, BigUint::zero());
            return Poly { data: ifft(&v) };
        });
        return u
            .mul_fast(v)
            .sub(&w)
            .div_t(size)
    }
}
