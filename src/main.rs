use ark_bn254::{Fq, G1Affine, G1Projective};
use ark_ec::{AffineRepr, PrimeGroup};
use num_bigint::BigUint;
use num_traits::One;

use crate::r1cs::{Poly, R1CS};
use crate::witness::Witness;

mod fft;
mod file_helpers;
mod gf;
mod r1cs;
mod vec_sparse;
mod witness;

fn main() -> std::io::Result<()> {
    let folder = "4";

    let circuit_file_name = format!("circuit/{}/code.r1cs", folder);
    let circuit = R1CS::read(circuit_file_name.as_str());
    println!("circuit: {:?}", circuit.header);

    let witness_path_name = format!("circuit/{}/witness.wtns", folder);
    let witness = Witness::read(witness_path_name.as_str());
    // println!("witness: {:?}", witness);

    circuit.pub_verify(&witness);
    let h = circuit.comp_h(&witness);
    println!("h: {:?}", h);

    // let one = BigUint::one();
    // let two = BigUint::from(2u64);
    // let three = BigUint::from(3u64);
    // let mut a = Poly { data: vec![one.clone(), two.clone()] };
    // let mut b = Poly { data: vec![two.clone(), three.clone()] };
    // let c = a.mul_fast(&mut b);
    // println!("c: {:?}", c);

    // let v1: [u8; 32] = [0x9d, 0x0d, 0x8f, 0xc5, 0x8d, 0x43, 0x5d, 0xd3, 0x3d, 0x0b, 0xc7, 0xf5, 0x28, 0xeb, 0x78, 0x0a, 0x2c, 0x46, 0x79, 0x78, 0x6f, 0xa3, 0x6e, 0x66, 0x2f, 0xdf, 0x07, 0x9a, 0xc1, 0x77, 0x0a, 0x0e];
    // let n = BigUint::from_bytes_le(&v1);
    // println!("n: {}", n);

    // let arr: [u8; 64] = [
    //     157, 13, 143, 197, 141, 67, 93, 211, 61, 11, 199, 245, 40, 235, 120, 10, 44, 70, 121, 120,
    //     111, 163, 110, 102, 47, 223, 7, 154, 193, 119, 10, 14, 58, 27, 30, 139, 27, 135, 186, 166,
    //     123, 22, 142, 235, 81, 214, 241, 20, 88, 140, 242, 240, 222, 70, 221, 204, 94, 190, 15, 52,
    //     131, 239, 20, 28,
    // ];
    // let a1 = &arr[..32];
    // let a2 = &arr[32..64];
    // println!("a1: {:?}", a1);
    // println!("a2: {:?}", a2);

    // let mut x = BigUint::from_bytes_le(a1);
    // let mut y = BigUint::from_bytes_le(a2);
    // println!("x: {:?}", x);
    // println!("y: {:?}", y);

    // x = gf::from_mont(&x);
    // y = gf::from_mont(&y);

    // let x_f = Fq::from(x);
    // let y_f = Fq::from(y);

    // let point = G1Affine::new(x_f, y_f);

    println!();
    Ok(())
}
