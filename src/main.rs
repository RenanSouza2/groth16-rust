use num_bigint::BigUint;
use num_traits::One;

use crate::r1cs::{Poly, R1CS};
use crate::witness::Witness;

mod file_helpers;
mod vec_sparse;
mod gf;
mod r1cs;
mod witness;
mod fft;

fn main() -> std::io::Result<()> {
    let folder = "4";

    let circuit_file_name = format!("circuit/{}/code.r1cs", folder);
    let circuit = R1CS::read(circuit_file_name.as_str());
    // println!("circuit: {:?}", circuit);

    let witness_path_name = format!("circuit/{}/witness.wtns", folder);
    let witness = Witness::read(witness_path_name.as_str());
    // println!("witness: {:?}", witness);

    circuit.pub_verify(&witness);
    let h = circuit.comp_h(&witness);
    // println!("h: {:?}", h);

    // let one = BigUint::one();
    // let two = BigUint::from(2u64);
    // let three = BigUint::from(3u64);
    // let mut a = Poly { data: vec![one.clone(), two.clone()] };
    // let mut b = Poly { data: vec![two.clone(), three.clone()] };
    // let c = a.mul_fast(&mut b);
    // println!("c: {:?}", c);

    println!();
    Ok(())
}
