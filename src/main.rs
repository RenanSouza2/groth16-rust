use num_bigint::BigUint;

use crate::{r1cs::R1CS, witness::Witness};

mod file_helpers;
mod vec_sparse;
mod gf;
mod r1cs;
mod witness;
mod fft;

fn main() -> std::io::Result<()> {
    let folder = "3";

    let circuit_file_name = format!("circuit/{}/code.r1cs", folder);
    let circuit = R1CS::read(circuit_file_name.as_str());
    println!("circuit: {:?}", circuit);

    let witness_path_name = format!("circuit/{}/witness.wtns", folder);
    let witness = Witness::read(witness_path_name.as_str());
    println!("witness: {:?}", witness);

    circuit.pub_verify(&witness);
    let h = circuit.comp_h(&witness);
    println!("h: {:?}", h);

    // let v0: Vec<u64> = vec![1, 2, 3, 4];
    // let v1: Vec<BigUint> = v0.into_iter().map(BigUint::from).collect();
    // let mut v2 = fft(&v1);
    // println!("v2: {:?}", v2);
    // v2 = ifft(&v2);
    // println!("v2: {:?}", v2);

    // let size: usize = 512;
    // let w = gf::w(size);
    // let a: Vec<Poly> = (0..size)
    //     .map(|i| {
    //     let wn = gf::pow(&w, &BigUint::from(i));
    //     let data = [gf::opo(&wn), BigUint::one()];
    //     return Poly { data: data.to_vec() };
    // }).collect();
    // let mut res = Poly::one();
    // for i in a.iter() {
    //     res = res.mul(i);
    //     // println!("res: {:?}", res);
    // }
    // println!("res: {:?}", res);

    // println!("q: {:?}", gf::opo(&res.data[0]));

    println!();
    Ok(())
}
