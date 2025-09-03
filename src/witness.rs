use std::fs::File;

use num_bigint::BigUint;

use crate::{file_helpers, gf};

#[derive(Debug)]
struct Header {
    field_size: usize,
    n_values: usize,
}

impl Header {
    fn read(file: &mut File) -> Self {
        file_helpers::seek_section(file, 1);

        let field_size = file_helpers::read_u64(file, 4) as usize;
        let field = file_helpers::read_big_uint(file, field_size);
        assert!(field == gf::q(), "unsupported curve");

        let n_values = file_helpers::read_u64(file, 4) as usize;

        return Header { field_size, n_values };
    }
}

#[derive(Debug)]
struct Values {
    data: Vec<BigUint>
}

impl Values {
    fn read(file: &mut File, header: &Header) -> Self {
        file_helpers::seek_section(file, 2);

        let mut data: Vec<BigUint> = Vec::new();
        for _i in 0..header.n_values {
            let value = file_helpers::read_big_uint(file, header.field_size);
            data.push(value);
        }

        return Values { data };
    }
}

#[derive(Debug)]
pub struct Witness {
    header: Header,
    values: Values,
}

impl Witness {
    pub fn read(file_path: &str) -> Self {
        let mut file = File::open(file_path).unwrap();
            
        file_helpers::validate_magic_value(&mut file, "wtns");
        file_helpers::validate_uint(&mut file, 2);

        let header = Header::read(&mut file);
        let values = Values::read(&mut file, &header);

        return Witness { header, values };
    }
}
