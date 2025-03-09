// core/field.rs

/// Mersenne prime p = 2^31 - 1
pub const P: u32 = 2147483647;

/// Default φ = (1 + x), where x² = x+1 mod p
pub const PHI_A: u32 = 0;
pub const PHI_B: u32 = 1;

/// Calculate p² - 1 for the multiplicative group order
const P_SQUARED: u64 = (P as u64) * (P as u64);
const GROUP_ORDER: u64 = P_SQUARED - 1;

/// GFp2 element (a + b*x), with x² = x+1 mod p.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GFp2 {
    pub a: u32,
    pub b: u32,
}

/// Compute mod p for p = 2^31-1. Uses two-step reduction.
#[inline]
pub fn modp(x: u64) -> u32 {
    let r = (x >> 31) + (x & ((1 << 31) - 1));
    let r = (r >> 31) + (r & ((1 << 31) - 1));
    let r = r as u32;
    if r >= P { r - P } else { r }
}

/// Multiply a and b mod p.
#[inline]
pub fn mul_mod(a: u32, b: u32) -> u32 {
    let r = modp((a as u64) * (b as u64));
    if r >= P { r - P } else { r }
}

/// Add a and b mod p.
#[inline]
pub fn add_mod(a: u32, b: u32) -> u32 {
    let s = a.wrapping_add(b);
    if s >= P { s - P } else { s }
}

/// Subtract b from a mod p.
#[inline]
pub fn sub_mod(a: u32, b: u32) -> u32 {
    let d = a.wrapping_sub(b);
    if d > a { d.wrapping_add(P) } else { d }
}

/// Multiply in GFp2: (a+b*x)*(c+d*x).
#[inline]
pub fn mul_gfp2(x: &GFp2, y: &GFp2) -> GFp2 {
    let ac = mul_mod(x.a, y.a);
    let bd = mul_mod(x.b, y.b);
    let ad_bc = add_mod(mul_mod(x.a, y.b), mul_mod(x.b, y.a));

    GFp2 {
        a: add_mod(ac, bd),        // ac + bd
        b: add_mod(ad_bc, bd)      // ad + bc + bd
    }
}

/// Convert scalar to GFp2
#[inline]
pub fn to_gfp2(d: u32) -> GFp2 {
    GFp2 { a: d % P, b: 0 }
}

/// Helper function for negative exponent
#[inline]
fn negative_exponent(k: u32) -> u32 {
    let k_reduced = (k as u64 % GROUP_ORDER) as u32;
    ((GROUP_ORDER - k_reduced as u64) % GROUP_ORDER) as u32
}

/// Exponentiate φ by e
pub fn exp_phi(base: GFp2, e: u32) -> GFp2 {
    let e = (e as u64 % GROUP_ORDER) as u32;
    let mut result = GFp2 { a: 1, b: 0 };
    let mut current = base;

    let bits = 32 - e.leading_zeros();

    for i in 0..bits {
        let bit = ((e >> i) & 1) as u32;
        let mask = bit.wrapping_neg();

        let temp = mul_gfp2(&result, &current);
        result = GFp2 {
            a: ((!mask & result.a) | (mask & temp.a)),
            b: ((!mask & result.b) | (mask & temp.b))
        };

        current = mul_gfp2(&current, &current);
    }
    result
}

pub fn exp_phi_inverse(base: GFp2, k: u32) -> GFp2 {
    exp_phi(base, negative_exponent(k))
}

/// Check irreducibility of x² - x - 1
pub fn check_irreducible() -> bool {
    let e = (P - 1) >> 1;
    let mut res = 1u32;
    let mut base = 5u32;
    let mut exp = e;

    while exp > 0 {
        if exp & 1 == 1 {
            res = mul_mod(res, base);
        }
        base = mul_mod(base, base);
        exp >>= 1;
    }
    res == P - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_arithmetic() {
        assert_eq!(modp(P as u64 + 1), 1);

        let a = GFp2 { a: 1, b: 1 };
        let b = GFp2 { a: 1, b: 0 };
        let c = mul_gfp2(&a, &b);
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 1);
    }

    #[test]
    fn test_phi_exponentiation() {
        let base = GFp2 { a: PHI_A, b: PHI_B };
        let exp = exp_phi(base, 1);
        assert_eq!(exp.a, PHI_A);
        assert_eq!(exp.b, PHI_B);
    }
}
