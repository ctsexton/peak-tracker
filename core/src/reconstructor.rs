use crate::analyzers::quadratic::PeakAnalyzer;
use crate::buffer::Ringbuffer;
use crate::osc::SinOsc;
use crate::peak::Peak;
use crate::smooth::SmoothedValue;
use crate::tracker::PeakTracker;
use crate::voice::{Event, Note, Synth, Voice};
use dasp::signal::noise;

#[derive(Debug)]
struct Smoothers {
    freq: SmoothedValue,
    amp: SmoothedValue,
    transpose: SmoothedValue,
    random: SmoothedValue,
    detune: SmoothedValue,
}

#[derive(Debug)]
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
        let (note_offset, note_amp) = {
            if let Some(note) = &self.note {
                let note_offset = note.note_number as i32 - MIDDLE_C as i32;
                (note_offset as f32, 0.25)
            } else {
                (0.0, 0.0)
            }
        };
        let freq_multiplier = 2_f32.powf(note_offset / 12.0);

        for Oscillator { osc, smoothers } in self.oscillators.iter_mut() {
            for sample in block.iter_mut() {
                let rand_amount = 2_f32
                    .powf(smoothers.random.next() * 2.0 * smoothers.detune.next())
                    .clamp(0.25, 4.0);
                osc.set_frequency_hz(
                    smoothers.freq.next()
                        * smoothers.transpose.next()
                        * rand_amount
                        * freq_multiplier,
                    self.sample_rate,
                );
                osc.set_amplitude(smoothers.amp.next() * note_amp);
                *sample = (*sample + osc.next()).clamp(-1.0, 1.0);
            }
        }
    }
}

impl ReconstructorVoice {
    fn new(sample_rate: f32) -> Self {
        let mut noise = noise(rand::random::<u64>());
        let oscillators = (0..20)
            .map(|_| Oscillator {
                osc: SinOsc::new(440.0, 0.0, 0.0),
                smoothers: Smoothers {
                    freq: SmoothedValue::new(440.0, 64),
                    amp: SmoothedValue::new(0.0, 64),
                    transpose: SmoothedValue::new(1.0, 64),
                    random: SmoothedValue::new(noise.next_sample() as f32, 64),
                    detune: SmoothedValue::new(0.0, 64),
                },
            })
            .collect::<Vec<Oscillator>>()
            .try_into()
            .unwrap();
        Self {
            sample_rate,
            note: None,
            oscillators,
        }
    }

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

struct ReconstructorSynth {
    voices: Vec<ReconstructorVoice>,
}

impl Synth<ReconstructorVoice> for ReconstructorSynth {
    fn get_voices(&self) -> &[ReconstructorVoice] {
        self.voices.as_slice()
    }

    fn get_voices_mut(&mut self) -> &mut [ReconstructorVoice] {
        self.voices.as_mut_slice()
    }
}

pub struct Reconstructor {
    peak_analyzer: PeakAnalyzer,
    peak_tracker: PeakTracker,
    buffer: Ringbuffer,
    sample_rate: f32,
    freeze: bool,
    transpose: f32,
    detune: f32,
    // default is used for non-synth mode
    // where there is a single always-on voice
    default_voice: ReconstructorVoice,
    synth_mode: bool,
    synth: ReconstructorSynth,
}

impl Reconstructor {
    pub fn new(sample_rate: f32) -> Self {
        let peak_analyzer = PeakAnalyzer::new(sample_rate);
        let peak_tracker = PeakTracker::new();
        let buffer = Ringbuffer::new(512);
        let freeze = false;
        let transpose = 1.0;
        let detune = 0.0;
        let voices = (0..8)
            .map(|_| ReconstructorVoice::new(sample_rate))
            .collect::<Vec<ReconstructorVoice>>();
        let synth = ReconstructorSynth { voices };
        let mut default_voice = ReconstructorVoice::new(sample_rate);
        default_voice.set_note(Some(Note {
            note_number: MIDDLE_C,
        }));
        Self {
            peak_analyzer,
            peak_tracker,
            buffer,
            sample_rate,
            freeze,
            transpose,
            detune,
            default_voice,
            synth_mode: false,
            synth,
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

    pub fn set_synth_mode(&mut self, is_active: bool) {
        self.synth_mode = is_active;
    }

    pub fn run(&mut self, input: &[f32], output: &mut [f32], events: &[Event]) {
        assert!(output.len() == input.len());
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

        if self.synth_mode {
            for voice in self.synth.voices.iter_mut() {
                voice.prepare_oscillators(peaks, self.freeze, self.transpose, self.detune);
            }
            self.synth.render_block(output, events);
        } else {
            self.default_voice
                .prepare_oscillators(peaks, self.freeze, self.transpose, self.detune);
            self.default_voice.render_block(output);
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
