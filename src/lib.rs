//! Simple samplerate conversion
//!
//! Input and output formats may be any of i16, u16 or f32. Input and
//! output does not have to be the same.
//! The resampling itself is done using f32

pub mod error;
mod filters;
mod filtering;
pub mod samplerate;
mod sample;
