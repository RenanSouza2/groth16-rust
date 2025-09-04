use num_bigint::BigUint;
use num_traits::One;

pub fn q() -> BigUint {
    return BigUint::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495617", 10)
        .unwrap();
}

pub fn w(size: usize) -> BigUint {
    let e = (q() - BigUint::one()) / (BigUint::from(size));
    return pow(&BigUint::from(5u64), &BigUint::from(e));
}

pub fn add(a: &BigUint, b: &BigUint) -> BigUint {
    return (a + b) % q();
}

pub fn sub(a: &BigUint, b: &BigUint) -> BigUint {
    if a < b {
        return a + q() - b;
    }

    return a - b;
}

pub fn opo(a: &BigUint) -> BigUint {
    return q() - a;
}

pub fn mul(a: &BigUint, b: &BigUint) -> BigUint {
    return (a * b) % q();
}

pub fn inv(a: &BigUint) -> BigUint {
    return a.modinv(&q()).unwrap();
}

pub fn div(a: &BigUint, b: &BigUint) -> BigUint {
    return mul(a, &inv(b));
}

pub fn pow(a: &BigUint, b: &BigUint) -> BigUint {
    return a.modpow(b, &q())
}
