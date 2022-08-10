use assert_no_alloc::assert_no_alloc;
use dasp::frame::Mono;
use dasp::signal::window::{hanning, Window};
use dasp::window::Hanning;
use realfft::num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex};
use std::f32::consts::PI;

pub fn apply_hanning_window<const SIZE: usize>(
    frame: &[f32; SIZE],
    window: Window<Mono<f32>, Hanning>,
) -> [f32; SIZE] {
    let mut output = [0_f32; SIZE];
    for ((index, x), [w]) in frame.iter().enumerate().zip(window) {
        output[index] = *x * w;
    }
    output
}

#[derive(Debug, Clone, Copy)]
pub struct Peak {
    pub frequency: f32,
    pub amplitude: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FrequencyDistance {
    a: usize,      // index of peak in array A
    b: usize,      // index of peak in array B
    distance: f32, // distance in frequency
}

fn find_bin_freq_quadratic(bins: &[Complex<f32>], bin: usize) -> f32 {
    let previous_magnitude = bins[bin - 1].norm();
    let current_magnitude = bins[bin].norm();
    let next_magnitude = bins[bin + 1].norm();
    let detune = (next_magnitude - previous_magnitude)
        / (2. * (2. * current_magnitude - previous_magnitude - next_magnitude));
    return bin as f32 + detune;
}

fn find_top_20_bins(bins: &[Complex<f32>]) -> [Option<(usize, f32)>; 20] {
    let mut peak_bins: [Option<(usize, f32)>; 256] = [None; 256];
    let mut peak_index = 0;
    let threshold = 1.0;
    let window_size = 512;
    for bin in 1..window_size - 1 {
        let previous_magnitude = bins[bin - 1].norm();
        let magnitude = bins[bin].norm();
        let next_magnitude = bins[bin + 1].norm();
        if magnitude > threshold && magnitude > previous_magnitude && magnitude > next_magnitude {
            peak_bins[peak_index] = Some((bin, magnitude));
            peak_index += 1;
            if peak_index == 256 {
                break;
            }
        }
    }
    peak_bins.sort_unstable_by(|a, b| {
        if a.is_none() {
            return std::cmp::Ordering::Less;
        } else if b.is_none() {
            return std::cmp::Ordering::Greater;
        } else {
            return a.unwrap().1.partial_cmp(&b.unwrap().1).unwrap();
        }
    });
    peak_bins.reverse();
    let mut top_20: [Option<(usize, f32)>; 20] = [None; 20];
    top_20.clone_from_slice(&peak_bins[0..20]);
    return top_20;
}

const NUM_PEAKS: usize = 20;

/// Finds the frequency distance between all peaks in a and all peaks in b
fn calculate_peak_distances(
    a: &[Option<Peak>],
    b: &[Option<Peak>],
    output: &mut [Option<FrequencyDistance>],
) {
    assert_eq!(a.len() * b.len(), output.len());

    for (index_a, item_a) in a.iter().enumerate() {
        for (index_b, item_b) in b.iter().enumerate() {
            let index = index_a * a.len() + index_b;
            if let (Some(item_a), Some(item_b)) = (item_a, item_b) {
                let distance = (item_a.frequency - item_b.frequency).abs();
                output[index] = Some(FrequencyDistance {
                    a: index_a,
                    b: index_b,
                    distance,
                });
            } else {
                output[index] = None;
            }
        }
    }
}

const MAX_PEAKS: usize = 20;
const MAX_DISTANCE: f32 = 187.5;

fn match_closest_peaks<const NPEAKS: usize>(
    a: &[Option<Peak>; NPEAKS],
    b: &[Option<Peak>; NPEAKS],
) -> [Option<usize>; NPEAKS]
where
    [(); NPEAKS * NPEAKS]: Sized,
{
    let mut distances: [Option<FrequencyDistance>; NPEAKS * NPEAKS] = [None; NPEAKS * NPEAKS];
    calculate_peak_distances(a, b, &mut distances);
    distances.sort_unstable_by(|a, b| {
        if let (Some(a), Some(b)) = (a, b) {
            return a.distance.partial_cmp(&b.distance).unwrap();
        } else if let Some(_a) = a {
            return std::cmp::Ordering::Less;
        } else {
            return std::cmp::Ordering::Greater;
        }
    });
    let mut matches: [Option<usize>; NPEAKS] = [None; NPEAKS];
    let mut taken_from_a = [false; NPEAKS];
    let mut taken_from_b = [false; NPEAKS];
    let mut match_index = 0;
    for item in distances.iter() {
        if let Some(item) = item {
            if match_index >= matches.len() || item.distance > MAX_DISTANCE {
                break;
            }
            if !taken_from_a[item.a] && !taken_from_b[item.b] {
                taken_from_a[item.a] = true;
                taken_from_b[item.b] = true;
                matches[item.a] = Some(item.b);
                match_index += 1;
            }
        }
    }
    matches
}

pub struct PeakAnalyzer {
    plan: std::sync::Arc<dyn RealToComplex<f32>>,
    fft_input: Vec<f32>,
    fft_scratch: Vec<Complex<f32>>,
    fft_output: Vec<Complex<f32>>,
}

impl PeakAnalyzer {
    pub fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let plan = planner.plan_fft_forward(1024);
        let fft_input = plan.make_input_vec();
        let fft_scratch = plan.make_scratch_vec();
        let fft_output = plan.make_output_vec();
        Self {
            plan,
            fft_input,
            fft_scratch,
            fft_output,
        }
    }

