use crate::buffer::Ringbuffer;
use crate::osc::SinOsc;
use crate::peak::{Peak, PeakAnalyzer, PeakTracker};
use crate::smooth::SmoothedValue;

pub struct Reconstructor {
    oscillators: [(SinOsc, SmoothedValue, SmoothedValue); 20],
    peak_analyzer: PeakAnalyzer,
    peak_tracker: PeakTracker,
    buffer: Ringbuffer,
}

impl Reconstructor {
    pub fn new() -> Self {
        let oscillators = (0..20)
            .map(|_| {
                (
                    SinOsc::new(440.0, 0.0, 0.0),
                    SmoothedValue::new(440.0, 64),
                    SmoothedValue::new(0.0, 64),
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

    pub fn run(&mut self, input: &[f32], output: &mut [f32]) {
        assert!(input.len() <= 512 && output.len() == input.len());
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
                freq_smoother.set_target(peak.frequency);
                amp_smoother.set_target(peak.amplitude);
                for sample in output.iter_mut() {
                    osc.set_frequency_hz(freq_smoother.next(), sample_rate);
                    osc.set_amplitude(amp_smoother.next());
                    *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
                }
            } else {
                amp_smoother.set_target(0.0);
                for sample in output.iter_mut() {
                    osc.set_amplitude(amp_smoother.next());
                    *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
                }
            }
        }
    }
}
