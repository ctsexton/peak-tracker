#![feature(generic_const_exprs)]

pub mod analyze;
pub mod buffer;
pub mod fft;
pub mod osc;
pub mod peak;
pub mod plugin;
pub mod reconstructor;
pub mod smooth;
pub mod utils;

use assert_no_alloc::*;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;
