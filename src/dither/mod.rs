/// Contains all the dithering logic for the error propagation based algorithms.
pub mod error;

/// Contains logic for Ordered / Bayer dithering.
pub mod ordered;

pub use error::{
    FLOYD_STEINBERG,
    JARVIS_JUDICE_NINKE,
    ATKINSON,
    BURKES,
    STUCKI,
    SIERRA,
    SIERRA_TWO_ROW,
    SIERRA_LITE,
};