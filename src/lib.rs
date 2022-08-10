#![feature(generic_const_exprs)]

pub mod analyze;
pub mod buffer;
pub mod fft;
pub mod osc;
pub mod peak;
pub mod smooth;
pub mod utils;

use crate::buffer::Ringbuffer;
use crate::osc::SinOsc;
use crate::peak::{Peak, PeakAnalyzer, PeakTracker};
use crate::smooth::SmoothedValue;
use assert_no_alloc::*;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

struct Reconstructor {
    oscillators: [(SinOsc, SmoothedValue, SmoothedValue); 20],
    peak_analyzer: PeakAnalyzer,
    peak_tracker: PeakTracker,
    buffer: Ringbuffer,
}

impl Reconstructor {
    fn new() -> Self {
        let oscillators = (0..20)
            .map(|_| {
                (
                    SinOsc::new(440.0, 0.0, 0.0),
                    SmoothedValue::new(440.0, 512),
                    SmoothedValue::new(0.0, 512),
                )
            })
            .collect::<Vec<(SinOsc, SmoothedValue, SmoothedValue)>>()
            .try_into()
            .unwrap();
        let peak_analyzer = PeakAnalyzer::new();
        let peak_tracker = PeakTracker::new();
        let buffer = Ringbuffer::new(512);
        Self {
            oscillators,
            peak_analyzer,
            peak_tracker,
            buffer,
        }
    }

    fn run(&mut self, input: &[f32]) {
        assert!(input.len() <= 512);
        for sample in input.iter() {
            self.buffer.write(*sample);
        }
        let mut analysis_sample = [0_f32; 512];
        let mut buffer_reader = self.buffer.get_reader();
        for sample in analysis_sample.iter_mut() {
            *sample = buffer_reader.next().unwrap();
        }
        let raw_peaks = self.peak_analyzer.get_raw_peaks(&analysis_sample);
        self.peak_tracker.update_peaks(raw_peaks);
        let peaks = self.peak_tracker.latest();

        let sample_rate = 48000.0;
        for (peak, (osc, freq_smoother, amp_smoother)) in
            peaks.iter().zip(self.oscillators.iter_mut())
        {
            if let Some(peak) = peak {
                // TODO: Smooth changes
                freq_smoother.set_target(peak.frequency);
                amp_smoother.set_target(peak.amplitude);
                osc.set_frequency_hz(freq_smoother.next(), sample_rate);
                osc.set_amplitude(amp_smoother.next());
                // run osc
            } else {
                // TODO: Ramp to zero
                // run osc if not at zero
                amp_smoother.set_target(0.0);
                osc.set_amplitude(amp_smoother.next());
            }
        }
    }
}
