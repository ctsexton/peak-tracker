use crate::analyzers::quadratic::PeakAnalyzer;
use crate::buffer::Ringbuffer;
use crate::osc::SinOsc;
use crate::peak::Peak;
use crate::smooth::SmoothedValue;
use crate::tracker::PeakTracker;
use crate::voice::{Note, Voice};
use dasp::signal::noise;

struct Smoothers {
    freq: SmoothedValue,
    amp: SmoothedValue,
    transpose: SmoothedValue,
    random: SmoothedValue,
    detune: SmoothedValue,
}

struct Oscillator {
    osc: SinOsc,
    smoothers: Smoothers,
}

struct ReconstructorVoice {
    sample_rate: f32,
    note: Option<Note>,
    oscillators: [Oscillator; 20],
}

const MIDDLE_C: u8 = 60; // Midi note num for center

impl Voice for ReconstructorVoice {
    fn get_note(&self) -> &Option<Note> {
        &self.note
    }

    fn set_note(&mut self, note: Option<Note>) {
        self.note = note;
    }

    fn render_block(&mut self, block: &mut [f32]) {
        // TODO: Render the note envelope
        if let Some(note) = &self.note {
            let note_offset = MIDDLE_C - note.note_number;
        }
        for Oscillator { osc, smoothers } in self.oscillators.iter_mut() {
            for sample in block.iter_mut() {
                let rand_amount = 2_f32
                    .powf(smoothers.random.next() * 2.0 * smoothers.detune.next())
                    .clamp(0.25, 4.0);
                osc.set_frequency_hz(
                    smoothers.freq.next() * smoothers.transpose.next() * rand_amount,
                    self.sample_rate,
                );
                osc.set_amplitude(smoothers.amp.next());
                *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
            }
        }
    }
}

impl ReconstructorVoice {
    fn prepare_oscillators(
        &mut self,
        peaks: &[Option<Peak>],
        freeze: bool,
        transpose: f32,
        detune: f32,
    ) {
        for (peak, Oscillator { osc, smoothers }) in peaks.iter().zip(self.oscillators.iter_mut()) {
            smoothers.transpose.set_target(transpose);
            smoothers.detune.set_target(detune);
            if !freeze {
                if let Some(peak) = peak {
                    smoothers.freq.set_target(peak.frequency);
                    smoothers.amp.set_target(peak.amplitude);
                } else {
                    smoothers.amp.set_target(0.0);
                }
            }
        }
    }
}

pub struct Reconstructor {
    oscillators: [(
        SinOsc,
        SmoothedValue,
        SmoothedValue,
        SmoothedValue,
        SmoothedValue,
        SmoothedValue,
    ); 20],
    peak_analyzer: PeakAnalyzer,
    peak_tracker: PeakTracker,
    buffer: Ringbuffer,
    sample_rate: f32,
    freeze: bool,
    transpose: f32,
    detune: f32,
}

impl Reconstructor {
    pub fn new(sample_rate: f32) -> Self {
        let mut noise = noise(0);
        let oscillators = (0..20)
            .map(|_| {
                (
                    SinOsc::new(440.0, 0.0, 0.0),
                    SmoothedValue::new(440.0, 64),
                    SmoothedValue::new(0.0, 64),
                    SmoothedValue::new(1.0, 64),
                    SmoothedValue::new(noise.next_sample() as f32, 64),
                    SmoothedValue::new(0.0, 64),
                )
            })
            .collect::<Vec<(
                SinOsc,
                SmoothedValue,
                SmoothedValue,
                SmoothedValue,
                SmoothedValue,
                SmoothedValue,
            )>>()
            .try_into()
            .unwrap();
        let peak_analyzer = PeakAnalyzer::new(sample_rate);
        let peak_tracker = PeakTracker::new();
        let buffer = Ringbuffer::new(512);
        let freeze = false;
        let transpose = 1.0;
        let detune = 0.0;
        Self {
            oscillators,
            peak_analyzer,
            peak_tracker,
            buffer,
            sample_rate,
            freeze,
            transpose,
            detune,
        }
    }

