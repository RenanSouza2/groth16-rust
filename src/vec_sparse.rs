use num_bigint::BigUint;
use num_traits::Zero;
use std::fs::File;
use crate::{file_helpers, gf};

#[derive(Debug)]
pub struct VecSparse {
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

    pub fn read(
        file: &mut File,
        field_size: usize,
        max: usize
    ) -> VecSparse {
        let n = file_helpers::read_u64(file, 4) as usize;
        let mut index: Vec<usize> = Vec::new();
        let mut value: Vec<BigUint> = Vec::new();
        for _i in 0..n {
            let idx = file_helpers::read_u64(file, 4) as usize;
            let val = file_helpers::read_big_uint(file, field_size);

            assert!(idx < max);

            index.push(idx);
            value.push(val);
        }

        return VecSparse { n, max, index, value };
    }

    fn get(&self, i: usize) -> Option<usize> {
        for _i in 0..self.n {
            if self.index[_i] == i {
                return Some(_i);
            }
        }
        return None;
    }

    fn push(&mut self, index: usize, value: BigUint) {
        assert!(self.n == 0 || self.index[self.n - 1] < index);
        assert!(index < self.max);

        if value.is_zero() {
            return;
        }

        self.n += 1;
        self.index.push(index);
        self.value.push(value);
    }

    pub fn inner(&self, v: &Vec<BigUint>) -> BigUint {
        let mut res = BigUint::zero();
        for i in 0..self.n {
            let index = self.index[i];
            let value = &self.value[i];
            
            let tmp = gf::mul(value, &v[index]);
            res = gf::add(&res, &tmp);
        }
        return res;
    }
}

#[derive(Debug)]
pub struct MatrixSparce {
    pub data: Vec<VecSparse>
}

impl MatrixSparce {
    pub fn new() -> Self {
        return MatrixSparce { data: Vec::new() };
    }

    pub fn matrix_transpose(&self) -> Self {
        let mut data: Vec<VecSparse> = (0..self.data[0].max)
            .map(|_| VecSparse::new(self.data.len()))
            .collect();

        for i  in 0..self.data.len() {
            let v = &self.data[i];
            for j in 0..v.n {
                let index = v.index[j];
                let value = v.value[j].clone();
                data[index].push(i, value);
            }
        }

        return MatrixSparce { data };
    }

    pub fn mul_vec(&self, v: &Vec<BigUint>) -> Vec<BigUint> {
        return self.data
            .iter()
            .map(|line| line.inner(&v))
            .collect();
    }
}