    pub fn get_raw_peaks(&mut self, input: &[f32; 512]) -> [Option<Peak>; 20] {
        assert_no_alloc(|| {
            for x in self.fft_input.iter_mut() {
                *x = 0.0;
            }
            let window = hanning::<Mono<f32>>(512);
            let windowed_input = apply_hanning_window::<512>(input, window);
            let fft_frame = &mut self.fft_input[0..512];
            for (x, y) in windowed_input.iter().zip(fft_frame.iter_mut()) {
                *y = *x;
            }
            self.plan.process_with_scratch(
                self.fft_input.as_mut_slice(),
                self.fft_output.as_mut_slice(),
                self.fft_scratch.as_mut_slice(),
            );
            let peak_bins = find_top_20_bins(self.fft_output.as_slice());
            let mut peaks: [Option<Peak>; 20] = [None; 20];
            let sample_rate = 48000.0;
            let window_size = 512;
            let freq_per_bin = 0.5 * sample_rate / window_size as f32;

            for (peak, peak_bin_pair) in peaks.iter_mut().zip(peak_bins.iter()) {
                if let Some((peak_bin, magnitude)) = peak_bin_pair {
                    let frequency = find_bin_freq_quadratic(self.fft_output.as_slice(), *peak_bin)
                        * freq_per_bin;
                    let amplitude = *magnitude / 512.0_f32.sqrt() * 0.2;
                    *peak = Some(Peak {
                        frequency,
                        amplitude,
                    });
                }
            }
            peaks
        })
    }
}

pub struct PeakTracker {
    peaks: [Option<Peak>; 20],
}

impl PeakTracker {
    pub fn new() -> Self {
        let peaks = [None; 20];
        Self { peaks }
    }

    pub fn update_peaks(&mut self, mut batch: [Option<Peak>; 20]) {
        assert_no_alloc(|| {
            let matches = match_closest_peaks(&self.peaks, &batch);
            let mut new_peaks: [Option<Peak>; 20] = [None; 20];
            for (index, item) in matches.iter().enumerate() {
                if let Some(target) = *item {
                    new_peaks[index] = batch[target].take();
                }
            }
            let mut unmapped_peaks = batch.iter_mut().flatten();
            for new_peak in new_peaks.iter_mut().filter(|p| p.is_none()) {
                if let Some(peak) = unmapped_peaks.next() {
                    *new_peak = Some(*peak);
                } else {
                    break;
                }
            }
            self.peaks = new_peaks;
        })
    }

    pub fn latest(&self) -> &[Option<Peak>; 20] {
        &self.peaks
    }
}

fn build_sample(partials: &[(f32, f32, f32)], size: usize, sample_rate: f32) -> Vec<f32> {
    let mut sample = vec![0_f32; size];
    for (frequency, amplitude, phase) in partials {
        for (index, x) in sample.iter_mut().enumerate() {
            *x = *x
                + amplitude
                    * (phase * 2. * PI + index as f32 * 2. * PI * frequency / sample_rate).sin()
        }
    }
    sample
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_peak_distances() {
        let a = [
            Some(Peak {
                frequency: 20.0,
                amplitude: 1.0,
            }),
            Some(Peak {
                frequency: 30.0,
                amplitude: 1.0,
            }),
        ];
        let b = [
            Some(Peak {
                frequency: 21.0,
                amplitude: 1.0,
            }),
            Some(Peak {
                frequency: 25.0,
                amplitude: 1.0,
            }),
        ];
        let mut result: [Option<FrequencyDistance>; 4] = [None; 4];
        calculate_peak_distances(&a, &b, &mut result);
        let expected_result: Vec<Option<FrequencyDistance>> =
            [(0, 0, 1.0), (0, 1, 5.0), (1, 0, 9.0), (1, 1, 5.0)]
                .into_iter()
                .map(|(a, b, distance)| Some(FrequencyDistance { a, b, distance }))
                .collect();
        assert_eq!(result.as_slice(), expected_result.as_slice());
    }

    #[test]
    fn sort_peak_distances() {
        let mut distances = [(0, 1, 5.0), (0, 0, 1.0), (1, 0, 9.0), (1, 1, 5.0)];
        distances.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        let expected_sorting = [(0, 0, 1.0), (0, 1, 5.0), (1, 1, 5.0), (1, 0, 9.0)];
        assert_eq!(distances, expected_sorting);
    }

    #[test]
    fn test_apply_hanning() {
        let frame = [2_f32; 5];
        let window = hanning::<Mono<f32>>(5);
        let windowed = apply_hanning_window::<5>(&frame, window);
        assert_eq!([0.0, 1.0, 2.0, 1.0, 0.0], windowed);
    }

    #[test]
    fn test_build_sample() {
        let partials = [(2.0, 1.0, 0.0)];
        let sample = build_sample(&partials, 8, 8.0);
        let expected_sample = [0.0, 1.0, 0.0, -1.0];
        for (item, expected) in sample.iter().zip(expected_sample.iter()) {
            let diff = (*item - *expected).abs();
            assert!(diff < 0.001);
        }
    }

    #[test]
    fn test_get_raw_peaks() {
        let sample = build_sample(
            &[(440.0, 1.0, 0.0), (1000.0, 0.5, 0.0), (100.0, 0.0, 0.0)],
            512,
            48000.0,
        );
        let mut analyzer = PeakAnalyzer::new();
        let _peaks_a = analyzer.get_raw_peaks(&sample[0..512].try_into().unwrap());
    }

    #[test]
    fn test_draw_tracks() {
        let sample_a = build_sample(
            &[(440.0, 1.0, 0.0), (1000.0, 0.5, 0.0), (100.0, 0.0, 0.0)],
            512,
            48000.0,
        );
        let mut analyzer = PeakAnalyzer::new();
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
    }
}
