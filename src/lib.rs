#![feature(generic_const_exprs)]

pub mod analyzers;
pub mod buffer;
pub mod osc;
pub mod peak;
pub mod plugin;
pub mod reconstructor;
pub mod smooth;
pub mod tracker;
pub mod utils;
pub mod window;

use assert_no_alloc::*;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;