    pub fn set_freeze(&mut self, status: bool) {
        self.freeze = status;
    }

    pub fn set_transpose(&mut self, amount: f32) {
        let value = 2_f32.powf(amount.clamp(-2.0, 2.0));
        self.transpose = value;
    }

    pub fn set_detune(&mut self, amount: f32) {
        self.detune = amount.clamp(0.0, 1.0);
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

        for (
            peak,
            (osc, freq_smoother, amp_smoother, transpose_smoother, rand_smoother, detune_smoother),
        ) in peaks.iter().zip(self.oscillators.iter_mut())
        {
            transpose_smoother.set_target(self.transpose);
            detune_smoother.set_target(self.detune);
            if let Some(peak) = peak {
                if !self.freeze {
                    freq_smoother.set_target(peak.frequency);
                    amp_smoother.set_target(peak.amplitude);
                }
                for sample in output.iter_mut() {
                    let rand_amount = 2_f32
                        .powf(rand_smoother.next() * 2.0 * detune_smoother.next())
                        .clamp(0.25, 4.0);
                    osc.set_frequency_hz(
                        freq_smoother.next() * transpose_smoother.next() * rand_amount,
                        self.sample_rate,
                    );
                    osc.set_amplitude(amp_smoother.next());
                    *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
                }
            } else {
                if !self.freeze {
                    amp_smoother.set_target(0.0);
                }
                for sample in output.iter_mut() {
                    let rand_amount = 2_f32
                        .powf(rand_smoother.next() * 2.0 * detune_smoother.next())
                        .clamp(0.25, 4.0);
                    osc.set_frequency_hz(
                        freq_smoother.next() * transpose_smoother.next() * rand_amount,
                        self.sample_rate,
                    );
                    osc.set_amplitude(amp_smoother.next());
                    *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::build_sample;

    #[test]
    fn test_draw_tracks() {
        let sample_a = build_sample(
            &[(440.0, 1.0, 0.0), (1000.0, 0.5, 0.0), (100.0, 0.0, 0.0)],
            512,
            48000.0,
        );
        let mut analyzer = PeakAnalyzer::new(48000.0);
        let peaks_a = analyzer.get_raw_peaks(&sample_a[0..512].try_into().unwrap());
        let mut peak_tracker = PeakTracker::new();
        peak_tracker.update_peaks(peaks_a);
        println!("PEAKS A: {:?}", peak_tracker.latest());
        let sample_b = build_sample(
            &[(450.0, 0.8, 0.0), (1100.0, 0.5, 0.0), (150.0, 1.0, 0.0)],
            512,
            48000.0,
        );
        let mut peaks_b = analyzer.get_raw_peaks(&sample_b[0..512].try_into().unwrap());
        peaks_b.reverse();
        peak_tracker.update_peaks(peaks_b);
        println!("PEAKS B: {:?}", peak_tracker.latest());
        let sample_c = build_sample(
            &[(430.0, 0.8, 0.0), (1150.0, 0.5, 0.0), (180.0, 0.5, 0.0)],
            512,
            48000.0,
        );
        let mut peaks_c = analyzer.get_raw_peaks(&sample_c[0..512].try_into().unwrap());
        peaks_c.reverse();
        peak_tracker.update_peaks(peaks_c);
        println!("PEAKS C: {:?}", peak_tracker.latest());

        let sample_d = build_sample(
            &[(430.0, 0.0, 0.0), (1150.0, 0.0, 0.0), (180.0, 0.0, 0.0)],
            512,
            48000.0,
        );
        let mut peaks_d = analyzer.get_raw_peaks(&sample_d[0..512].try_into().unwrap());
        peaks_d.reverse();
        peak_tracker.update_peaks(peaks_d);
        println!("PEAKS D: {:?}", peak_tracker.latest());
    }
}
