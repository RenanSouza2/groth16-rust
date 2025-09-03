use std::fs::File;
use array_init::array_init;


use crate::{file_helpers, gf, vec_sparse};
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
}

#[derive(Debug)]
struct Constraints {
    data: [Vec<VecSparse>; 3],
}

impl Constraints {
    fn read(file: &mut File, header: &Header) -> Self {
        file_helpers::seek_section(file, 2);

        let mut m_vec: [Vec<VecSparse>; 3] = array_init(|_| Vec::new());

        for _i in 0..header.n_contraints {
            for j in 0..3 {
                let v = VecSparse::read(file, header.field_size, header.n_wires);
                m_vec[j].push(v);
            }
        }

        return Constraints{ data: m_vec.map(vec_sparse::matrix_transpose) };
    }
}

#[derive(Debug)]
pub struct R1CS {
    header: Header,
    constraints: Constraints
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
}
