use std::mem;
use std::ops::Shr;
use num_bigint::BigUint;

use crate::gf;

fn bit_inv(value: usize, bit_lenth: usize) -> usize {
    let offset = mem::size_of::<usize>() * 8 - bit_lenth;
    return value.reverse_bits().shr(offset);
}

fn _shuffle(v: &Vec<BigUint>) -> Vec<BigUint> {
    if v.len() == 1 {
        return v.clone();
    }

    let mut res: Vec<BigUint> = Vec::new();

    let bit_length = v.len().trailing_zeros() as usize;
    for i in 0..v.len() {
        let i_inv = bit_inv(i, bit_length);
        let value = v[i_inv].clone();
        res.push(value);
    }

    return res;
}

fn _fft_rec(v: &Vec<BigUint>, begin: usize, size: usize, r: &BigUint) -> Vec<BigUint> {
    if size == 1 {
        return vec![v[begin].clone()];
    }

    let w_next = gf::mul(&r, &r);
    let v0_b = _fft_rec(&v, begin, size / 2, &w_next);
    let v1_b = _fft_rec(&v, begin + size / 2, size / 2, &w_next);

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

pub fn fft(v: &Vec<BigUint>) -> Vec<BigUint> {
    let w = gf::w(v.len());
    let v_sh = _shuffle(&v);
    return _fft_rec(&v_sh, 0, v.len(), &w);
}

pub fn ifft(v: &Vec<BigUint>) -> Vec<BigUint> {
    let w = gf::w(v.len());
    let w_inv = gf::inv(&w);
    let mut v_res = _shuffle(&v);
    v_res = _fft_rec(&v_res, 0, v.len(), &w_inv);

    let i = gf::inv(&BigUint::from(v.len()));
    return v_res.iter().map(|value| gf::mul(value, &i)).collect();
}
