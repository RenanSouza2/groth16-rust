mod file_helpers;
mod vec_sparse;
mod gf;
mod r1cs;
mod witness;

use crate::{r1cs::R1CS, witness::Witness};

fn main() -> std::io::Result<()> {
    let circuit_file_name = "circuit/code.r1cs";
    let circuit = R1CS::read(circuit_file_name);
    println!("circuit: {:?}", circuit);

    let witness_path_name = "circuit/witness.wtns";
    let witness = Witness::read(witness_path_name);
    println!("witness: {:?}", witness);

    // let v0: Vec<u64> = vec![0, 1, 0, 0];
    // let v1: Vec<BigUint> = v0.into_iter().map(BigUint::from).collect();
    // let v2 = VecSparse::from(v1);
    // let mut v3 = v2.fft();
    // println!("v3: {:?}", v3);

    // let w = gf::w(4);
    // println!("w: {}", w);
    // let q = gf::q();
    // println!("w: {}", q);
    // let v = gf::mul(&v3[1], &v3[3]);
    // println!("a: {}", v);
    
    // // for _i in 1..4 {
    // //     let v4 = VecSparse::from(v3);
    // //     v3 = v4.fft();
    // //     println!("v3: {:?}", v3);
    // // }

    // let v4 = VecSparse::from(v3);
    // v3 = v4.ifft();
    // println!("v3: {:?}", v3);

    println!();
    Ok(())
}
