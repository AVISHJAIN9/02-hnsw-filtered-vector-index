//! Hardware-accelerated SIMD and scalar vector distance microkernels.

pub mod distance;
pub mod error;

pub use distance::{cosine_distance, distance, inner_product_distance, l2_distance};
