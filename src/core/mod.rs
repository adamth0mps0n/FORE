// core/mod.rs
mod field;
mod transform;
mod system;

pub use field::{
    GFp2, P, PHI_A, PHI_B,
    modp, mul_mod, add_mod, sub_mod,
    mul_gfp2, to_gfp2, exp_phi, exp_phi_inverse,
    check_irreducible
};

pub use transform::{
    binary_haar_transform, apply_phi_transform,
    init_chunk_size, get_chunk_size, BASE_CHUNK_SIZE
};

pub use system::ForeSystem;
