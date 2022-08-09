#![feature(generic_const_exprs)]

pub mod analyze;
pub mod fft;
pub mod osc;
pub mod peak;
pub mod utils;

use assert_no_alloc::*;
use crate::osc::SinOsc;
use crate::peak::{Peak, PeakAnalyzer, PeakTracker};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

struct SmoothedValue {
    value: f32,
    target: f32,
    steps_to_target: usize,
}

struct Reconstructor {
    oscillators: [SinOsc; 20],
    peak_analyzer: PeakAnalyzer,
    peak_tracker: PeakTracker,
}

impl Reconstructor {
    fn run(&mut self) {
        // copy to internal buffer
        // 
        // peak_analyzer.get_raw_peaks
        // peak_tracker.update_peaks
        let sample_rate = 48000.0;
        const peaks: [Option<Peak>; 20] = [None; 20];
        for (peak, osc) in peaks.iter().zip(self.oscillators.iter_mut()) {
            if let Some(peak) = peak {
                // TODO: Smooth changes
                osc.set_frequency_hz(peak.frequency, sample_rate);
                osc.set_amplitude(peak.amplitude);
                // run osc
            } else {
                // TODO: Ramp to zero
                // run osc if not at zero
                osc.set_amplitude(0.0);
            }
        }
    }
}
