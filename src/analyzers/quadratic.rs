use crate::peak::Peak;
use crate::window::apply_hanning_window;
use assert_no_alloc::assert_no_alloc;
use realfft::num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex};

const MAX_PEAKS: usize = 20;

fn find_bin_freq_quadratic(bins: &[Complex<f32>], bin: usize) -> f32 {
    let previous_magnitude = bins[bin - 1].norm();
    let current_magnitude = bins[bin].norm();
    let next_magnitude = bins[bin + 1].norm();
    let detune = (next_magnitude - previous_magnitude)
        / (2. * (2. * current_magnitude - previous_magnitude - next_magnitude));
    return bin as f32 + detune;
}

fn find_top_20_bins(bins: &[Complex<f32>]) -> [Option<(usize, f32)>; MAX_PEAKS] {
    let mut peak_bins: [Option<(usize, f32)>; 256] = [None; 256];
    let mut peak_index = 0;
    let threshold = 0.6;
    let window_size = 512;
    let minimum_bin = 2;
    for bin in minimum_bin..window_size - 1 {
        let previous_magnitude = bins[bin - 1].norm();
        let previous2_magnitude = bins[bin - 2].norm();
        let magnitude = bins[bin].norm();
        let next_magnitude = bins[bin + 1].norm();
        let next2_magnitude = bins[bin + 2].norm();
        if magnitude > threshold
            && magnitude > previous_magnitude
            && magnitude > next_magnitude
            && magnitude > previous2_magnitude
            && magnitude > next2_magnitude
        {
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
    let mut top_20: [Option<(usize, f32)>; MAX_PEAKS] = [None; MAX_PEAKS];
    top_20.clone_from_slice(&peak_bins[0..MAX_PEAKS]);
    return top_20;
}

pub struct PeakAnalyzer {
    plan: std::sync::Arc<dyn RealToComplex<f32>>,
    fft_input: Vec<f32>,
    fft_scratch: Vec<Complex<f32>>,
    fft_output: Vec<Complex<f32>>,
    sample_rate: f32,
}

impl PeakAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
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
            sample_rate,
        }
    }

    pub fn get_raw_peaks(&mut self, input: &[f32; 512]) -> [Option<Peak>; MAX_PEAKS] {
        assert_no_alloc(|| {
            for x in self.fft_input.iter_mut() {
                *x = 0.0;
            }
            let windowed_input = apply_hanning_window::<512>(input);
            let fft_frame = &mut self.fft_input[0..512];
            for (x, y) in windowed_input.iter().zip(fft_frame.iter_mut()) {
                *y = *x;
            }
            let _result = self.plan.process_with_scratch(
                self.fft_input.as_mut_slice(),
                self.fft_output.as_mut_slice(),
                self.fft_scratch.as_mut_slice(),
            );
            let peak_bins = find_top_20_bins(self.fft_output.as_slice());
            let mut peaks: [Option<Peak>; MAX_PEAKS] = [None; MAX_PEAKS];
            let window_size = 512;
            let freq_per_bin = 0.5 * self.sample_rate / window_size as f32;

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::build_sample;

    #[test]
    fn test_get_raw_peaks() {
        let sample = build_sample(
            &[(440.0, 1.0, 0.0), (1000.0, 0.5, 0.0), (100.0, 0.0, 0.0)],
            512,
            48000.0,
        );
        let mut analyzer = PeakAnalyzer::new(48000.0);
        let peaks_a = analyzer.get_raw_peaks(&sample[0..512].try_into().unwrap());
        let expected = [
            Some(Peak {
                frequency: 439.69232,
                amplitude: 1.1020504,
            }),
            Some(Peak {
                frequency: 998.8542,
                amplitude: 0.5557409,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        assert_eq!(expected, peaks_a);
    }
}
