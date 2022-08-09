#![feature(generic_const_exprs)]

pub mod analyze;
pub mod fft;
pub mod osc;
pub mod peak;
pub mod utils;

use assert_no_alloc::*;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

struct SmoothedValue {
    value: f32,
    target: f32,
    steps_to_target: usize,
}
