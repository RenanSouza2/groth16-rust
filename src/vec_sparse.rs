use num_bigint::BigUint;
use num_traits::Zero;
use std::mem;
use std::ops::Shr;
use crate::gf;

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

fn bit_inv(value: usize, bit_lenth: usize) -> usize {
    let offset = mem::size_of::<usize>() * 8 - bit_lenth;
    return value.reverse_bits().shr(offset);
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

    pub fn with_data(
        n: usize,
        max: usize,
        index: Vec<usize>,
        value: Vec<BigUint>
    ) -> Self {
        return VecSparse  { n, max, index, value };
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

    fn _shuffle(&self) -> VecSparse {
        let mut res = VecSparse::new(self.max);

        let bit_length = self.max.trailing_zeros() as usize;
        for i in 0..self.max {
            let i_inv = bit_inv(i, bit_length);
            let id = self.get(i_inv);
            if id.is_none() {
                continue;
            }

            let value = self.value[id.unwrap()].clone();
            res.push(i, value);
        }

        return res;
    }

    fn _fft_rec(&self, r: &BigUint) -> Vec<BigUint> {
        if self.n == 0 {
            return vec![BigUint::zero(); self.max];
        }

        if self.max == 1 {
            return vec![self.value[0].clone()];
        }

        let (v0_a, v1_a) = self.split();
        let w_next = gf::mul(&r, &r);
        let v0_b = v0_a._fft_rec(&w_next);
        let v1_b = v1_a._fft_rec(&w_next);

        let mut v1_c: Vec<BigUint> = Vec::new();
        let mut w = BigUint::from(1u64);
        for i in 0..v1_b.len() {
            let mut value = v1_b[i].clone();
            value = gf::mul(&value, &w);
            v1_c.push(value);

            w = gf::mul(&w, &r);
        }

        let mut v0_d: Vec<BigUint> = v0_b
            .iter()
            .zip(v1_c.iter())
            .map(|(a, b)| gf::add(a, b))
            .collect();

        let v1_d: Vec<BigUint> = v0_b
            .iter()
            .zip(v1_c.iter())
            .map(|(a, b)| gf::sub(a, b))
            .collect();

        v0_d.extend(v1_d);
        return v0_d;
    }

    pub fn fft(&self) -> Vec<BigUint> {
        let w = gf::w(self.max);
        return self._shuffle()._fft_rec(&w);
    }

    pub fn ifft(&self) -> Vec<BigUint> {
        let w = gf::w(self.max);
        let w_inv = gf::inv(&w);
        let res: Vec<BigUint> = self._shuffle()._fft_rec(&w_inv);

        let i = gf::inv(&BigUint::from(self.max));
        return res
            .iter()
            .map(|value| gf::mul(value, &i))
            .collect();
    }
}

pub fn matrix_transpose(m: Vec<VecSparse>) -> Vec<VecSparse> {
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
